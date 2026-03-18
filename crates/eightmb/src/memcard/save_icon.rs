use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

use bytemuck::Pod;
use bytemuck::Zeroable;

use crate::memcard::MemcardError;
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
    pub texture: Box<[u32; 128 * 128]>,
}

impl SaveIcon {
    pub const MAGIC: u32 = 0x010000;

    pub fn read<R: Read>(reader: &mut R) -> Result<Self, MemcardError> {
        let magic = read_u32(reader)?;
        let num_anim_shapes = read_u32(reader)?;
        let tex_type = read_u32(reader)?;
        let unk_0xc = read_u32(reader)?;
        let num_vertices = read_u32(reader)?;

        let mut save_icon = Self {
            magic,
            num_anim_shapes,
            tex_flags: tex_type,
            unk_0xc,
            num_vertices,
            vertices: Vec::with_capacity(num_vertices as usize),
            // default to black
            texture: Box::new([0xff000000; 128 * 128]),
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

        fn unpack_a1b5g5r5<R: Read>(
            reader: &mut R,
            save_icon: &mut SaveIcon,
        ) -> Result<(), MemcardError> {
            for px in save_icon.texture.as_mut() {
                let mut buf = [0_u8; 2];
                reader.read_exact(&mut buf)?;
                let packed = u16::from_le_bytes(buf);
                let r = (packed << 3) as u8;
                let g = (packed >> 2) as u8 & 0b11111000;
                let b = (packed >> 7) as u8 & 0b11111000;
                *px = u32::from_le_bytes([r, g, b, 0xff]);
            }
            Ok(())
        }

        println!("tex_flags: {tex_type:02X?}");

        // Values encountered so far:
        // - 0x0F           (Burnout 3, Ratchet 2, Armored Core 2)
        // - 0x07 ?1B5G5R5
        // - 0x06 ?1B5G5R5  (Dog's Life)
        // - 0x03 NONE      (ICO, Sly 2, Sly 3)
        match tex_type {
            0x07 | 0x06 => unpack_a1b5g5r5(reader, &mut save_icon)?,
            0x03 => (),
            v => println!("unknown texture type {v:X?}"),
        }

        /*{
            println!("compressed");
            let compressed_len = read_u32(reader)? as usize;
            let mut data = Vec::with_capacity(128 * 128 * 2);

            let mut read_off = 0;
            while read_off < compressed_len {
                let rle = read_i16(reader)?;
                let repeat = rle < 0;
                println!("  rle: {rle}");

                if repeat {
                    let times = 0x8000 + rle as usize;
                    println!("  rep: {times}");
                    let mut buf = vec![0; 2];
                    reader.read_exact(&mut buf)?;
                    for _ in 0..times {
                        data.extend_from_slice(&buf);
                    }
                    read_off += 4;
                } else {
                    let bytes = rle as usize * 2;
                    println!("norep: {bytes}");
                    let mut buf = vec![0; bytes];
                    reader.read_exact(&mut buf)?;
                    data.append(&mut buf);
                    read_off += bytes + 2;
                }
            }
            unpack_a1b5g5r5(&mut BufReader::new(data.as_slice()), &mut save_icon)?;
        }*/

        Ok(save_icon)
    }

    pub fn validate(&self) -> Result<(), MemcardError> {
        if self.magic != Self::MAGIC {
            return Err(MemcardError::InvalidMagic);
        }

        if !self.num_vertices.is_multiple_of(3) {
            return Err(MemcardError::SaveIconInvalidVertexCount);
        }

        Ok(())
    }

    pub fn debug_dump_wavefront(&self, filename: &str) {
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let temp_dir = workspace_root.join("temp");
        let out_path = temp_dir.join(filename).with_added_extension("obj");

        let mut wavefront = String::new();

        wavefront += "# ";
        wavefront += filename;

        for v in &self.vertices {
            let x = v.coords[0].x as f32 / 0x1000 as f32;
            let y = v.coords[0].y as f32 / 0x1000 as f32;
            let z = v.coords[0].z as f32 / 0x1000 as f32;
            wavefront += &format!("\nv {} {} {}", x, y, z)
        }

        for i in 0..(self.vertices.len() / 3) {
            let o = i * 3;
            wavefront += &format!("\nf {} {} {}", o + 1, o + 2, o + 3);
        }

        let mut f = File::create(out_path).unwrap();
        f.write_all(wavefront.as_bytes()).unwrap();
    }
}
