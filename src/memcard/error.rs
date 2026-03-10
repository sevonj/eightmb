#[derive(Debug)]
pub enum MemcardError {
    Io(std::io::Error),
    InvalidMagic,
    InvalidVersion,
    InvalidRootdirCluster,
    InvalidType,
    UnsupportedPageLen(u16),
    UnsupportedPagesPerCluster(u16),
    UnsupportedPagesPerBlock(u16),
    UnsupportedClustersTotal(u32),
    Ecc,
    FreeClusterAllocated,
}

impl From<std::io::Error> for MemcardError {
    fn from(source: std::io::Error) -> Self {
        Self::Io(source)
    }
}
