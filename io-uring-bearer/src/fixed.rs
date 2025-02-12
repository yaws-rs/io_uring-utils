//! Fixed register mappings including registered filehandles handling.
//! Some io_uring operations require "fixed" or "registered" filehandle (or buffer)
//! Keeping track of these things here.

use hashbrown::HashMap as SelectedHashMap;
use io_uring_fd::RegisteredFd;
use nohash_hasher::BuildNoHashHasher as SelectedHasher;
use slab::{Slab, VacantEntry};

#[derive(Clone, Debug)]
pub(crate) struct FixedFdRegister {
    fno2fixed: SelectedHashMap<u32, u32, SelectedHasher<u32>>,
    fixed_ord: Slab<(u32, RegisteredFd)>,
    capacity: u32,
}

impl FixedFdRegister {
    pub(crate) fn with_fixed_capacity(cap: u32) -> FixedFdRegister {
        let fno2fixed: SelectedHashMap<u32, u32, SelectedHasher<u32>> =
            SelectedHashMap::<u32, u32, SelectedHasher<u32>>::with_capacity_and_hasher(
                cap as usize,
                SelectedHasher::default(),
            );
        FixedFdRegister {
            fno2fixed,
            capacity: cap,
            fixed_ord: Slab::with_capacity(cap as usize),
        }
    }
    #[inline]
    pub(crate) fn vacant_entry(&mut self) -> VacantEntry<'_, (u32, RegisteredFd)> {
        self.fixed_ord.vacant_entry()
    }
    #[inline]
    pub(crate) fn len(&self) -> u32 {
        self.fixed_ord.len() as u32
    }
    #[inline]
    pub(crate) fn capacity(&self) -> u32 {
        self.capacity
    }
    #[inline]
    pub(crate) fn get(&self, i: u32) -> Option<&(u32, RegisteredFd)> {
        self.fixed_ord.get(i as usize)
    }
    #[inline]
    pub(crate) fn insert(&mut self, i: (u32, RegisteredFd)) -> u32 {
        self.fixed_ord.insert(i) as u32
    }
}
