use std::collections::HashMap;
use thiserror::Error;
use stardust_common::voxel::Voxel;
use stardust_common::math::*;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("unknown parsing error")]
    Unknown,
    #[error("unexpected end of file")]
    UnexpectedEOF,
}

struct Header {
    version_major: u16,
    version_minor: u16,
    brick_size: u16,
    voxel_count: u64,
}

impl Header {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let major = u16::from_le_bytes([bytes[0], bytes[1]]);
        let minor = u16::from_le_bytes([bytes[2], bytes[3]]);
        let brick_size = u16::from_le_bytes([bytes[4], bytes[5]]);
        let voxel_count = u64::from_le_bytes([bytes[6], bytes[7], bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13]]);
        Self {
            version_major: major,
            version_minor: minor,
            brick_size: brick_size,
            voxel_count: voxel_count,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let major_bytes = u16::to_le_bytes(self.version_major);
        let minor_bytes = u16::to_le_bytes(self.version_minor);
        let brick_size_bytes = u16::to_le_bytes(self.brick_size);
        let voxel_count_bytes = u64::to_le_bytes(self.voxel_count);

        let mut bytes = Vec::new();
        bytes.extend_from_slice(&major_bytes);
        bytes.extend_from_slice(&minor_bytes);
        bytes.extend_from_slice(&brick_size_bytes);
        bytes.extend_from_slice(&voxel_count_bytes);
        bytes.extend_from_slice(&[0u8, 0u8]); // Padding
        bytes
    }
}

struct Brick {
    position: [u16; 3],
    indices: Vec<u32>,
}

impl Brick {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        let x_bytes = u16::to_le_bytes(self.position[0]);
        let y_bytes = u16::to_le_bytes(self.position[1]);
        let z_bytes = u16::to_le_bytes(self.position[2]);
        bytes.extend_from_slice(&x_bytes);
        bytes.extend_from_slice(&y_bytes);
        bytes.extend_from_slice(&z_bytes);
        bytes.extend_from_slice(&[0u8, 0u8]); // Padding

        for index in &self.indices {
            let index_bytes = u32::to_le_bytes(*index);
            bytes.extend_from_slice(&index_bytes);
        }

        bytes
    }
}

pub struct Model {
    header: Header,
    bricks: Vec<Brick>,

    pub voxels: Vec<Voxel>,
}

impl Model {
    pub fn empty() -> Self {
        Self {
            header: Header {
                version_major: 0,
                version_minor: 1,
                brick_size: 8,
                voxel_count: 0,
            },
            bricks: Vec::new(),
            voxels: Vec::new(),
        }
    }

    /// Voxels with the same position are not handled properly yet.
    /// Currently, if 2 voxels have the same position, they simply overwrite eachother in the
    /// order of the list
    pub fn with_voxels(voxels: impl Iterator<Item =(UVec3, Voxel)>, brick_size: u16) -> Self {
        let mut voxels_processed = vec![Voxel::empty()];
        let mut brick_map: HashMap<UVec3, Brick> = HashMap::new();

        let mut voxels_mapped = Vec::new();
        for (wpos, voxel) in voxels {
            let mut idx = None;
            'search: for i in 0..voxels_processed.len() {
                if voxels_processed[i].0 == voxel.0 {
                    idx = Some(i as u32);
                    break 'search;
                }
            }
            if let Some(idx) = idx {
                // Voxel already exists in the voxel pool
                voxels_mapped.push((wpos, idx));
            } else {
                // Voxel doesn't yet exist in the voxel pool
                let idx = voxels_processed.len() as u32;
                voxels_processed.push(voxel);
                voxels_mapped.push((wpos, idx));
            }
        }

