use worth_store::physical_runtime::ObservedRecoveryArtifact;
use worth_store_physical_integrity::{
    PhysicalArtifactScope, PhysicalByteRange, UntrustedPhysicalArtifact,
};

use super::super::RecoveryIntegrityIngressRejection;

/// One exact borrowed incarnation of a C.4 bounded recovery observation.
pub(crate) struct ObservedRecoverySource<'media> {
    observed: &'media ObservedRecoveryArtifact,
    scope: PhysicalArtifactScope,
    selection: ObservationSelection,
}

#[derive(Clone, Copy)]
enum ObservationSelection {
    Complete,
    RelativeRange(PhysicalByteRange),
}

impl<'media> ObservedRecoverySource<'media> {
    pub(crate) const fn complete(
        observed: &'media ObservedRecoveryArtifact,
        scope: PhysicalArtifactScope,
    ) -> Self {
        Self {
            observed,
            scope,
            selection: ObservationSelection::Complete,
        }
    }

    pub(crate) const fn bounded_subrange(
        observed: &'media ObservedRecoveryArtifact,
        scope: PhysicalArtifactScope,
        relative_range: PhysicalByteRange,
    ) -> Self {
        Self {
            observed,
            scope,
            selection: ObservationSelection::RelativeRange(relative_range),
        }
    }

    pub(in crate::integrity_ingress) const fn scope(&self) -> PhysicalArtifactScope {
        self.scope
    }

    pub(in crate::integrity_ingress) const fn observed(&self) -> &'media ObservedRecoveryArtifact {
        self.observed
    }

    pub(in crate::integrity_ingress) const fn selected_range(&self) -> PhysicalByteRange {
        match self.selection {
            ObservationSelection::Complete => self.scope.byte_range(),
            ObservationSelection::RelativeRange(range) => range,
        }
    }

    pub(in crate::integrity_ingress) fn input(
        &self,
    ) -> Result<UntrustedPhysicalArtifact<'media>, RecoveryIntegrityIngressRejection> {
        if self.observed.store_identity() != self.scope.store_identity()
            || !super::artifact_scope::matches_artifact(self.observed.artifact(), self.scope)
        {
            return Err(RecoveryIntegrityIngressRejection::ScopeMismatch);
        }
        let bytes = self
            .observed
            .bytes()
            .ok_or(RecoveryIntegrityIngressRejection::MissingBoundedArtifact)?;
        let inspected = match self.selection {
            ObservationSelection::Complete => {
                if self.observed.offset() != self.scope.byte_range().offset() {
                    return Err(RecoveryIntegrityIngressRejection::ScopeMismatch);
                }
                bytes
            }
            ObservationSelection::RelativeRange(range) => {
                if self.observed.offset().checked_add(range.offset())
                    != Some(self.scope.byte_range().offset())
                    || range.length() != self.scope.byte_range().length()
                {
                    return Err(RecoveryIntegrityIngressRejection::ScopeMismatch);
                }
                bounded_slice(bytes, range)?
            }
        };
        if inspected.len() as u64 != self.scope.byte_range().length() {
            return Err(RecoveryIntegrityIngressRejection::ScopeMismatch);
        }
        Ok(UntrustedPhysicalArtifact::from_bounded_bytes(inspected))
    }
}

fn bounded_slice(
    bytes: &[u8],
    range: PhysicalByteRange,
) -> Result<&[u8], RecoveryIntegrityIngressRejection> {
    let start = usize::try_from(range.offset())
        .map_err(|_| RecoveryIntegrityIngressRejection::SourceRangeOutsideObservation)?;
    let end = usize::try_from(range.end_exclusive())
        .map_err(|_| RecoveryIntegrityIngressRejection::SourceRangeOutsideObservation)?;
    bytes
        .get(start..end)
        .ok_or(RecoveryIntegrityIngressRejection::SourceRangeOutsideObservation)
}
