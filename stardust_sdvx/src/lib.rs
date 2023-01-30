use thiserror::Error;
use stardust_common::voxel::Voxel;

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
}

struct Brick {
    indices: Vec<u32>,
}

pub struct Model {
    header: Header,
    bricks: Vec<Brick>,

    pub voxels: Vec<Voxel>,
}

impl Model {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ParseError> {
        let bytes = bytes.into_iter();
        let mut header_bytes: Vec<u8> = Vec::new();
        for _ in 0..16 {
            header_bytes.push(*bytes.next().ok_or(ParseError::UnexpectedEOF)?);
        }
        let header = Header::from_bytes(&header_bytes);
        let voxel_count = header.voxel_count;
        let mut voxels = Vec::new();
        for i in 0..voxel_count {
            let a = *bytes.next().ok_or(ParseError::UnexpectedEOF)?;
            let b = *bytes.next().ok_or(ParseError::UnexpectedEOF)?;
            let c = *bytes.next().ok_or(ParseError::UnexpectedEOF)?;
            let d = *bytes.next().ok_or(ParseError::UnexpectedEOF)?;
            let raw = u32::from_le_bytes([a,b,c,d]);
            voxels.push(Voxel(raw));
        }

        Ok(Self {
            header: header,
            bricks: bricks,

            voxels: voxels,
        })
    }
}
