use std::{collections::HashMap, io::BufRead, marker::PhantomData};

use derive_more as dm;
pub use fst;
pub use trie_rs as trie;

pub trait Hash: std::hash::Hash + Ord {
    fn hash_str(str: impl AsRef<str>) -> Self;
}

#[derive(
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Copy,
    dm::AsRef,
    dm::AsMut,
    dm::From,
    dm::Deref,
    dm::DerefMut,
)]
pub struct WadHash(pub u64);
#[derive(
    Hash,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Copy,
    dm::AsRef,
    dm::AsMut,
    dm::From,
    dm::Deref,
    dm::DerefMut,
)]
pub struct BinHash(pub u64);

pub struct WadHasher;
pub struct BinHasher;

pub struct Hashtable<H: Hash> {
    pub hashes: HashMap<H, String>,
    hasher: PhantomData<H>,
}

impl<H: Hash> Hashtable<H> {
    /// Read from CommunityDragon/Data style hashtable files
    pub fn read_hashtable_file<R: BufRead>(reader: &mut R) -> std::io::Result<Self> {
        todo!()
    }
}

impl<H: Hash> From<HashMap<H, String>> for Hashtable<H> {
    fn from(value: HashMap<H, String>) -> Self {
        Self {
            hashes: value,
            hasher: PhantomData,
        }
    }
}

#[cfg(feature = "fst")]
pub type Trie = trie_rs::Trie<u8>;

#[cfg(feature = "trie")]
mod trie_impl {
    use crate::{Hash, Hashtable};

    impl<H: Hash> From<Hashtable<H>> for trie_rs::Trie<u8> {
        fn from(value: Hashtable<H>) -> Self {
            trie_rs::Trie::from_iter(value.hashes.into_values())
        }
    }
}

#[cfg(feature = "fst")]
pub type Fst = fst::Set<Vec<u8>>;

#[cfg(feature = "fst")]
mod fst_impl {
    use crate::{Hash, Hashtable};
    use std::marker::PhantomData;

    impl<H: Hash> TryFrom<fst::Set<Vec<u8>>> for Hashtable<H> {
        fn try_from(value: fst::Set<Vec<u8>>) -> Result<Self, Self::Error> {
            Ok(Self {
                hashes: value
                    .into_fst()
                    .stream()
                    .into_str_keys()?
                    .into_iter()
                    .map(|v| (H::hash_str(&v), v))
                    .collect(),
                hasher: PhantomData,
            })
        }

        type Error = fst::Error;
    }
    impl<H: Hash> From<Hashtable<H>> for fst::Set<Vec<u8>> {
        fn from(value: Hashtable<H>) -> Self {
            let mut values = value.hashes.into_values().collect::<Vec<_>>();
            values.sort();
            fst::Set::from_iter(values).unwrap()
        }
    }
}

impl Hash for WadHash {
    fn hash_str(str: impl AsRef<str>) -> Self {
        // 64-bit XXH64 with seed 0, applied on lowercased string value
        Self(xxhash_rust::xxh64::xxh64(
            str.as_ref().to_lowercase().as_bytes(),
            0,
        ))
    }
}
