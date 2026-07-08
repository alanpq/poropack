use std::{
    cell::RefCell,
    collections::HashMap,
    io::{self, Cursor, Read, Seek, Write},
    mem,
    num::NonZero,
    rc::Rc,
};

use lru::LruCache;

type Index<H> = HashMap<H, (u16, u32)>;

pub const MAGIC: &[u8; 8] = b"POROBLOB";

pub struct Poroblob<H: ltk_hash::Hash, B> {
    pub block_size: u16,
    pub index: Index<H>,
    pub blocks: Vec<BlockHeader>,
    pub block_data_off: usize,
    pub block_data: B,
}

#[derive(Default)]
struct Block {
    /// byte offset of the start of this block
    pub offset: u32,
    /// number of entries in this block
    pub entries: u16,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct BlockHeader {
    pub offset: u32,
    /// length of block data (compressed)
    pub len: u32,
}

pub struct Builder<H: ltk_hash::Hash> {
    /// size of block (in entries)
    block_size: u16,

    index: Index<H>,

    data: Vec<u8>,

    /// info about current block we're writing to
    block: Block,
    blocks: Vec<BlockHeader>,
}

impl<H: ltk_hash::Hash> Builder<H> {
    pub fn new(block_size: u16) -> Self {
        Self {
            block_size,
            index: HashMap::new(),
            data: Vec::new(),
            block: Block::default(),
            blocks: Vec::new(),
        }
    }

    fn flush_block(&mut self) -> io::Result<()> {
        let initial_len = self.data.len();

        let mut encoder = zstd::Encoder::new(&mut self.data, 3)?;
        encoder.write_all(&self.block.data)?;
        encoder.finish()?;

        self.blocks.push(BlockHeader {
            offset: self.block.offset,
            len: u32::try_from(self.data.len() - initial_len).unwrap(/* TODO: err */),
        });

        self.block.data.clear();
        self.block.entries = 0;
        self.block.offset = u32::try_from(self.data.len()).unwrap(/* TODO: err */);

        Ok(())
    }

    pub fn insert(&mut self, str: impl AsRef<str>) -> io::Result<()> {
        let str = str.as_ref();

        let hash = H::hash_str(str);

        let b = str.as_bytes();
        let entry_off = u32::try_from(self.block.data.len()).unwrap(/* TODO: err */);
        self.block.data.extend_from_slice(b);
        self.block.data.push(b'\0');

        self.index.insert(
            hash,
            (
                u16::try_from(self.blocks.len()).unwrap(/* TODO: err */),
                entry_off,
            ),
        );

        self.block.entries += 1;

        // current block is full, we move to next
        if self.block.entries >= self.block_size {
            self.flush_block()?;
        }
        Ok(())
    }

    pub fn extend<E, I>(&mut self, entries: E) -> io::Result<()>
    where
        E: IntoIterator<Item = I>,
        I: AsRef<str>,
    {
        for e in entries {
            self.insert(e)?;
        }
        Ok(())
    }

    pub fn write(mut self, writer: &mut (impl Write + ?Sized)) -> io::Result<()> {
        self.flush_block()?;

        use byteorder::{WriteBytesExt as _, LE};
        writer.write_all(MAGIC)?; // magic
        writer.write_u8(1)?; // version

        writer.write_u16::<LE>(self.block_size)?;
        writer.write_u16::<LE>(u16::try_from(self.blocks.len()).unwrap(/* TODO: err */))?;
        writer.write_u32::<LE>(u32::try_from(self.index.len()).unwrap(/* TODO: err */))?;

        for block in &self.blocks {
            writer.write_u32::<LE>(block.len)?;
        }

        for (k, v) in &self.index {
            k.to_writer::<LE>(writer)?;
            writer.write_u16::<LE>(v.0)?;
            writer.write_u32::<LE>(v.1)?;
        }

        writer.write_all(&self.data)?;

        Ok(())
    }
}

impl<H: ltk_hash::Hash> Poroblob<H, ()> {
    pub fn from_reader(reader: &mut (impl Read + ?Sized)) -> io::Result<Self> {
        use byteorder::{ReadBytesExt as _, LE};

        let mut magic = [0; 8];
        reader.read_exact(&mut magic)?;
        assert_eq!(magic, *MAGIC);

        let version = reader.read_u8()?;
        assert_eq!(version, 1);

        let block_size = reader.read_u16::<LE>()?;
        let block_count = reader.read_u16::<LE>()?;
        let entry_count = reader.read_u32::<LE>()?;

        let mut blocks = Vec::with_capacity(block_count.into());

        let mut off = 0;
        for _ in 0..block_count {
            let len = reader.read_u32::<LE>()?;
            blocks.push(BlockHeader { offset: off, len });
            off += len;
        }

        let mut index = HashMap::with_capacity(usize::try_from(entry_count).unwrap());

        for _ in 0..entry_count {
            index.insert(
                H::from_reader::<LE>(reader)?,
                (reader.read_u16::<LE>()?, reader.read_u32::<LE>()?),
            );
        }

        Ok(Self {
            block_data_off: 9
                + 2
                + 2
                + 4
                + blocks.len() * 4
                + index.len() * (size_of::<H>() + 2 + 4),
            block_size,
            index,
            blocks,
            block_data: (),
        })
    }

