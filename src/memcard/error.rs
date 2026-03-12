use std::error::Error;

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
    FilenameInvalidCharacters,
    FilenameIsEmpty,
    FilenameIsntNullterminated,
    StringDecodeFailed,
    IconSysBadSubtitleOffset,
    SaveIconInvalidVertexCount,
}

impl From<std::io::Error> for MemcardError {
    fn from(source: std::io::Error) -> Self {
        Self::Io(source)
    }
}

impl std::fmt::Display for MemcardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use MemcardError::*;

        match self {
            Io(source) => source.fmt(f),
            InvalidMagic => write!(
                f,
                "Invalid magic. Is this really a PS2 memory card, and is it formatted?"
            ),
            InvalidVersion => write!(f, "Unknown file system version."),
            InvalidRootdirCluster => write!(f, "Unexpected value for rootdir cluster"),
            InvalidType => write!(f, "Invalid memcard type. Should be '2'."),
            UnsupportedPageLen(v) => write!(
                f,
                "Unsupported page length '{v}'. Only standard 8MB cards are supported at the moment."
            ),
            UnsupportedPagesPerCluster(v) => write!(
                f,
                "Unsupported pages per cluster '{v}'. Only standard 8MB cards are supported at the moment."
            ),
            UnsupportedPagesPerBlock(v) => write!(
                f,
                "Unsupported pages per block '{v}'. Only standard 8MB cards are supported at the moment."
            ),
            UnsupportedClustersTotal(v) => write!(
                f,
                "Unsupported memcard size '{v}' clusters. Only standard 8MB cards are supported at the moment."
            ),
            Ecc => write!(f, "Bad ECC. Is the card corrupted?"),
            FreeClusterAllocated => write!(f, "Entry contains a cluster that is marked free"),
            FilenameInvalidCharacters => write!(f, "Filename contains invalid characters."),
            FilenameIsEmpty => write!(f, "Filename is empty."),
            FilenameIsntNullterminated => write!(f, "Filename isn't null-terminated."),
            StringDecodeFailed => write!(f, "Couldn't decode string."),
            IconSysBadSubtitleOffset => write!(f, "icon.sys subtitle offset is bad."),
            SaveIconInvalidVertexCount => {
                write!(f, "Save icon vertex count isn't a multiple of 3.")
            }
        }
    }
}

impl Error for MemcardError {}
