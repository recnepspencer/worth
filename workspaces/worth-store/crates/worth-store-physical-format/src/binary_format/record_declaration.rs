#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum PhysicalRecordFormatVersion {
    V2 = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PhysicalRecordByteOrder {
    LittleEndian = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PhysicalRecordRootProtocol {
    StagedCatalogV1 = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PhysicalRecordIntegrity {
    Crc32c = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalRecordFormatDeclaration {
    version: PhysicalRecordFormatVersion,
    page_size: PhysicalPageSizeClass,
    byte_order: PhysicalRecordByteOrder,
    root_protocol: PhysicalRecordRootProtocol,
    integrity: PhysicalRecordIntegrity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecordFormatDenial {
    UnsupportedVersion(u16),
    UnsupportedPageBytes(u32),
    UnsupportedByteOrder(u8),
    UnsupportedRootProtocol(u8),
    UnsupportedIntegrity(u8),
    UnsupportedRecordIdentityBytes(u8),
}

#[derive(Debug, Default)]
pub struct PhysicalRecordFormatDeclarationBuilder {
    version: Option<PhysicalRecordFormatVersion>,
    page_size: Option<PhysicalPageSizeClass>,
}

impl PhysicalRecordFormatDeclaration {
    pub fn builder() -> PhysicalRecordFormatDeclarationBuilder {
        PhysicalRecordFormatDeclarationBuilder::default()
    }

    pub const fn version(self) -> PhysicalRecordFormatVersion {
        self.version
    }

    pub const fn page_size(self) -> PhysicalPageSizeClass {
        self.page_size
    }

    pub(crate) const fn page_bytes(self) -> u32 {
        self.page_size.bytes()
    }

    pub const fn byte_order(self) -> PhysicalRecordByteOrder {
        self.byte_order
    }

    pub const fn root_protocol(self) -> PhysicalRecordRootProtocol {
        self.root_protocol
    }

    pub const fn integrity(self) -> PhysicalRecordIntegrity {
        self.integrity
    }

    /// Returns the complete canonical persisted identity of this format.
    ///
    /// This is representation, not admission authority. Consumers must still
    /// compare it with a format admitted by the current Store owner.
    pub const fn canonical_identity_bytes(self) -> [u8; 10] {
        self.encode()
    }

    pub(crate) fn decode(bytes: [u8; 10]) -> Result<Self, PhysicalRecordFormatDenial> {
        let version = u16::from_le_bytes([bytes[0], bytes[1]]);
        if version != PhysicalRecordFormatVersion::V2 as u16 {
            return Err(PhysicalRecordFormatDenial::UnsupportedVersion(version));
        }
        let page_bytes = u32::from_le_bytes(bytes[2..6].try_into().expect("fixed field"));
        let page_size = decode_page_size(page_bytes)?;
        if bytes[6] != PhysicalRecordByteOrder::LittleEndian as u8 {
            return Err(PhysicalRecordFormatDenial::UnsupportedByteOrder(bytes[6]));
        }
        if bytes[7] != PhysicalRecordRootProtocol::StagedCatalogV1 as u8 {
            return Err(PhysicalRecordFormatDenial::UnsupportedRootProtocol(
                bytes[7],
            ));
        }
        if bytes[8] != PhysicalRecordIntegrity::Crc32c as u8 {
            return Err(PhysicalRecordFormatDenial::UnsupportedIntegrity(bytes[8]));
        }
        if bytes[9] != 24 {
            return Err(PhysicalRecordFormatDenial::UnsupportedRecordIdentityBytes(
                bytes[9],
            ));
        }
        Ok(Self::canonical(page_size))
    }

    pub(crate) const fn encode(self) -> [u8; 10] {
        let version = (self.version as u16).to_le_bytes();
        let page = self.page_size.bytes().to_le_bytes();
        [
            version[0],
            version[1],
            page[0],
            page[1],
            page[2],
            page[3],
            self.byte_order as u8,
            self.root_protocol as u8,
            self.integrity as u8,
            24,
        ]
    }

    const fn canonical(page_size: PhysicalPageSizeClass) -> Self {
        Self {
            version: PhysicalRecordFormatVersion::V2,
            page_size,
            byte_order: PhysicalRecordByteOrder::LittleEndian,
            root_protocol: PhysicalRecordRootProtocol::StagedCatalogV1,
            integrity: PhysicalRecordIntegrity::Crc32c,
        }
    }
}

impl PhysicalRecordFormatDeclarationBuilder {
    pub fn format_version(mut self, version: PhysicalRecordFormatVersion) -> Self {
        self.version = Some(version);
        self
    }

    pub fn page_size(mut self, page_size: PhysicalPageSizeClass) -> Self {
        self.page_size = Some(page_size);
        self
    }

    pub fn admit(self) -> Result<PhysicalRecordFormatDeclaration, PhysicalRecordFormatDenial> {
        let version = self.version.unwrap_or(PhysicalRecordFormatVersion::V2);
        let page_size = self.page_size.unwrap_or(PhysicalPageSizeClass::KiB16);
        Ok(PhysicalRecordFormatDeclaration {
            version,
            page_size,
            byte_order: PhysicalRecordByteOrder::LittleEndian,
            root_protocol: PhysicalRecordRootProtocol::StagedCatalogV1,
            integrity: PhysicalRecordIntegrity::Crc32c,
        })
    }
}

fn decode_page_size(bytes: u32) -> Result<PhysicalPageSizeClass, PhysicalRecordFormatDenial> {
    match bytes {
        16_384 => Ok(PhysicalPageSizeClass::KiB16),
        32_768 => Ok(PhysicalPageSizeClass::KiB32),
        65_536 => Ok(PhysicalPageSizeClass::KiB64),
        _ => Err(PhysicalRecordFormatDenial::UnsupportedPageBytes(bytes)),
    }
}
use super::PhysicalPageSizeClass;
