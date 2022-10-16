#[derive(Copy, Clone)]
pub struct UsageFlags(u8);

impl UsageFlags {
    pub fn empty() -> Self { Self(0) }

    fn set_bit(&mut self, bit: usize, val: bool) {
        if val { self.0 |= 1<<bit; } else { self.0 &= !(1<<bit); }
    }

    fn get_bit(&self, bit: usize) -> bool {
        ((self.0 >> bit) & 1) > 0
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.set_bit(0, dirty);
    }

    pub fn dirty(&self) -> bool {
        self.get_bit(0)
    }

    pub fn set_in_use(&mut self, in_use: bool) {
        self.set_bit(1, in_use);
    }

    pub fn in_use(&self) -> bool {
        self.get_bit(1)
    }
}
