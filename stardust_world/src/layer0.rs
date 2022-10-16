// #[repr(C)]
#[derive(Copy, Clone)]
pub struct Layer0 {
    pub brick_indices: [u32; 16*16*16],
}

impl Layer0 {
    pub fn empty() -> Self {
        Self {
            brick_indices: [0; 16*16*16],
        }
    }
}
