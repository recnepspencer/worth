//! One bounded home manifest, with opaque owner-native section bytes.
use worth_foundational::facade::{
    BoundaryProtocolCompatibilityWindow, BoundaryProtocolIdentity,
    BoundaryProtocolUnsupportedVersion, BoundaryProtocolVersion, DurableArtifactRetention,
    DurableOwnerFamily, RetentionClass,
};
use worth_store::physical_runtime::{AnchoredRecord, AnchoredRecordMembership};

pub const HOME_CHECKPOINT_PROTOCOL: BoundaryProtocolIdentity =
    BoundaryProtocolIdentity::new("worth.application.home.checkpoint");
pub const HOME_CHECKPOINT_VERSION: BoundaryProtocolVersion = BoundaryProtocolVersion::new(1);
const MAGIC: &[u8; 8] = b"WAPHCKP1";
pub(super) const HEADER_BYTES: usize = 24;

#[derive(Debug)]
pub enum HomeManifestDenial {
    Malformed,
    UnsupportedVersion(BoundaryProtocolUnsupportedVersion),
    InvalidVersion,
    UnsupportedOwner(DurableOwnerFamily),
    UnknownOwner { tag: u16 },
}

pub(crate) struct SelectedHomeManifest {
    membership: AnchoredRecordMembership,
    bytes: Vec<u8>,
}

impl DurableArtifactRetention for SelectedHomeManifest {
    const RETENTION_CLASS: RetentionClass = RetentionClass::Derived;
}

impl SelectedHomeManifest {
    pub(crate) fn from_record(record: AnchoredRecord) -> Result<Self, HomeManifestDenial> {
        let (membership, bytes) = record.into_parts();
        decode(&bytes)?;
        Ok(Self { membership, bytes })
    }

    pub(crate) fn relational(&self) -> (&[u8], &AnchoredRecordMembership) {
        (&self.bytes[HEADER_BYTES..], &self.membership)
    }
    pub(crate) fn generation(&self) -> u64 {
        self.membership.version().generation()
    }
    pub(crate) fn members(&self) -> &[worth_store::physical_runtime::AnchoredRecordMemberLocator] {
        self.membership.members()
    }
}

pub(super) fn emit(section: &[u8], bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&HOME_CHECKPOINT_VERSION.get().to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&DurableOwnerFamily::Relational.tag().to_le_bytes());
    bytes.extend_from_slice(&(section.len() as u64).to_le_bytes());
    bytes.extend_from_slice(section);
}

fn decode(bytes: &[u8]) -> Result<(), HomeManifestDenial> {
    use HomeManifestDenial as Denial;
    let rest = bytes.strip_prefix(MAGIC).ok_or(Denial::Malformed)?;
    let (version, rest) = rest.split_first_chunk::<4>().ok_or(Denial::Malformed)?;
    let version = BoundaryProtocolVersion::try_new(u32::from_le_bytes(*version))
        .map_err(|_| Denial::InvalidVersion)?;
    BoundaryProtocolCompatibilityWindow::inclusive(
        HOME_CHECKPOINT_VERSION,
        HOME_CHECKPOINT_VERSION,
    )
    .admit(version)
    .map_err(Denial::UnsupportedVersion)?;
    let (count, rest) = rest.split_first_chunk::<2>().ok_or(Denial::Malformed)?;
    if u16::from_le_bytes(*count) != 1 {
        return Err(Denial::Malformed);
    }
    let (tag, rest) = rest.split_first_chunk::<2>().ok_or(Denial::Malformed)?;
    let tag = u16::from_le_bytes(*tag);
    let owner = DurableOwnerFamily::from_tag(tag).map_err(|_| Denial::UnknownOwner { tag })?;
    match owner {
        DurableOwnerFamily::Relational => {}
        DurableOwnerFamily::Signal
        | DurableOwnerFamily::Bridge
        | DurableOwnerFamily::World
        | DurableOwnerFamily::Query
        | DurableOwnerFamily::Dispatch
        | DurableOwnerFamily::Correction => return Err(Denial::UnsupportedOwner(owner)),
    }
    let (length, rest) = rest.split_first_chunk::<8>().ok_or(Denial::Malformed)?;
    if u64::from_le_bytes(*length) != rest.len() as u64 {
        return Err(Denial::Malformed);
    }
    Ok(())
}
