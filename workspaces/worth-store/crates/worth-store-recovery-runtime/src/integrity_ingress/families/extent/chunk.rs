use sha2::{Digest, Sha256};
use worth_store::physical_runtime::ObservedRecoveryArtifact;
use worth_store_physical_format::PhysicalPageLsn;
use worth_store_physical_integrity::{
    validate_extent_chunk_membership, IntegrityValidatedExtentChunkFrame, PhysicalArtifactScope,
    SelectedExtentPayloadBuilder,
};

use super::super::super::admission::require_observed_recovery_source;
use super::super::super::{
    ObservedRecoverySource, RecoveryIntegrityIngressCounters, RecoveryIntegrityIngressRejection,
};

pub(crate) struct IntegrityAdmittedExtentChunkFrame<'media> {
    source: ObservedRecoverySource<'media>,
    validated: IntegrityValidatedExtentChunkFrame<'media>,
}

pub(crate) struct ExtentChunkProjection {
    pub page_lsn: PhysicalPageLsn,
    pub encoded_digest: [u8; 32],
}

pub(crate) fn admit_extent_chunk_projection(
    observed: &ObservedRecoveryArtifact,
    scope: PhysicalArtifactScope,
    membership: worth_store_physical_integrity::IntegrityValidatedExtentMembership,
    trace: &mut super::super::super::RecoveryIntegrityIngressTrace,
) -> Result<ExtentChunkProjection, RecoveryIntegrityIngressRejection> {
    admit_chunk(observed, scope, membership, trace, None)
        .map(|projection| projection.expect("ordinary chunk projection is present"))
}

pub(crate) fn admit_extent_chunk_projection_for_selected_record(
    observed: &ObservedRecoveryArtifact,
    scope: PhysicalArtifactScope,
    membership: worth_store_physical_integrity::IntegrityValidatedExtentMembership,
    trace: &mut super::super::super::RecoveryIntegrityIngressTrace,
    builder: &mut SelectedExtentPayloadBuilder,
) -> Result<Option<ExtentChunkProjection>, RecoveryIntegrityIngressRejection> {
    admit_chunk(observed, scope, membership, trace, Some(builder))
}

fn admit_chunk(
    observed: &ObservedRecoveryArtifact,
    scope: PhysicalArtifactScope,
    membership: worth_store_physical_integrity::IntegrityValidatedExtentMembership,
    trace: &mut super::super::super::RecoveryIntegrityIngressTrace,
    mut builder: Option<&mut SelectedExtentPayloadBuilder>,
) -> Result<Option<ExtentChunkProjection>, RecoveryIntegrityIngressRejection> {
    let input = ObservedRecoverySource::complete(observed, scope)
        .input()
        .map_err(|rejection| trace.reject(scope, rejection))?;
    let validation = validate_extent_chunk_membership(input, scope, membership).0;
    let attempt = super::super::super::IntegrityAdmittedRecoveryArtifact::bind_extent_chunk(
        observed,
        scope,
        validation,
        trace.counters_mut(),
    );
    trace.retain(attempt.observation());
    match attempt.into_outcome()? {
        super::super::super::IntegrityAdmittedRecoveryArtifact::ExtentChunk(admitted) => {
            if let Some(builder) = builder.as_mut() {
                let input = admitted.source.input().expect("bound C.4 chunk source");
                if builder.append(&admitted.validated, input).is_none() {
                    return Ok(None);
                }
            }
            Ok(Some(admitted.project(trace.counters_mut())))
        }
        _ => unreachable!("extent ingress returns its family-specific admitted variant"),
    }
}

impl<'media> IntegrityAdmittedExtentChunkFrame<'media> {
    pub(in crate::integrity_ingress) fn bind(
        source: ObservedRecoverySource<'media>,
        validated: IntegrityValidatedExtentChunkFrame<'media>,
    ) -> Result<Self, RecoveryIntegrityIngressRejection> {
        require_observed_recovery_source(&source, validated.scope(), |input| {
            validated.matches_input(input)
        })?;
        Ok(Self { source, validated })
    }

    pub(crate) fn project(
        &self,
        counters: &mut RecoveryIntegrityIngressCounters,
    ) -> ExtentChunkProjection {
        counters.record_owner_projection();
        counters.record_owner_decoder();
        let input = self
            .source
            .input()
            .expect("an admitted extent chunk retains its exact C.4 observation");
        ExtentChunkProjection {
            page_lsn: self.validated.page_lsn(),
            encoded_digest: Sha256::digest(input.bytes()).into(),
        }
    }
}

#[cfg(test)]
pub(super) fn owner_valid_compile_contract() {
    fn bind<'media>(
        source: ObservedRecoverySource<'media>,
        validated: IntegrityValidatedExtentChunkFrame<'media>,
    ) {
        let _ = IntegrityAdmittedExtentChunkFrame::bind(source, validated);
    }
    let _ = bind;
}
