//! Socket required types

/// What kind of filehandle we expect to be returned from kernel as result                                   
#[derive(Clone, Debug, PartialEq)]
pub enum TargetFdType {
    /// Regular filehandle which is not considered "fixed" in io_uring                                       
    Regular,
    /// Automaticly assigned io_uring associated Fixed Fd                                                    
    ///                                                                                                      
    /// # Note                                                                                               
    ///                                                                                                      
    /// There must be free entries (-1) in the registered filehandles table.                                 
    FixedAuto,
    /// Manually pinned io_uring associated Fixed Fd                                                         
    FixedManual(u32),
}
