use std::io::Read;

use crate::memcard::MemcardError;
use crate::util::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Vec4 {
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub w: u32,
}

impl Vec4 {
    pub fn read<R: Read>(reader: &mut R) -> Result<Self, MemcardError> {
        Ok(Self {
            x: read_u32(reader)?,
            y: read_u32(reader)?,
            z: read_u32(reader)?,
            w: read_u32(reader)?,
        })
    }
}

/// PlayStation 2 save info file
/// Color value range: 0x00..=0xff
#[derive(Debug, Clone)]
#[repr(C)]
pub struct IconSys {
    pub magic: [u8; 4],
    pub unk_0x04: u16,
    /// Title is split into two rows at this offset
    pub subtitle_off: u16,
    pub unk_0x08: u32,
    pub bg_opacity: u32,
    /// Top left RGB_
    pub bg_color_a: Vec4,
    /// Top right RGB_
    pub bg_color_b: Vec4,
    /// Bottom left RGB_
    pub bg_color_c: Vec4,
    /// Bottom right RGB_
    pub bg_color_d: Vec4,
    pub light_a_dir: Vec4,
    pub light_b_dir: Vec4,
    pub light_c_dir: Vec4,
    /// RGB_
    pub light_a_color: Vec4,
    /// RGB_
    pub light_b_color: Vec4,
    /// RGB_
    pub light_c_color: Vec4,
    /// RGB_
    pub light_ambient_color: Vec4,
    /// Encoding: Shift-JIS
    pub title: [u8; 68],
    /// Required, icon for normal view. Encoding: Ascii filename
    pub list_icon: [u8; 64],
    /// Optional, falls back to list icon. Encoding: Ascii filename
    pub copy_icon: [u8; 64],
    /// Optional, falls back to list icon. Encoding: Ascii filename
    pub delete_icon: [u8; 64],
}

impl IconSys {
    pub const MAGIC: &[u8; 4] = b"PS2D";

    pub fn title(&self) -> String {
        let subtitle_off = self.subtitle_off as usize;
        shiftjis_to_string(&self.title[..subtitle_off]).unwrap()
    }

    pub fn subtitle(&self) -> String {
        let subtitle_off = self.subtitle_off as usize;
        shiftjis_to_string(&self.title[subtitle_off..]).unwrap()
    }

    pub fn list_icon(&self) -> String {
        filename_to_string(self.list_icon.as_slice()).unwrap()
    }

    pub fn copy_icon(&self) -> Option<String> {
        if self.copy_icon[0] == 0 {
            return None;
        }
        Some(filename_to_string(self.copy_icon.as_slice()).unwrap())
    }

    pub fn delete_icon(&self) -> Option<String> {
        if self.delete_icon[0] == 0 {
            return None;
        }
        Some(filename_to_string(self.delete_icon.as_slice()).unwrap())
    }

    pub fn read<R: Read>(reader: &mut R) -> Result<Self, MemcardError> {
        let mut magic = [0; 4];
        reader.read_exact(&mut magic)?;
        let unk_0x04 = read_u16(reader)?;
        let unk_0x06 = read_u16(reader)?;
        let unk_0x08 = read_u32(reader)?;
        let bg_opacity = read_u32(reader)?;
        let bg_color_a = Vec4::read(reader)?;
        let bg_color_b = Vec4::read(reader)?;
        let bg_color_c = Vec4::read(reader)?;
        let bg_color_d = Vec4::read(reader)?;
        let light_a_dir = Vec4::read(reader)?;
        let light_b_dir = Vec4::read(reader)?;
        let light_c_dir = Vec4::read(reader)?;
        let light_a_color = Vec4::read(reader)?;
        let light_b_color = Vec4::read(reader)?;
        let light_c_color = Vec4::read(reader)?;
        let light_ambient_color = Vec4::read(reader)?;
        let mut title = [0; 68];
        reader.read_exact(&mut title)?;
        let mut icon_list = [0; 64];
        reader.read_exact(&mut icon_list)?;
        let mut icon_copy = [0; 64];
        reader.read_exact(&mut icon_copy)?;
        let mut icon_delete = [0; 64];
        reader.read_exact(&mut icon_delete)?;

        let iconsys = Self {
            magic,
            unk_0x04,
            subtitle_off: unk_0x06,
            unk_0x08,
            bg_opacity,
            bg_color_a,
            bg_color_b,
            bg_color_c,
            bg_color_d,
            light_a_dir,
            light_b_dir,
            light_c_dir,
            light_a_color,
            light_b_color,
            light_c_color,
            light_ambient_color,
            title,
            list_icon: icon_list,
            copy_icon: icon_copy,
            delete_icon: icon_delete,
        };

        iconsys.validate()?;

        Ok(iconsys)
    }

    pub fn validate(&self) -> Result<(), MemcardError> {
        if &self.magic != Self::MAGIC {
            return Err(MemcardError::InvalidMagic);
        }

        let subtitle_off = self.subtitle_off as usize;
        if subtitle_off > self.title.len() {
            return Err(MemcardError::IconSysBadSubtitleOffset);
        }
        validate_shiftjis(self.title.as_slice())?;
        validate_shiftjis(&self.title[..subtitle_off])?;
        validate_shiftjis(&self.title[subtitle_off..])?;

        validate_filename(self.list_icon.as_slice())?;

        if self.copy_icon[0] != 0 {
            validate_filename(self.copy_icon.as_slice())?;
        }

        if self.delete_icon[0] != 0 {
            validate_filename(self.delete_icon.as_slice())?;
        }

        Ok(())
    }
}
