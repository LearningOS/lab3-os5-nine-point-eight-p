pub type OSResult<T = ()> = Result<T, (ErrorSource, ErrorType)>;

#[derive(Clone, Copy, Debug)]
pub enum ErrorSource {
    PageTable
}

#[derive(Clone, Copy, Debug)]
pub enum ErrorType {
    // Page table:
    // Address conversion
    VirtAddrNotAligned,
    PhysAddrNotAligned,
    // Frame allocating
    AllocFrameFailed,
    // Map / Unmap
    PageAlreadyMapped,
    PageNotMapped,
    PageNotFound,

    // Unknown
    Unknown,
}