        for (wpos, voxel_idx) in voxels_mapped {
            let brick_pos = wpos / (brick_size as u32);
            let local_pos = wpos % (brick_size as u32);
            let local_idx = local_pos.x as usize + local_pos.y as usize * brick_size as usize + local_pos.z as usize * brick_size as usize * brick_size as usize;
            if let Some(brick) = brick_map.get_mut(&brick_pos) {
                brick.indices[local_idx] = voxel_idx;
            } else {
                let mut indices = vec![0; brick_size as usize * brick_size as usize * brick_size as usize];
                indices[local_idx] = voxel_idx;
                let brick = Brick {
                    position: [brick_pos.x as u16, brick_pos.y as u16, brick_pos.z as u16],
                    indices: indices,
                };
                brick_map.insert(brick_pos, brick);
            }
        }

        let mut bricks = Vec::new();
        for (_, brick) in brick_map {
            bricks.push(brick);
        }

        Self {
            header: Header {
                version_major: 0,
                version_minor: 1,
                brick_size: brick_size,
                voxel_count: voxels_processed.len() as u64,
            },
            bricks: bricks,
            voxels: voxels_processed,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut bytes = bytes.into_iter();
        let mut header_bytes: Vec<u8> = Vec::new();
        for _ in 0..16 {
            header_bytes.push(*bytes.next().ok_or(ParseError::UnexpectedEOF)?);
        }
        let header = Header::from_bytes(&header_bytes);
        let voxel_count = header.voxel_count;
        let mut voxels = Vec::new();
        for _ in 0..voxel_count {
            let a = *bytes.next().ok_or(ParseError::UnexpectedEOF)?;
            let b = *bytes.next().ok_or(ParseError::UnexpectedEOF)?;
            let c = *bytes.next().ok_or(ParseError::UnexpectedEOF)?;
            let d = *bytes.next().ok_or(ParseError::UnexpectedEOF)?;
            let raw = u32::from_le_bytes([a,b,c,d]);
            voxels.push(Voxel(raw));
        }

        let remaining_bytes: Vec<u8> = bytes.map(|b| *b).collect();

        let brick_byte_size = 8 + (header.brick_size as usize * header.brick_size as usize * header.brick_size as usize * 4);
        let mut bricks = Vec::new();
        for brick_bytes in remaining_bytes.chunks_exact(brick_byte_size) {
            let mut bb_iter = brick_bytes.iter();

            let xa = *bb_iter.next().ok_or(ParseError::UnexpectedEOF)?;
            let xb = *bb_iter.next().ok_or(ParseError::UnexpectedEOF)?;
            let ya = *bb_iter.next().ok_or(ParseError::UnexpectedEOF)?;
            let yb = *bb_iter.next().ok_or(ParseError::UnexpectedEOF)?;
            let za = *bb_iter.next().ok_or(ParseError::UnexpectedEOF)?;
            let zb = *bb_iter.next().ok_or(ParseError::UnexpectedEOF)?;

            let _padding_a = *bb_iter.next().ok_or(ParseError::UnexpectedEOF)?;
            let _padding_b = *bb_iter.next().ok_or(ParseError::UnexpectedEOF)?;

            let x = u16::from_le_bytes([xa,xb]);
            let y = u16::from_le_bytes([ya,yb]);
            let z = u16::from_le_bytes([za,zb]);

            let mut indices = Vec::new();
            for _ in 0..(brick_byte_size / 4) {
                let a = *bb_iter.next().ok_or(ParseError::UnexpectedEOF)?;
                let b = *bb_iter.next().ok_or(ParseError::UnexpectedEOF)?;
                let c = *bb_iter.next().ok_or(ParseError::UnexpectedEOF)?;
                let d = *bb_iter.next().ok_or(ParseError::UnexpectedEOF)?;
                let raw = u32::from_le_bytes([a,b,c,d]);
                indices.push(raw);
            }
            bricks.push(Brick {
                position: [x,y,z],
                indices
            });
        }

        Ok(Self {
            header: header,
            bricks: bricks,

            voxels: voxels,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.append(&mut self.header.to_bytes());

        for voxel in &self.voxels {
            let voxel_bytes = u32::to_le_bytes(voxel.0);
            bytes.extend_from_slice(&voxel_bytes);
        }

        for brick in &self.bricks {
            bytes.append(&mut brick.to_bytes());
        }

        bytes
    }
}
