use worth_store::physical_runtime::{
    IntegrityAdmittedRecoveryWalFrame as StoreAdmittedWalFrame, RecoveryWalIntegrityAdmissionDenial,
};
use worth_store_physical_integrity::IntegrityValidatedWalFrame;

use super::super::{
    ObservedWalFrameSource, RecoveryIntegrityIngressCounters, RecoveryIntegrityIngressRejection,
};

pub(crate) struct IntegrityAdmittedWalFrame<'media> {
    admitted: StoreAdmittedWalFrame,
    _source_lifetime: std::marker::PhantomData<&'media ()>,
}

impl<'media> IntegrityAdmittedWalFrame<'media> {
    pub(in crate::integrity_ingress) fn bind(
        owner: &worth_store::physical_runtime::PhysicalRecoveryCoordination,
        source: ObservedWalFrameSource<'media>,
        validated: IntegrityValidatedWalFrame<'media>,
    ) -> Result<Self, RecoveryIntegrityIngressRejection> {
        let scope = source.scope();
        let admitted = owner
            .admit_recovery_wal_frame(source.observed(), scope, source.relative_range(), validated)
            .map_err(map_store_denial)?;
        Ok(Self {
            admitted,
            _source_lifetime: std::marker::PhantomData,
        })
    }

    /// Consumes ingress into the Store-owned frame carrying the admitted redo projection.
    pub(crate) fn into_owner_redo_projection(
        self,
        counters: &mut RecoveryIntegrityIngressCounters,
    ) -> StoreAdmittedWalFrame {
        counters.record_owner_projection();
        self.admitted
    }

    pub(crate) const fn scope(&self) -> worth_store_physical_integrity::PhysicalArtifactScope {
        self.admitted.scope()
    }
}

fn map_store_denial(
    denial: RecoveryWalIntegrityAdmissionDenial,
) -> RecoveryIntegrityIngressRejection {
    match denial {
        RecoveryWalIntegrityAdmissionDenial::MissingBoundedArtifact => {
            RecoveryIntegrityIngressRejection::MissingBoundedArtifact
        }
        RecoveryWalIntegrityAdmissionDenial::ScopeMismatch => {
            RecoveryIntegrityIngressRejection::ScopeMismatch
        }
        RecoveryWalIntegrityAdmissionDenial::SourceRangeOutsideObservation => {
            RecoveryIntegrityIngressRejection::SourceRangeOutsideObservation
        }
        RecoveryWalIntegrityAdmissionDenial::SourceIncarnationMismatch => {
            RecoveryIntegrityIngressRejection::SourceIncarnationMismatch
        }
    }
}
