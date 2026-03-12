use std::io::Read;

use bytemuck::Pod;
use bytemuck::Zeroable;

use crate::memcard::MemcardError;
use crate::memcard::save_icon;
use crate::util::*;

#[derive(Debug, Default, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct VertexCoord {
    pub x: i16,
    pub y: i16,
    pub z: i16,
    pub w: i16,
}

#[derive(Debug, Default, Clone)]
#[repr(C)]
pub struct Vertex {
    pub coords: Vec<VertexCoord>,
    pub normal: VertexCoord,
    pub u: i16,
    pub v: i16,
    pub rgba: [u8; 4],
}

/// PlayStation 2 save icon
#[derive(Debug, Clone)]
#[repr(C)]
pub struct SaveIcon {
    pub magic: u32,
    pub num_anim_shapes: u32,
    pub tex_flags: u32,
    pub unk_0xc: u32,
    pub num_vertices: u32,
    pub vertices: Vec<Vertex>,
}

impl SaveIcon {
    pub const MAGIC: u32 = 0x010000;

    pub fn read<R: Read>(reader: &mut R) -> Result<Self, MemcardError> {
        let magic = read_u32(reader)?;
        let num_anim_shapes = read_u32(reader)?;
        let tex_flags = read_u32(reader)?;
        let unk_0xc = read_u32(reader)?;
        let num_vertices = read_u32(reader)?;

        let mut save_icon = Self {
            magic,
            num_anim_shapes,
            tex_flags,
            unk_0xc,
            num_vertices,
            vertices: Vec::with_capacity(num_vertices as usize),
        };

        for _ in 0..num_vertices {
            let mut coords: Vec<VertexCoord> = Vec::with_capacity(num_anim_shapes as usize);
            for _ in 0..num_anim_shapes {
                let mut bytes = [0_u8; size_of::<VertexCoord>()];
                reader.read_exact(&mut bytes)?;
                coords.push(bytemuck::cast(bytes));
            }
            let mut bytes = [0_u8; size_of::<VertexCoord>()];
            reader.read_exact(&mut bytes)?;
            let normal: VertexCoord = bytemuck::cast(bytes);
            let u = read_i16(reader)?;
            let v = read_i16(reader)?;
            let mut rgba = [0_u8; 4];
            reader.read_exact(&mut rgba)?;
            save_icon.vertices.push(Vertex {
                coords,
                normal,
                u,
                v,
                rgba,
            });
        }

        Ok(save_icon)
    }

    pub fn validate(&self) -> Result<(), MemcardError> {
        if self.magic != Self::MAGIC {
            return Err(MemcardError::InvalidMagic);
        }

        if self.num_vertices % 3 != 0 {
            return Err(MemcardError::SaveIconInvalidVertexCount);
        }

        Ok(())
    }
}
