use std::io::Read;

use crate::memcard::Entry;
use crate::memcard::MemcardError;

#[derive(Debug, Default, Clone)]
#[repr(C)]
pub struct Directory {
    // The first entry "."
    pub dot: Entry,
    // The second entry "..". Unused?
    pub dotdot: Entry,
    pub entries: Vec<Entry>,
}

impl Directory {
    pub fn read<R: Read>(reader: &mut R, entry: &Entry) -> Result<Self, MemcardError> {
        let dot = Entry::read(reader)?;
        let dotdot = Entry::read(reader)?;
        assert!(dot.is_dir());
        assert!(dotdot.is_dir());

        let num_entries = entry.len as usize - 2;
        let mut entries = Vec::with_capacity(num_entries);
        for _ in 0..num_entries {
            entries.push(Entry::read(reader)?);
        }

        Ok(Self {
            dot,
            dotdot,
            entries,
        })
    }


    pub fn read_root<R: Read>(reader: &mut R) -> Result<Self, MemcardError> {
        let dot = Entry::read(reader)?;
        let dotdot = Entry::read(reader)?;
        assert!(dot.is_dir());
        assert!(dotdot.is_dir());

        let num_entries = dot.len as usize - 2;
        let mut entries = Vec::with_capacity(num_entries);
        for _ in 0..num_entries {
            entries.push(Entry::read(reader)?);
        }

        Ok(Self {
            dot,
            dotdot,
            entries,
        })
    }
}
