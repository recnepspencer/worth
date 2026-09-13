/// Checksum algorithm named by a persisted physical format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhysicalIntegrityAlgorithm {
    Crc32c,
    Sha256,
}
