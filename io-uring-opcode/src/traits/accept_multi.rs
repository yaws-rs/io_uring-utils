//! AcceptMulti extension trait

/// Connect Expansion trait
pub trait OpExtAcceptMulti {
    /// Underlying Fixed Fd ref
    fn fixed_fd(&self) -> u32;
}
