use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    io::BufRead,
    marker::PhantomData,
};

use derive_more as dm;

pub trait Hash: std::hash::Hash + Ord {
    fn hash_str(str: impl AsRef<str>) -> Self;
}

#[derive(
    Debug,
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

impl Display for WadHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:0>16x}", self.0)
    }
}

#[derive(
    Debug,
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
pub struct BinHash(pub u32);

impl Display for BinHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:0>8x}", self.0)
    }
}

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

impl<H: Hash, T: IntoIterator<Item = (H, String)>> From<T> for Hashtable<H> {
    fn from(values: T) -> Self {
        Self {
            hashes: values.into_iter().collect(),
            hasher: PhantomData,
        }
    }
}

#[cfg(feature = "trie")]
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
pub use fst;
#[cfg(feature = "fst")]
pub type Fst = fst::Set<Vec<u8>>;

#[cfg(feature = "fst")]
mod fst_impl {
    use crate::{Hash, Hashtable};

    impl<H: Hash> Hashtable<H> {
        pub fn from_fst(fst: super::Fst) -> Result<Self, fst::Error> {
            Ok(fst
                .into_fst()
                .stream()
                .into_str_keys()?
                .into_iter()
                .map(|v| (H::hash_str(&v), v))
                .into())
        }
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
