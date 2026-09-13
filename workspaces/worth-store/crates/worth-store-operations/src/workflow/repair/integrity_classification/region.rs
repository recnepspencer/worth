#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::workflow::repair) enum IntegrityRepairRegionClass {
    DerivedRebuildable,
    AuthorityTrustedSourceRequired,
    ContentTrustedSourceRequired,
    QuarantineRequired,
    Indeterminate,
    Unrecoverable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::workflow::repair) enum IntegrityRepairArtifactFamily {
    Manifest,
    Page,
    Extent,
    Wal,
    LayoutIndex,
    BlobChunk,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::workflow::repair) struct IntegrityRepairOwnerBinding {
    family: IntegrityRepairArtifactFamily,
    observed_generation: Option<u64>,
    physical_owner_identity: Option<[u8; 32]>,
    security_scope_identity: Option<[u8; 32]>,
}

impl IntegrityRepairOwnerBinding {
    pub(in crate::workflow::repair) const fn observed(
        family: IntegrityRepairArtifactFamily,
        observed_generation: Option<u64>,
        physical_owner_identity: Option<[u8; 32]>,
        security_scope_identity: Option<[u8; 32]>,
    ) -> Self {
        Self {
            family,
            observed_generation,
            physical_owner_identity,
            security_scope_identity,
        }
    }

    pub(in crate::workflow::repair) const fn family(self) -> IntegrityRepairArtifactFamily {
        self.family
    }

    pub(in crate::workflow::repair) const fn observed_generation(self) -> Option<u64> {
        self.observed_generation
    }

    pub(in crate::workflow::repair) const fn physical_owner_identity(self) -> Option<[u8; 32]> {
        self.physical_owner_identity
    }

    pub(in crate::workflow::repair) const fn security_scope_identity(self) -> Option<[u8; 32]> {
        self.security_scope_identity
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::workflow::repair) struct IntegrityRepairRegion {
    start: u64,
    end_exclusive: u64,
    identity: [u8; 32],
    class: IntegrityRepairRegionClass,
    evidence_digest: [u8; 32],
    target_identity: [u8; 32],
    owner_binding: IntegrityRepairOwnerBinding,
}

impl IntegrityRepairRegion {
    pub(in crate::workflow::repair) fn bounded(
        identity: [u8; 32],
        start: u64,
        end_exclusive: u64,
        class: IntegrityRepairRegionClass,
        evidence_digest: [u8; 32],
        target_identity: [u8; 32],
        owner_binding: IntegrityRepairOwnerBinding,
    ) -> Option<Self> {
        if identity == [0; 32]
            || evidence_digest == [0; 32]
            || target_identity == [0; 32]
            || start >= end_exclusive
        {
            return None;
        }
        Some(Self {
            start,
            end_exclusive,
            identity,
            class,
            evidence_digest,
            target_identity,
            owner_binding,
        })
    }

    pub(in crate::workflow::repair) const fn start(self) -> u64 {
        self.start
    }

    pub(in crate::workflow::repair) const fn end_exclusive(self) -> u64 {
        self.end_exclusive
    }

    pub(in crate::workflow::repair) const fn identity(self) -> [u8; 32] {
        self.identity
    }

    pub(in crate::workflow::repair) const fn class(self) -> IntegrityRepairRegionClass {
        self.class
    }

    pub(in crate::workflow::repair) const fn evidence_digest(self) -> [u8; 32] {
        self.evidence_digest
    }

    pub(in crate::workflow::repair) const fn target_identity(self) -> [u8; 32] {
        self.target_identity
    }

    pub(in crate::workflow::repair) const fn owner_binding(self) -> IntegrityRepairOwnerBinding {
        self.owner_binding
    }
}

pub(in crate::workflow::repair) const fn family_tag(family: IntegrityRepairArtifactFamily) -> u8 {
    match family {
        IntegrityRepairArtifactFamily::Manifest => 1,
        IntegrityRepairArtifactFamily::Page => 2,
        IntegrityRepairArtifactFamily::Extent => 3,
        IntegrityRepairArtifactFamily::Wal => 4,
        IntegrityRepairArtifactFamily::LayoutIndex => 5,
        IntegrityRepairArtifactFamily::BlobChunk => 6,
        IntegrityRepairArtifactFamily::Unknown => 7,
    }
}

pub(in crate::workflow::repair) const fn class_tag(class: IntegrityRepairRegionClass) -> u8 {
    match class {
        IntegrityRepairRegionClass::DerivedRebuildable => 1,
        IntegrityRepairRegionClass::AuthorityTrustedSourceRequired => 2,
        IntegrityRepairRegionClass::ContentTrustedSourceRequired => 3,
        IntegrityRepairRegionClass::QuarantineRequired => 4,
        IntegrityRepairRegionClass::Indeterminate => 5,
        IntegrityRepairRegionClass::Unrecoverable => 6,
    }
}
