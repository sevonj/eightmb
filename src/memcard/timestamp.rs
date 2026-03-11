use std::cmp::Ordering;
use std::io::Read;

use crate::memcard::MemcardError;
use crate::util::*;

/// Serialized bytes: _smhDMYY
/// Always japanese time zone
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub struct Timestamp {
    pub sec: u8,
    pub min: u8,
    pub hour: u8,
    pub day: u8,
    pub mon: u8,
    pub year: u16,
}

impl Timestamp {
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, MemcardError> {
        let mut timestamp = Self::default();
        reader.read_exact(&mut [0])?;
        timestamp.sec = read_u8(reader)?;
        timestamp.min = read_u8(reader)?;
        timestamp.hour = read_u8(reader)?;
        timestamp.day = read_u8(reader)?;
        timestamp.mon = read_u8(reader)?;
        timestamp.year = read_u16(reader)?;
        Ok(timestamp)
    }
}

impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sec = self.sec;
        let min = self.min;
        let hour = self.hour;
        let day = self.day;
        let mon = self.mon;
        let year = self.year;

        // RFC 3339-ish representation
        write!(f, "{year:04}-{mon:02}-{day:02} {hour:02}:{min:02}:{sec:02}")
    }
}

impl Ord for Timestamp {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.year.cmp(&other.year) {
            Ordering::Equal => (),
            ord => return ord,
        }
        match self.mon.cmp(&other.mon) {
            Ordering::Equal => (),
            ord => return ord,
        }
        match self.day.cmp(&other.day) {
            Ordering::Equal => (),
            ord => return ord,
        }
        match self.hour.cmp(&other.hour) {
            Ordering::Equal => (),
            ord => return ord,
        }
        match self.min.cmp(&other.min) {
            Ordering::Equal => (),
            ord => return ord,
        }
        self.sec.cmp(&other.sec)
    }
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use super::*;

    #[test]
    fn test_sizeof_timestamp() {
        assert_eq!(size_of::<Timestamp>(), 8)
    }

    #[test]
    fn test_readsize_timestamp() {
        let mut eight = BufReader::new([0; 8].as_slice());
        assert!(Timestamp::read(&mut eight).is_ok());
        assert!(eight.read_exact(&mut [0]).is_err());
    }
}
