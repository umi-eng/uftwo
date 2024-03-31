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

    pub fn has_extensions(&self) -> bool {
        self.flags.contains(Flags::ExtensionTags)
    }

    pub fn extensions(&self) -> Option<Extensions> {
        if self.has_extensions() {
            let start = self.payload_size as usize;
            let start = start.next_multiple_of(Extensions::ALIGN);
            let end = self.data.len();
            Some(Extensions::new(&self.data[start..end]))
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
#[cfg_attr(defmt, defmt::Format)]
pub struct Extensions<'a> {
    start: usize,
    data: &'a [u8],
}

impl<'a> Extensions<'a> {
    /// Length byte + tag bytes.
    const HEADER_SIZE: usize = 4;

    /// Align to 4 byte boundary
    const ALIGN: usize = 4;

    pub fn new(data: &'a [u8]) -> Self {
        Self { start: 0, data }
    }

    fn current_tag(&self) -> ExtensionTag {
        let tag = u32::from_le_bytes([
            self.data[self.start + 1],
            self.data[self.start + 2],
            self.data[self.start + 3],
            0,
        ]);
        ExtensionTag::from(tag)
    }
}

impl<'a> Iterator for Extensions<'a> {
    type Item = Extension<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start > self.data.len() {
            // we are at the end
            return None;
        }

        let len = self.data[self.start] as usize;

        if self.start + Self::HEADER_SIZE > self.start + len {
            // there is no more tags
            return None;
        }

        let extension = Extension {
            tag: self.current_tag(),
            data: &self.data[self.start + Self::HEADER_SIZE..self.start + len],
        };

        // incerment start point
        // i.e where does the next (potential) tag start
        self.start += len;
        self.start = self.start.next_multiple_of(Self::ALIGN);

        Some(extension)
    }
}

/// An additional piece of information which can be appended after payload
/// data.
///
/// Converting the extension tag to UTF-8 strings or otherwise is an exercise
/// left to the user.
#[derive(Debug)]
#[cfg_attr(defmt, defmt::Format)]
pub struct Extension<'a> {
    pub tag: ExtensionTag,
    pub data: &'a [u8],
}

#[derive(Debug, PartialEq, Eq)]
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
            _ => Self::Unknown(value), // still valid, just unknown to us
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

    #[test]
    fn block_checksum() {
        let mut block = Block::default();
        assert_eq!(block.has_checksum(), false);

        block.flags |= Flags::Checksum;
        assert_eq!(block.has_checksum(), true);

        let cksm = block.checksum();
        assert!(cksm.is_some());
    }

    #[test]
    fn block_extension() {
        let mut block = Block {
            flags: Flags::ExtensionTags,
            payload_size: 0,
            ..Default::default()
        };

        // Semver string
        block.data[0..12].copy_from_slice(&[
            0x09, 0xbc, 0xc7, 0x9f, 0x30, 0x2e, 0x31, 0x2e, 0x32, 0x00, 0x00,
            0x00,
        ]);
        // Semver string
        block.data[12..24].copy_from_slice(&[
            0x09, 0xbc, 0xc7, 0x9f, 0x30, 0x2e, 0x31, 0x2e, 0x32, 0x00, 0x00,
            0x00,
        ]);
        // Device description
        block.data[24..44].copy_from_slice(&[
            0x14, 0x9d, 0x0d, 0x65, 0x41, 0x43, 0x4d, 0x45, 0x20, 0x54, 0x6f,
            0x61, 0x73, 0x74, 0x65, 0x72, 0x20, 0x6d, 0x6b, 0x33,
        ]);

        assert!(block.extensions().is_some());

        let mut extensions = block.extensions().unwrap();

        let first = extensions.next().unwrap();
        assert_eq!(first.tag, ExtensionTag::SemverString);
        assert_eq!(first.data, b"0.1.2");

        let second = extensions.next().unwrap();
        assert_eq!(second.tag, ExtensionTag::SemverString);
        assert_eq!(second.data, b"0.1.2");

        let third = extensions.next().unwrap();
        assert_eq!(third.tag, ExtensionTag::DescriptionString);
        assert_eq!(third.data, b"ACME Toaster mk3");
    }
}
