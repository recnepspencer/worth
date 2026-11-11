//! One bounded home manifest, with opaque owner-native section bytes.
#[cfg(test)]
mod tests;
use worth_foundational::facade::{
    BoundaryProtocolCompatibilityWindow, BoundaryProtocolIdentity,
    BoundaryProtocolUnsupportedVersion, BoundaryProtocolVersion, DurableArtifactRetention,
    DurableOwnerFamily, RetentionClass,
};
use worth_store::physical_runtime::{AnchoredRecordMembership, AnchoredRecordReadMembership};

pub const HOME_CHECKPOINT_PROTOCOL: BoundaryProtocolIdentity =
    BoundaryProtocolIdentity::new("worth.application.home.checkpoint");
pub const HOME_CHECKPOINT_VERSION: BoundaryProtocolVersion = BoundaryProtocolVersion::new(3);
const MAGIC: &[u8; 8] = b"WAPHCKP1";
pub(super) const HEADER_BYTES: usize = 44;

#[derive(Debug)]
pub enum HomeManifestDenial {
    Malformed,
    UnsupportedVersion(BoundaryProtocolUnsupportedVersion),
    InvalidVersion,
    SizeLimit,
    UnsupportedOwner(DurableOwnerFamily),
    UnknownOwner { tag: u16 },
}

#[derive(Clone)]
pub(crate) struct SelectedHomeManifest {
    membership: AnchoredRecordReadMembership,
    bytes: worth_execution::ExecutionImmutableBytes,
    relational: std::ops::Range<usize>,
    world: std::ops::Range<usize>,
    signal: std::ops::Range<usize>,
}

impl DurableArtifactRetention for SelectedHomeManifest {
    const RETENTION_CLASS: RetentionClass = RetentionClass::Derived;
}

impl SelectedHomeManifest {
    pub(super) fn from_verified_read(
        bytes: worth_execution::ExecutionImmutableBytes,
        membership: AnchoredRecordReadMembership,
    ) -> Result<Self, HomeManifestDenial> {
        let sections = decode(bytes.as_ref())?;
        let native_end = HEADER_BYTES
            .checked_add(sections.relational.len())
            .ok_or(HomeManifestDenial::SizeLimit)?;
        let relational = HEADER_BYTES..native_end;
        let world_end = native_end
            .checked_add(sections.world.len())
            .ok_or(HomeManifestDenial::SizeLimit)?;
        let world = native_end..world_end;
        let signal_end = world_end
            .checked_add(sections.signal.len())
            .ok_or(HomeManifestDenial::SizeLimit)?;
        let signal = world_end..signal_end;
        bytes
            .get(relational.clone())
            .ok_or(HomeManifestDenial::Malformed)?;
        bytes
            .get(world.clone())
            .ok_or(HomeManifestDenial::Malformed)?;
        Ok(Self {
            membership,
            bytes,
            relational,
            world,
            signal,
        })
    }

    pub(crate) fn relational(&self) -> (&[u8], &AnchoredRecordMembership) {
        (
            self.bytes
                .get(self.relational.clone())
                .expect("admitted Native region"),
            self.membership.membership(),
        )
    }
    pub(crate) fn world(&self) -> &[u8] {
        self.bytes
            .get(self.world.clone())
            .expect("admitted World region")
    }
    pub(crate) fn relational_read(
        &self,
    ) -> (
        worth_execution::ExecutionImmutableBytes,
        std::ops::Range<usize>,
        &AnchoredRecordReadMembership,
    ) {
        (
            self.bytes.clone(),
            self.relational.clone(),
            &self.membership,
        )
    }
    pub(crate) fn world_region(
        &self,
    ) -> (
        worth_execution::ExecutionImmutableBytes,
        std::ops::Range<usize>,
    ) {
        (self.bytes.clone(), self.world.clone())
    }
    pub(crate) fn signal_region(
        &self,
    ) -> (
        worth_execution::ExecutionImmutableBytes,
        std::ops::Range<usize>,
    ) {
        (self.bytes.clone(), self.signal.clone())
    }
    pub(crate) fn generation(&self) -> u64 {
        self.membership.membership().version().generation()
    }
    pub(crate) fn members(&self) -> &[worth_store::physical_runtime::AnchoredRecordMemberLocator] {
        self.membership.membership().members()
    }
}

pub(super) fn emit(native: &[u8], world: &[u8], signal: &[u8], bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&HOME_CHECKPOINT_VERSION.get().to_le_bytes());
    bytes.extend_from_slice(&3_u16.to_le_bytes());
    for (family, section) in [
        (DurableOwnerFamily::Relational, native),
        (DurableOwnerFamily::World, world),
        (DurableOwnerFamily::Signal, signal),
    ] {
        bytes.extend_from_slice(&family.tag().to_le_bytes());
        bytes.extend_from_slice(&(section.len() as u64).to_le_bytes());
    }
    bytes.extend_from_slice(native);
    bytes.extend_from_slice(world);
    bytes.extend_from_slice(signal);
}

pub(crate) struct HomeManifestSections<'a> {
    pub(crate) relational: &'a [u8],
    pub(crate) world: &'a [u8],
    pub(crate) signal: &'a [u8],
}
pub(crate) fn decode(bytes: &[u8]) -> Result<HomeManifestSections<'_>, HomeManifestDenial> {
    use HomeManifestDenial as Denial;
    if bytes.len() > 1 << 30 {
        return Err(Denial::SizeLimit);
    }
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
    if u16::from_le_bytes(*count) != 3 {
        return Err(Denial::Malformed);
    }
    let (native, rest) = length(rest, DurableOwnerFamily::Relational)?;
    let (world, rest) = length(rest, DurableOwnerFamily::World)?;
    let (signal, payload) = length(rest, DurableOwnerFamily::Signal)?;
    if native == 0
        || world == 0
        || signal == 0
        || native
            .checked_add(world)
            .and_then(|size| size.checked_add(signal))
            != Some(payload.len())
    {
        return Err(Denial::Malformed);
    }
    let (relational, rest) = payload.split_at_checked(native).ok_or(Denial::Malformed)?;
    let (world, signal) = rest.split_at_checked(world).ok_or(Denial::Malformed)?;
    Ok(HomeManifestSections {
        relational,
        world,
        signal,
    })
}
fn length(
    bytes: &[u8],
    expected: DurableOwnerFamily,
) -> Result<(usize, &[u8]), HomeManifestDenial> {
    use HomeManifestDenial as Denial;
    let (tag, rest) = bytes.split_first_chunk::<2>().ok_or(Denial::Malformed)?;
    let tag = u16::from_le_bytes(*tag);
    let family = DurableOwnerFamily::from_tag(tag).map_err(|_| Denial::UnknownOwner { tag })?;
    if family != expected {
        return Err(match family {
            DurableOwnerFamily::Relational
            | DurableOwnerFamily::World
            | DurableOwnerFamily::Signal => Denial::Malformed,
            family @ (DurableOwnerFamily::Bridge
            | DurableOwnerFamily::Query
            | DurableOwnerFamily::Dispatch
            | DurableOwnerFamily::Correction) => Denial::UnsupportedOwner(family),
        });
    }
    let (length, rest) = rest.split_first_chunk::<8>().ok_or(Denial::Malformed)?;
    let length = usize::try_from(u64::from_le_bytes(*length)).map_err(|_| Denial::SizeLimit)?;
    Ok((length, rest))
}
