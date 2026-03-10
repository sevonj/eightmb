use std::io::Read;

use crate::memcard::MemcardError;
use crate::memcard::timestamp::Timestamp;
use crate::util::*;

#[derive(Debug, Default, Clone)]
#[repr(C)]
pub struct Entry {
    pub mode: u16,
    pub len: u32,
    pub created: Timestamp,
    pub cluster: u32,
    pub dir_entry: u32,
    pub modified: Timestamp,
    pub attr: u32,
    pub name: [u8; 32],
}

impl Entry {
    pub const FLAG_READ: u16 = 0x0001;
    pub const FLAG_WRITE: u16 = 0x0002;
    pub const FLAG_EXECUTE: u16 = 0x0004;
    pub const FLAG_PROTECTED: u16 = 0x0008;
    pub const FLAG_FILE: u16 = 0x0010;
    pub const FLAG_DIRECTORY: u16 = 0x0020;
    pub const FLAG_DIR_INTERNAL: u16 = 0x0040;
    pub const FLAG_COPIED_MAYBE: u16 = 0x0080;
    pub const FLAG_0X0100: u16 = 0x0100;
    pub const FLAG_CREATE: u16 = 0x0200;
    pub const FLAG_0X0400: u16 = 0x0400;
    pub const FLAG_POCKETSTN: u16 = 0x0800;
    pub const FLAG_PSX: u16 = 0x1000;
    pub const FLAG_HIDDEN: u16 = 0x2000;
    pub const FLAG_0X4000: u16 = 0x4000;
    pub const FLAG_EXISTS: u16 = 0x8000;

    pub fn read<R: Read>(reader: &mut R) -> Result<Self, MemcardError> {
        let mut entry = Self::default();
        entry.mode = read_u16(reader)?;
        reader.read_exact(&mut [0; 2])?; // align
        entry.len = read_u32(reader)?;
        entry.created = Timestamp::read(reader)?;
        entry.cluster = read_u32(reader)?;
        entry.dir_entry = read_u32(reader)?;
        entry.modified = Timestamp::read(reader)?;
        entry.attr = read_u32(reader)?;
        reader.read_exact(&mut [0; 0x1c])?; // align
        reader.read_exact(&mut entry.name)?;
        reader.read_exact(&mut [0; 0x1a0])?; // unused space?
        Ok(entry)
    }

    pub fn can_read(&self) -> bool {
        self.mode & Self::FLAG_READ != 0
    }

    pub fn can_write(&self) -> bool {
        self.mode & Self::FLAG_WRITE != 0
    }

    pub fn can_execute(&self) -> bool {
        self.mode & Self::FLAG_EXECUTE != 0
    }

    pub fn is_protected(&self) -> bool {
        self.mode & Self::FLAG_PROTECTED != 0
    }

    pub fn is_file(&self) -> bool {
        self.mode & Self::FLAG_FILE != 0
    }

    pub fn is_dir(&self) -> bool {
        self.mode & Self::FLAG_DIRECTORY != 0
    }

    pub fn flag_dir_internal(&self) -> bool {
        self.mode & Self::FLAG_DIR_INTERNAL != 0
    }

    pub fn is_copied_maybe(&self) -> bool {
        self.mode & Self::FLAG_COPIED_MAYBE != 0
    }

    pub fn flag_0x0100(&self) -> bool {
        self.mode & Self::FLAG_0X0100 != 0
    }

    pub fn flag_create(&self) -> bool {
        self.mode & Self::FLAG_CREATE != 0
    }

    pub fn flag_0x0400(&self) -> bool {
        self.mode & Self::FLAG_0X0400 != 0
    }

    pub fn is_pocketstn_save(&self) -> bool {
        self.mode & Self::FLAG_POCKETSTN != 0
    }

    pub fn is_psx_save(&self) -> bool {
        self.mode & Self::FLAG_PSX != 0
    }

    pub fn is_hidden(&self) -> bool {
        self.mode & Self::FLAG_HIDDEN != 0
    }

    pub fn flag_0x4000(&self) -> bool {
        self.mode & Self::FLAG_0X4000 != 0
    }

    pub fn exists(&self) -> bool {
        self.mode & Self::FLAG_EXISTS != 0
    }

    pub fn name(&self) -> String {
        String::from_utf8_lossy(&self.name)
            .trim_end_matches('\0')
            .to_owned()
    }
}

impl std::fmt::Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.name();

        let mode = format!(
            "{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}",
            if self.can_read() { 'r' } else { '-' },
            if self.can_write() { 'w' } else { '-' },
            if self.can_execute() { 'x' } else { '-' },
            if self.is_protected() { 'p' } else { '-' },
            if self.is_file() { 'f' } else { '-' },
            if self.is_dir() { 'd' } else { '-' },
            if self.flag_dir_internal() { 'i' } else { '-' },
            if self.is_copied_maybe() { 'c' } else { '-' },
            if self.flag_0x0100() { '?' } else { '-' },
            if self.flag_create() { 'C' } else { '-' },
            if self.flag_0x0400() { '?' } else { '-' },
            if self.is_pocketstn_save() { 'P' } else { '-' },
            if self.is_psx_save() { 'X' } else { '-' },
            if self.is_hidden() { 'h' } else { '-' },
            if self.flag_0x4000() { '?' } else { '-' },
            if self.exists() { 'e' } else { '-' },
        );

        write!(
            f,
            "'{name}'
    type:       {}
    len:        {}
    created:    {}
    modified:   {}
    cluster:    {}
    dir_entry:  {}
    mode:       {mode}
    attr:       {}",
            if self.is_dir() { "dir" } else { "file" },
            self.len,
            self.created,
            self.modified,
            self.cluster,
            self.dir_entry,
            self.attr,
        )
    }
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use super::*;

    #[test]
    fn test_entry_timestamp() {
        let mut fiveonetwo = BufReader::new([0; 512].as_slice());
        assert!(Entry::read(&mut fiveonetwo).is_ok());
        assert!(fiveonetwo.read_exact(&mut [0]).is_err());
    }
}
