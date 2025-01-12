//! Kernel / Userspace ownership status record

/// Ownership denotes where the ownership stands in long living records and where it is important to know it's status.
/// All Accept, EpollCtl and Buffer records have varying dynamics how these
/// are allocated and re-used and this requires a flexible type given re-allocation
/// may be expensive in case of buffers whilst desierable just re-create record in
/// Accept or EpollCtl case.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Owner {
    /// Record was created (default)
    #[default]
    Created,
    /// Record is owned by the Kernel
    Kernel,
    /// Record is owned by the user(space)
    User,
    /// Record is marked for re-use (e.g. expensive allocation)
    /// Typical user: BufferRec
    Reusable,
}
