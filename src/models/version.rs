#[derive(Debug, PartialEq, Eq, Default)]
pub struct Version(pub [u8; Version::SIZE]);

impl Version {
    pub const SIZE: usize = 3;
}
