//! Capacity type/s describing all the capacities requires within the io_uring bearer.
//! We integrate the capacity crate to describe the capacities.

/// Describes all the different intended fixed capacity used in the bearer.
/// ```rust
/// use io_uring_bearer::BearerCapacityKind;
/// use capacity::{Capacity, Setting};
///
/// #[derive(Clone, Debug)]
/// pub struct MyCapacity;
///
/// impl Setting<BearerCapacityKind> for MyCapacity {
///     fn setting(&self, v: &BearerCapacityKind) -> usize {
///         match v {
///             BearerCapacityKind::CoreQueue => 1,
///             BearerCapacityKind::RegisteredFd => 2,
///             BearerCapacityKind::PendingCompletions => 3,
///             BearerCapacityKind::Buffers => 4,
///             BearerCapacityKind::Futexes => 5,
///         }
///     }
/// }
/// fn main() {
///   let my_cap = Capacity::<MyCapacity, BearerCapacityKind>::with_planned(MyCapacity {});
/// }
/// ```
#[derive(Clone, Debug)]
pub enum BearerCapacityKind {
    /// io_uring Queue capacity, in power of twos.
    CoreQueue,
    /// How many filehandles can be registered.
    RegisteredFd,
    /// How many pending Completions.
    PendingCompletions,
    /// How many buffers can be managed.
    Buffers,
    /// How many futexes can be managed.
    Futexes,
}
