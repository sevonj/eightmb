use std::io::Read;

use crate::memcard::Entry;
use crate::memcard::MemcardError;
use crate::memcard::Timestamp;

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
    pub fn entry_by_name(&self, filename: &str) -> Option<&Entry> {
        match filename {
            "." => return Some(&self.dot),
            ".." => return Some(&self.dotdot),
            _ => {
                for entry in &self.entries {
                    if filename == &entry.name() {
                        return Some(entry);
                    }
                }
            }
        }
        None
    }

    pub fn last_modified(&self) -> Timestamp {
        let mut last = self.dot.modified.max(self.dotdot.modified);
        for entry in &self.entries {
            if entry.modified > last {
                last = entry.modified;
            }
        }
        last
    }

    pub fn total_size(&self) -> u32 {
        self.entries
            .iter()
            .filter(|e| e.is_file())
            .map(|e| e.len)
            .sum()
    }

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
