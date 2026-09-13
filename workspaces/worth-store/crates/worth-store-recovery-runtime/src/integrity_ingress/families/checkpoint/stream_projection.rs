use worth_store::physical_runtime::{
    ObservedRecoveryArtifact, StoreRecoveryCheckpointBindingBasis,
    StoreRecoveryCheckpointBindingRebuilder,
};
use worth_store_physical_format::PhysicalCheckpointIdentity;
use worth_store_physical_integrity::{
    UntrustedPhysicalArtifact, VerifiedCheckpointStream, VerifiedCheckpointStreamAssemblyDenial,
};

use super::{
    IntegrityAdmittedCheckpointBinding, IntegrityAdmittedCheckpointBindingCompaction,
    IntegrityAdmittedCheckpointDirtyBasis, IntegrityAdmittedCheckpointFooter,
    IntegrityAdmittedCheckpointStreamHeader,
};
use crate::integrity_ingress::{
    ObservedRecoverySource, RecoveryIntegrityIngressObservation, RecoveryIntegrityIngressRejection,
    RecoveryIntegrityIngressTrace,
};

pub(crate) struct IntegrityAdmittedCheckpointStream<'media> {
    header: IntegrityAdmittedCheckpointStreamHeader<'media>,
    dirty: Vec<IntegrityAdmittedCheckpointDirtyBasis<'media>>,
    compaction: IntegrityAdmittedCheckpointBindingCompaction<'media>,
    bindings: Vec<IntegrityAdmittedCheckpointBinding<'media>>,
    footer: IntegrityAdmittedCheckpointFooter<'media>,
}

pub(crate) struct OwnerCheckpointProjection {
    pub(crate) checkpoint: VerifiedCheckpointStream,
    pub(crate) binding_basis: StoreRecoveryCheckpointBindingBasis,
}

impl<'media> IntegrityAdmittedCheckpointStream<'media> {
    pub(crate) fn assemble(
        header: IntegrityAdmittedCheckpointStreamHeader<'media>,
        dirty: Vec<IntegrityAdmittedCheckpointDirtyBasis<'media>>,
        compaction: IntegrityAdmittedCheckpointBindingCompaction<'media>,
        bindings: Vec<IntegrityAdmittedCheckpointBinding<'media>>,
        footer: IntegrityAdmittedCheckpointFooter<'media>,
    ) -> Result<Self, RecoveryIntegrityIngressRejection> {
        let observed = header.source().observed();
        let identity = header.checkpoint_identity();
        let mut next_offset = 0_u64;
        require_record(header.source(), observed, None, &mut next_offset)?;
        for record in &dirty {
            require_record(record.source(), observed, Some(identity), &mut next_offset)?;
        }
        require_record(
            compaction.source(),
            observed,
            Some(identity),
            &mut next_offset,
        )?;
        for record in &bindings {
            require_record(record.source(), observed, Some(identity), &mut next_offset)?;
        }
        require_record(footer.source(), observed, Some(identity), &mut next_offset)?;
        let observed_bytes = observed
            .bytes()
            .ok_or(RecoveryIntegrityIngressRejection::MissingBoundedArtifact)?;
        if next_offset != observed_bytes.len() as u64 {
            return Err(RecoveryIntegrityIngressRejection::ScopeMismatch);
        }
        Ok(Self {
            header,
            dirty,
            compaction,
            bindings,
            footer,
        })
    }

    pub(crate) fn into_owner_checkpoint(
        self,
        maximum_binding_records: u64,
        trace: &mut RecoveryIntegrityIngressTrace,
    ) -> Result<OwnerCheckpointProjection, RecoveryIntegrityIngressRejection> {
        let footer_scope = self.footer.scope();
        let bytes = self
            .header
            .source()
            .observed()
            .bytes()
            .ok_or(RecoveryIntegrityIngressRejection::MissingBoundedArtifact)?;
        let dirty = self
            .dirty
            .iter()
            .map(IntegrityAdmittedCheckpointDirtyBasis::validated)
            .collect::<Vec<_>>();
        let bindings = self
            .bindings
            .iter()
            .map(IntegrityAdmittedCheckpointBinding::validated)
            .collect::<Vec<_>>();
        let source = self.header.validated().source();
        let mut binding_rebuilder = StoreRecoveryCheckpointBindingRebuilder::begin(
            source.identity().store_identity(),
            source,
            self.footer
                .validated()
                .footer()
                .binding_compaction_generation(),
            maximum_binding_records,
        );
        for binding in &self.bindings {
            binding_rebuilder.consume(binding.validated(), binding.source().input()?);
        }
        let verified = VerifiedCheckpointStream::assemble_from_validated_records(
            UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
            self.header.validated(),
            &dirty,
            self.compaction.validated(),
            &bindings,
            self.footer.validated(),
        )
        .map_err(|denial| match denial {
            VerifiedCheckpointStreamAssemblyDenial::SourceIdentityMismatch
            | VerifiedCheckpointStreamAssemblyDenial::RecordScopeMismatch => {
                RecoveryIntegrityIngressRejection::ScopeMismatch
            }
            VerifiedCheckpointStreamAssemblyDenial::InputIncarnationMismatch => {
                RecoveryIntegrityIngressRejection::SourceIncarnationMismatch
            }
            VerifiedCheckpointStreamAssemblyDenial::FooterBasisMismatch(rejection) => {
                RecoveryIntegrityIngressRejection::Integrity(rejection)
            }
        })?;
        trace.record(RecoveryIntegrityIngressObservation::admitted(footer_scope));
        let counters = trace.counters_mut();
        counters.record_owner_projection();
        for _ in &self.dirty {
            counters.record_owner_projection();
        }
        counters.record_owner_projection();
        for _ in &self.bindings {
            counters.record_owner_projection();
        }
        counters.record_owner_projection();
        let binding_basis = binding_rebuilder.finish(&verified);
        Ok(OwnerCheckpointProjection {
            checkpoint: verified,
            binding_basis,
        })
    }
}

fn require_record(
    source: &ObservedRecoverySource<'_>,
    observed: &ObservedRecoveryArtifact,
    expected_identity: Option<PhysicalCheckpointIdentity>,
    next_offset: &mut u64,
) -> Result<(), RecoveryIntegrityIngressRejection> {
    if !core::ptr::eq(source.observed(), observed)
        || expected_identity.is_some() && source.scope().checkpoint_identity() != expected_identity
    {
        return Err(RecoveryIntegrityIngressRejection::SourceIncarnationMismatch);
    }
    let range = source.selected_range();
    if range.offset() != *next_offset || range != source.scope().byte_range() {
        return Err(RecoveryIntegrityIngressRejection::ScopeMismatch);
    }
    *next_offset = range.end_exclusive();
    Ok(())
}
