#[derive(Debug, PartialEq, Eq, Default)]
pub struct MagicBytes(pub [u8; MagicBytes::SIZE]);

impl MagicBytes {
    pub const SIZE: usize = 4;
}