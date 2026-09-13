use worth_store_physical_integrity::PhysicalArtifactScope;

/// Owner disposition for an intact derived artifact and its current basis.
///
/// C.9 has no current derived family, so this type intentionally has no issuer.
/// The first real family adds its concrete owner adapter without reshaping the
/// public disposition contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntactPhysicalDerivedObservation {
    derived_scope: PhysicalArtifactScope,
    authoritative_basis_scope: PhysicalArtifactScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamagedPhysicalDerivedDisposition {
    RebuildableDerived(RebuildablePhysicalDerivedObservation),
    Unknown(UnknownDerivedRebuildability),
    Indeterminate(IndeterminateDerivedRebuildability),
}

/// Owner observation that a damaged derived artifact has an intact current basis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RebuildablePhysicalDerivedObservation {
    damaged_derived_scope: PhysicalArtifactScope,
    intact_authoritative_basis_scope: PhysicalArtifactScope,
}

/// Owner observation that rebuildability cannot be classified from known facts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnknownDerivedRebuildability {
    damaged_derived_scope: PhysicalArtifactScope,
}

/// Owner observation that rebuildability could not be decided by a bounded check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndeterminateDerivedRebuildability {
    damaged_derived_scope: PhysicalArtifactScope,
}

impl IntactPhysicalDerivedObservation {
    pub const fn derived_scope(self) -> PhysicalArtifactScope {
        self.derived_scope
    }

    pub const fn authoritative_basis_scope(self) -> PhysicalArtifactScope {
        self.authoritative_basis_scope
    }
}

impl RebuildablePhysicalDerivedObservation {
    pub const fn damaged_derived_scope(self) -> PhysicalArtifactScope {
        self.damaged_derived_scope
    }

    pub const fn intact_authoritative_basis_scope(self) -> PhysicalArtifactScope {
        self.intact_authoritative_basis_scope
    }
}

impl UnknownDerivedRebuildability {
    pub const fn damaged_derived_scope(self) -> PhysicalArtifactScope {
        self.damaged_derived_scope
    }
}

impl IndeterminateDerivedRebuildability {
    pub const fn damaged_derived_scope(self) -> PhysicalArtifactScope {
        self.damaged_derived_scope
    }
}
