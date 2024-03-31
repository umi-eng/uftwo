#![no_std]

use zerocopy::{AsBytes, FromBytes, FromZeroes};

/// Magic numbers.
pub const MAGIC_NUMBER: [u32; 3] = [0x0A324655, 0x9E5D5157, 0x0AB16F30];

/// Block structure.
///
/// Length is fixed at 512 bytes with a variable size data section up to 476 bytes.
#[derive(Debug, AsBytes, FromBytes, FromZeroes)]
#[repr(C)]
#[cfg_attr(defmt, defmt::Format)]
pub struct Block {
    /// First magic number.
    magic_start_0: u32,
    /// Second magic number.
    magic_start_1: u32,
    /// Flags.
    pub flags: Flags,
    /// Address in flash where the data should be written.
    pub target_addr: u32,
    /// Number of bytes used in data.
    pub payload_size: u32,
    //// Sequential block number, starting at 0.
    pub block_no: u32,
    /// Total number of blocks.
    pub num_blocks: u32,
    /// File size or board family ID or zero.
    pub file_size_board_family: u32,
    /// Payload data, padded with zeros.
    ///
    /// When the MD5 checksum flag is set, the last 24 bytes hold the checksum
    /// as well as address start and length.
    pub data: [u8; 476],
    /// Final magic number.
    magic_end: u32,
}

const _: () = {
    // Ensure block is correct size.
    assert!(core::mem::size_of::<Block>() == 512);
};

impl Default for Block {
    fn default() -> Self {
        Self {
            magic_start_0: MAGIC_NUMBER[0],
            magic_start_1: MAGIC_NUMBER[1],
            flags: Flags::default(),
            target_addr: 0,
            payload_size: 0,
            block_no: 0,
            num_blocks: 0,
            file_size_board_family: 0,
            data: [0; 476],
            magic_end: MAGIC_NUMBER[2],
        }
    }
}

impl Block {
    /// Returns if the checksum flag is set.
    pub fn has_checksum(&self) -> bool {
        self.flags.contains(Flags::Checksum)
    }

    /// Returns the checksum value only if the checksum flag is set.
    pub fn checksum(&self) -> Option<&Checksum> {
        if self.has_checksum() {
            let len = self.data.len();
            Checksum::ref_from(&self.data[len - 24..len])
        } else {
            None
        }
    }
}

/// Checksum information.
///
/// This allows skipping writing data that is the same.
#[derive(Debug, AsBytes, FromBytes, FromZeroes)]
#[repr(C)]
#[cfg_attr(defmt, defmt::Format)]
pub struct Checksum {
    start: u32,
    length: u32,
    checksum: [u8; 16],
}

const _: () = {
    // Ensure Checksum is correct size.
    assert!(core::mem::size_of::<Checksum>() == 24);
};

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, AsBytes, FromBytes, FromZeroes,
)]
#[repr(C)]
#[cfg_attr(defmt, defmt::Format)]
pub struct Flags(u32);

bitflags::bitflags! {
    impl Flags: u32 {
        const NotMainFlash = 0x00000001;
        const FileContainer = 0x00001000;
        const FamilyId = 0x00002000;
        const Checksum = 0x00004000;
        const ExtensionTags = 0x00008000;
        const _ = !0;
    }
}

#[derive(Debug)]
#[repr(u32)]
#[cfg_attr(defmt, defmt::Format)]
pub enum ExtensionTag {
    /// UTF-8 Semantic Versioning string.
    SemverString = 0x9fc7bc,
    /// UTF-8 device description.
    DescriptionString = 0x650d9d,
    /// Page size of target device.
    TagetPageSize = 0x0be9f7,
    /// SHA-2 checksum of the firmware.
    Sha2Checksum = 0xb46db0,
    /// Device type identifier.
    DeviceTypeId = 0xc8a729,
    /// Unknown tag.
    Unknown(u32),
}

impl From<u32> for ExtensionTag {
    fn from(value: u32) -> Self {
        match value {
            0x9fc7bc => Self::SemverString,
            0x650d9d => Self::DescriptionString,
            0x0be9f7 => Self::TagetPageSize,
            0xb46db0 => Self::Sha2Checksum,
            0xc8a729 => Self::DeviceTypeId,
            _ => Self::Unknown(value), // not
        }
    }
}

#[cfg(test)]
mod tests {
    use zerocopy::AsBytes;

    use super::*;

    #[test]
    fn magic_number() {
        assert_eq!(MAGIC_NUMBER[0].as_bytes(), b"UF2\n");
    }
}
