//! Slab Types
//! These types are typically stored in slab / slotmap that can gurantee
//! memory address would not change for example when used in ffi calls
//! which may complete asynchronously and may where the records may be
//! retained beyond the initial call.

// Accept async return stores IP + Port of accepted (new) socket
pub(crate) mod accept;
#[doc(inline)]
pub use accept::AcceptRec;

// Epoll on wait returns user_data u64 and triggered events (we init-zeroo)
pub(crate) mod epoll_ctl;
#[doc(inline)]
pub use epoll_ctl::EpollRec;

// Buffers are shared between the kernel and userspace and may be returned
// on Read/Recv etc. calls
pub(crate) mod buffer;
#[doc(inline)]
pub use buffer::ProvideBufferRec;

/// Slottable is the trait for storage slab-slotmaps for the purpose of
/// keeping the underlying request data alive when kernel is either modifying
/// or keeping a reference of the said data through raw pointers.
///
/// This trait is opinionated into purpose of network server proramming that may
/// have high and low load periods with ramp-up and downs.
///
/// The underlying implementation does not need to be Sync or Send and typically
/// should live within one thread only requiring no synchronisation / atomics.
///
/// The user of the slab-slotmap must guarantee that the owned slotmap itself is
/// not dropped whilst guaranteeing the items inside are not moving or dropped.
///
/// # Required Guarantees
///
/// The slab-slotmap implementing this trait must:
///
/// 1. Keep the memory addresses stable as-in self-referential structs
/// 2. Provide free slot and upon freeing the slot must be re-usable
/// 3. Lookable key by usize that can be copy-referenced without pointer access
/// 4. Upon insertion keep the underlying data immutable
/// 5. Not to allow unaligned field reference access
/// 6. Must not leak memory beyond the fixed capacity max.
/// 7. Must be tested for 1-6 and documented for A-E and perhaps benchmarked.
///
/// # Desired Properties
///
/// A. Fast random access via key that is usize
/// B. Occupying takes a sequential usize id
/// C. Not re-using sequential usize id until it is recycled at usize::MAX
/// D; Ability to free-up memory e.g. in acses of ramp-up / down high/low loads.
/// E. Minimal memory usage for free slots
///
pub trait Slottable<Slotter, T> {
    type Error;
    /// Provided with capacity the impl must keep the underlying T addresses stable.
    /// The capacity must be fixed and must not change.
    fn with_fixed_capacity(_: usize) -> Result<Slotter, Self::Error>;
    /// Take the next free slot, ideally with least re-used ID and return it's key ID
    fn take_next_with(&mut self, _: T) -> Result<usize, Self::Error>;
    /// Mark a given slot for re-use
    fn mark_for_reuse(&mut self, _: usize) -> Result<T, Self::Error>;
    /// The capacity of the slab-slotmap
    fn capacity(&self) -> Option<usize>;
    /// Occupied count of the slab-slotmap
    fn count_occupied(&self) -> usize;
    /// Free count of the slab-slotap
    fn count_free(&self) -> usize;
    /// Remaining capacity of teh slab-slotmap
    fn remaining(&self) -> Option<usize>;
    /// Reap memory that can be freed opportunistically-optionally but keep the capacity intanct.
    /// This may mean wiping out entries at the tail and re-allocating at the end etc.
    /// Implementations that don't provide this should return None and the ones providing it
    /// must return the number of slots affected denoting the expected effetiveness of it.
    /// This is an opportunity to reap the freelist or gc in the periods that may afford slowness
    /// traded for opportunity to free up operating memory.
    fn reap(&mut self) -> Option<usize>;
}