    pub fn with_block_data<B>(self, block_data: B) -> Poroblob<H, B> {
        Poroblob {
            block_size: self.block_size,
            index: self.index,
            blocks: self.blocks,

            block_data_off: self.block_data_off,
            block_data,
        }
    }
}

pub struct LruBlocks<T: AsRef<[u8]>> {
    lru: RefCell<LruCache<BlockHeader, Option<Rc<[u8]>>>>,
    data: T,
}

impl<T: AsRef<[u8]>> LruBlocks<T> {
    pub fn new(data: T, capacity: NonZero<usize>) -> Self {
        Self {
            lru: LruCache::new(capacity).into(),
            data,
        }
    }
}

impl<T: AsRef<[u8]>> LookupBlock for LruBlocks<T> {
    fn get_block(&self, offset: usize, block: BlockHeader) -> Option<Rc<[u8]>> {
        self.lru
            .borrow_mut()
            .get_or_insert(block, || self.data.get_block(offset, block))
            .clone()
    }
}

pub trait LookupBlock {
    fn get_block(&self, offset: usize, block: BlockHeader) -> Option<Rc<[u8]>>;
}

impl<B: AsRef<[u8]>> LookupBlock for B {
    fn get_block(&self, offset: usize, block: BlockHeader) -> Option<Rc<[u8]>> {
        let off = offset + usize::try_from(block.offset).unwrap();
        let len = usize::try_from(block.len).unwrap();

        let data = zstd::decode_all(&self.as_ref()[off..off + len]).unwrap();
        Some(data.into())
    }
}

impl<H: ltk_hash::Hash, B: LookupBlock> Poroblob<H, B> {
    pub fn get(&self, hash: impl Into<H>) -> Option<String> {
        let hash = hash.into();
        let (block_idx, entry_off) = self.index.get(&hash)?;
        let block = self.blocks.get(usize::from(*block_idx))?;

        let data = self.block_data.get_block(self.block_data_off, *block)?;
        let data = String::from_utf8_lossy(&data[usize::try_from(*entry_off).unwrap()..]);

        let (entry, _) = data.split_once('\0')?;

        Some(entry.to_string())
    }
}

// pub struct Header<H: ltk_hash::Hash> {
//     mask: H,
//     block_size: u32,
//     block_offsets: Vec<u32>,
//     table: Vec<Option<(H, u32)>>,
// }

// impl<H: ltk_hash::Hash> Header<H> {
//     pub fn from_reader(r: &mut (impl Read + ?Sized)) -> io::Result<Self> {
//         use byteorder::{ReadBytesExt as _, LE};

//         let mask = H::from_reader::<LE>(r)?;
//         let block_size = r.read_u32::<LE>()?;

//         let block_offsets = todo!();
//         let table = todo!();

//         Ok(Self {
//             mask,
//             block_size,
//             block_offsets,
//             table,
//         })
//     }
// }

// trait HeaderExt<H: ltk_hash::Hash> {
//     fn table_index(&self, hash: H) -> usize;
//     fn next_table_index(&self, hash: H) -> usize;
// }

// pub struct Poroblob<S: AsRef<[u8]>, H: ltk_hash::Hash> {
//     header: Header<H>,
//     blocks: S,
// }

// // impl<S: AsRef<[u8]> + Read> Poroblob<S> {
// //     // for basic reading from file/other reader + subsequent reading of blocks from that same reader impl
// //     pub fn from_reader(r: impl Read + Seek) -> io::Result<Self> {
// //         todo!()
// //     }
// //     // to support direct slice stuff like mmap
// //     pub fn from_slice(s: impl AsRef<[u8]>) -> io::Result<Self> {
// //         todo!()
// //     }
// // }

// impl<S: AsRef<[u8]>, H: ltk_hash::Hash> Poroblob<S, H> {
//     #[inline]
//     fn find_entry(&self, hash: H) -> Option<u32> {
//         let mut idx = (hash & self.header.mask);

//         loop {
//             match self.header.table[idx] {
//                 None => return None,
//                 Some((h, v)) if h == hash => return Some(v),
//                 _ => idx = (idx + 1) & self.header.mask,
//             }
//         }
//     }

//     fn block_for(&self, entry_id: u32) -> usize {
//         (entry_id as usize) / usize::try_from(self.header.block_size).unwrap()
//     }

//     fn decode_block(&self, block: usize) -> Vec<String> {
//         let blocks = self.blocks.as_ref();
//         let start = self.header.block_offsets[block] as usize;
//         let end = if block + 1 < self.header.block_offsets.len() {
//             self.header.block_offsets[block + 1] as usize
//         } else {
//             blocks.len()
//         };

//         let compressed = &blocks[start..end - start];

//         let decompressed = zstd::decode_all(compressed).unwrap();

//         decompressed
//             .split(|&b| b == b'\n')
//             .map(|s| unsafe { String::from_utf8_unchecked(s.to_vec()) })
//             .collect()
//     }

//     pub fn get(&self, hash: u64) -> Option<String> {
//         let entry_id = self.find_entry(hash)?;
//         let block = self.block_for(entry_id);

//         let decoded = self.decode_block(block);

//         let base = block * usize::try_from(self.header.block_size).unwrap();
//         let local = (entry_id as usize) - base;

//         decoded.into_iter().nth(local)
//     }
// }
