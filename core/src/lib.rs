use std::{collections::BTreeMap, hash::Hash, io::BufRead, marker::PhantomData};

pub use fst;
pub use trie_rs as trie;

pub trait Hasher {
    type Output: Hash + Ord;
    fn hash(str: impl AsRef<str>) -> Self::Output;
}

pub struct WadHasher;
pub struct BinHasher;

pub struct Hashtable<H: Hasher> {
    pub hashes: BTreeMap<H::Output, String>,
    hasher: PhantomData<H>,
}

impl<H: Hasher> Hashtable<H> {
    // pub fn from_trie(trie: HashtableTrie) -> Self {
    //     let hashes = trie
    //         .iter()
    //         .map(|entry: String| (H::hash(&entry), entry.to_string()))
    //         .collect();
    //     Self {
    //         hashes,
    //         hasher: PhantomData,
    //     }
    // }

    /// Read from CommunityDragon/Data style hashtable files
    pub fn read_hashtable_file<R: BufRead>(reader: &mut R) -> std::io::Result<Self> {
        todo!()
    }
}

impl<H: Hasher> From<BTreeMap<H::Output, String>> for Hashtable<H> {
    fn from(value: BTreeMap<H::Output, String>) -> Self {
        Self {
            hashes: value,
            hasher: PhantomData,
        }
    }
}

impl<H: Hasher> From<Hashtable<H>> for trie_rs::Trie<u8> {
    fn from(value: Hashtable<H>) -> Self {
        trie_rs::Trie::from_iter(value.hashes.into_values())
    }
}
impl<H: Hasher> From<Hashtable<H>> for trie_rs::map::TrieSet<u8> {
    fn from(value: Hashtable<H>) -> Self {
        trie_rs::map::TrieSet::from_iter(value.hashes.into_values())
    }
}

impl<H: Hasher> TryFrom<fst::Set<Vec<u8>>> for Hashtable<H> {
    fn try_from(value: fst::Set<Vec<u8>>) -> Result<Self, Self::Error> {
        Ok(Self {
            hashes: value
                .into_fst()
                .stream()
                .into_str_keys()?
                .into_iter()
                .map(|v| (H::hash(&v), v))
                .collect(),
            hasher: PhantomData,
        })
    }

    type Error = fst::Error;
}
impl<H: Hasher> From<Hashtable<H>> for fst::Set<Vec<u8>> {
    fn from(value: Hashtable<H>) -> Self {
        let mut values = value.hashes.into_values().collect::<Vec<_>>();
        values.sort();
        fst::Set::from_iter(values).unwrap()
    }
}

impl Hasher for WadHasher {
    type Output = u64;

    fn hash(str: impl AsRef<str>) -> Self::Output {
        // 64-bit XXH64 with seed 0, applied on lowercased string value
        xxhash_rust::xxh64::xxh64(str.as_ref().to_lowercase().as_bytes(), 0)
    }
}
