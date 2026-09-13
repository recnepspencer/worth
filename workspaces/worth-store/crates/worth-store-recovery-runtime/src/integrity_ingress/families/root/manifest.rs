use worth_store_physical_format::{DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration};
use worth_store_physical_integrity::IntegrityValidatedRootManifest;

use super::super::super::admission::require_observed_recovery_source;
use super::super::super::{
    ObservedRecoverySource, RecoveryIntegrityIngressCounters, RecoveryIntegrityIngressRejection,
};

pub(crate) struct IntegrityAdmittedRootManifest<'media> {
    source: ObservedRecoverySource<'media>,
    validated: IntegrityValidatedRootManifest<'media>,
}

impl<'media> IntegrityAdmittedRootManifest<'media> {
    pub(in crate::integrity_ingress) fn bind(
        source: ObservedRecoverySource<'media>,
        validated: IntegrityValidatedRootManifest<'media>,
    ) -> Result<Self, RecoveryIntegrityIngressRejection> {
        require_observed_recovery_source(&source, validated.scope(), |input| {
            validated.matches_input(input)
        })?;
        let admitted = Self { source, validated };
        let (manifest, format) = admitted.manifest();
        if admitted.source.input()?.bytes() != manifest.encode(format).as_slice() {
            return Err(RecoveryIntegrityIngressRejection::NonCanonicalEncoding);
        }
        Ok(admitted)
    }

    fn manifest(&self) -> (DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration) {
        let manifest = DurablePhysicalRootManifest::builder(
            self.validated.root_generation(),
            self.validated.tree_identity(),
            self.validated.node_capacity(),
            self.validated.free_space_checksum(),
        )
        .record_count(self.validated.record_count())
        .next_block(self.validated.next_block())
        .next_segment_block(self.validated.next_segment_block())
        .routing_root(self.validated.routing_root())
        .segment_root(self.validated.segment_root())
        .free_space_root(self.validated.free_space_root())
        .last_inline_record(self.validated.last_inline_record())
        .last_inline_segment(self.validated.last_inline_segment())
        .admit()
        .expect("sealed root-manifest fields preserve the format contract");
        (manifest, self.validated.record_format())
    }

    pub(crate) fn project(self) -> (DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration) {
        self.manifest()
    }

    pub(crate) fn project_for_recovery(
        &self,
        counters: &mut RecoveryIntegrityIngressCounters,
    ) -> (DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration) {
        counters.record_owner_projection();
        self.manifest()
    }

    pub(crate) fn scope(&self) -> worth_store_physical_integrity::PhysicalArtifactScope {
        self.source.scope()
    }

    pub(crate) fn bind_checkpoint_base(
        &self,
        selected: &worth_store_recovery_physics::SelectedPhysicalRoot,
        checkpoint: worth_store_physical_integrity::VerifiedCheckpointStream,
        counters: &mut RecoveryIntegrityIngressCounters,
    ) -> Result<
        worth_store_recovery_physics::PhysicalCheckpointBase,
        worth_store_recovery_physics::PhysicalCheckpointBaseDenial,
    > {
        counters.record_owner_projection();
        worth_store_recovery_physics::PhysicalCheckpointBase::admit(
            selected,
            checkpoint,
            &self.validated,
        )
    }
}

pub(crate) fn admit_root_manifest<'media>(
    source: ObservedRecoverySource<'media>,
    validated: IntegrityValidatedRootManifest<'media>,
) -> Result<IntegrityAdmittedRootManifest<'media>, RecoveryIntegrityIngressRejection> {
    IntegrityAdmittedRootManifest::bind(source, validated)
}

#[cfg(test)]
pub(super) fn owner_valid_compile_contract() {
    fn bind<'media>(
        source: ObservedRecoverySource<'media>,
        validated: IntegrityValidatedRootManifest<'media>,
    ) {
        let _ = admit_root_manifest(source, validated);
    }
    let _ = bind;
}
