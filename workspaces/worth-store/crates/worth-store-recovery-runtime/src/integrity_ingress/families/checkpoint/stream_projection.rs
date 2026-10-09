use worth_store::physical_runtime::{
    ObservedRecoveryArtifact, SharedRecoveryCheckpoint, StoreRecoveryCheckpointBindingBasis,
};
use worth_store_physical_format::PhysicalCheckpointIdentity;
use worth_store_physical_integrity::{
    PhysicalByteRange, UntrustedPhysicalArtifact, VerifiedCheckpointStream,
    VerifiedCheckpointStreamAssemblyDenial,
};

use super::{
    IntegrityAdmittedCheckpointBinding, IntegrityAdmittedCheckpointBindingCompaction,
    IntegrityAdmittedCheckpointDirtyBasis, IntegrityAdmittedCheckpointFooter,
    IntegrityAdmittedCheckpointStreamHeader,
};
use crate::integrity_ingress::{
    checkpoint_stream::RecordEvidenceAllocation, ObservedRecoverySource,
    RecoveryIntegrityIngressObservation, RecoveryIntegrityIngressRejection,
    RecoveryIntegrityIngressTrace,
};

pub(crate) struct IntegrityAdmittedCheckpointStream<'media> {
    header: IntegrityAdmittedCheckpointStreamHeader<'media>,
    dirty: Vec<IntegrityAdmittedCheckpointDirtyBasis<'media>>,
    compaction: IntegrityAdmittedCheckpointBindingCompaction<'media>,
    bindings: Vec<IntegrityAdmittedCheckpointBinding<'media>>,
    certificates: Vec<(PhysicalByteRange, &'media [u8])>,
    footer: IntegrityAdmittedCheckpointFooter<'media>,
}

pub(crate) struct OwnerCheckpointProjection {
    pub(crate) checkpoint: SharedRecoveryCheckpoint,
    pub(crate) binding_basis: StoreRecoveryCheckpointBindingBasis,
}

impl<'media> IntegrityAdmittedCheckpointStream<'media> {
    pub(crate) fn assemble(
        header: IntegrityAdmittedCheckpointStreamHeader<'media>,
        dirty: Vec<IntegrityAdmittedCheckpointDirtyBasis<'media>>,
        compaction: IntegrityAdmittedCheckpointBindingCompaction<'media>,
        bindings: Vec<IntegrityAdmittedCheckpointBinding<'media>>,
        certificates: Vec<(PhysicalByteRange, &'media [u8])>,
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
        for (range, bytes) in &certificates {
            if range.offset() != next_offset
                || observed.bytes().and_then(|all| {
                    all.get(range.offset() as usize..range.end_exclusive() as usize)
                }) != Some(*bytes)
            {
                return Err(RecoveryIntegrityIngressRejection::ScopeMismatch);
            }
            next_offset = range.end_exclusive();
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
            certificates,
            footer,
        })
    }

    pub(crate) fn into_owner_checkpoint(
        self,
        maximum_binding_records: u64,
        trace: &mut RecoveryIntegrityIngressTrace,
        allocation: &mut RecordEvidenceAllocation<'_, '_>,
    ) -> Result<OwnerCheckpointProjection, crate::integrity_ingress::CheckpointStreamAdmissionFailure>
    {
        use crate::integrity_ingress::CheckpointStreamAdmissionFailure as Failure;
        let footer_scope = self.footer.scope();
        let bytes = self
            .header
            .source()
            .observed()
            .bytes()
            .ok_or(RecoveryIntegrityIngressRejection::MissingBoundedArtifact)?;
        let mut dirty = allocation.reserve_records(self.dirty.len() as u64)?;
        dirty.extend(
            self.dirty
                .iter()
                .map(IntegrityAdmittedCheckpointDirtyBasis::validated),
        );
        let mut bindings = allocation.reserve_records(self.bindings.len() as u64)?;
        bindings.extend(
            self.bindings
                .iter()
                .map(IntegrityAdmittedCheckpointBinding::validated),
        );
        let assembly = VerifiedCheckpointStream::validate_records_with_certificates(
            UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
            self.header.validated(),
            &dirty,
            self.compaction.validated(),
            &bindings,
            &self.certificates,
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
        if assembly.facts().source().root().generation() == 0 {
            return Err(Failure::Binding(
                worth_store_recovery_physics::PhysicalCheckpointBaseDenial::RootGenerationMismatch,
            ));
        }
        let verified = allocation
            .retain_checkpoint(assembly)
            .map_err(Failure::Backing)?;
        let mut binding_rebuilder =
            allocation.begin_binding_rebuild(&verified, maximum_binding_records)?;
        for binding in &self.bindings {
            allocation.consume_binding(
                &mut binding_rebuilder,
                binding.validated(),
                binding.source().input()?,
            )?;
        }
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
        let binding_basis = binding_rebuilder
            .finish()
            .map_err(Failure::BindingBasisBacking)?;
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
