use sha2::{Digest, Sha256};
use worth_store::physical_runtime::ObservedRecoveryArtifact;
use worth_store_physical_format::PhysicalPageLsn;
use worth_store_physical_integrity::{
    validate_inline_page, IntegrityValidatedPageFrame, PhysicalArtifactScope,
};

use super::super::admission::require_observed_recovery_source;
use super::super::{
    ObservedRecoverySource, RecoveryIntegrityIngressCounters, RecoveryIntegrityIngressRejection,
};

pub(crate) struct IntegrityAdmittedPageFrame<'media> {
    source: ObservedRecoverySource<'media>,
    validated: IntegrityValidatedPageFrame<'media>,
}

pub(crate) struct PageFrameProjection {
    pub page_lsn: PhysicalPageLsn,
    pub encoded_digest: [u8; 32],
}

pub(crate) fn admit_page_projection(
    observed: &ObservedRecoveryArtifact,
    scope: PhysicalArtifactScope,
    expected_segment: worth_store_physical_format::RecordArtifactFile,
    trace: &mut super::super::RecoveryIntegrityIngressTrace,
) -> Result<PageFrameProjection, RecoveryIntegrityIngressRejection> {
    let admitted = admit_page_frame(observed, scope, expected_segment, trace)?;
    Ok(admitted.project(trace.counters_mut()))
}

/// Extract only the exact selected record from the integrity-admitted page.
pub(crate) fn admit_inline_record_payload<'media>(
    observed: &'media ObservedRecoveryArtifact,
    scope: PhysicalArtifactScope,
    expected_segment: worth_store_physical_format::RecordArtifactFile,
    placement: worth_store_physical_format::DurableInlineRecordPlacement,
    trace: &mut super::super::RecoveryIntegrityIngressTrace,
) -> Result<&'media [u8], RecoveryIntegrityIngressRejection> {
    let admitted = admit_page_frame(observed, scope, expected_segment, trace)?;
    let input = admitted
        .source
        .input()
        .map_err(|rejection| trace.reject(scope, rejection))?;
    let record = admitted
        .validated
        .project_record(input, placement)
        .map_err(|_| trace.reject(scope, RecoveryIntegrityIngressRejection::ScopeMismatch))?;
    trace.counters_mut().record_owner_projection();
    trace.counters_mut().record_owner_decoder();
    Ok(&input.bytes()[record.payload_range()])
}

fn admit_page_frame<'media>(
    observed: &'media ObservedRecoveryArtifact,
    scope: PhysicalArtifactScope,
    expected_segment: worth_store_physical_format::RecordArtifactFile,
    trace: &mut super::super::RecoveryIntegrityIngressTrace,
) -> Result<IntegrityAdmittedPageFrame<'media>, RecoveryIntegrityIngressRejection> {
    if observed.artifact()
        != &worth_store::physical_runtime::RecoveryDiscoveryArtifact::Record(expected_segment)
    {
        return Err(trace.reject(scope, RecoveryIntegrityIngressRejection::ScopeMismatch));
    }
    let input = ObservedRecoverySource::complete(observed, scope)
        .input()
        .map_err(|rejection| trace.reject(scope, rejection))?;
    let validation = validate_inline_page(input, scope).0;
    let attempt = super::super::IntegrityAdmittedRecoveryArtifact::bind_page_frame(
        observed,
        scope,
        validation,
        trace.counters_mut(),
    );
    trace.retain(attempt.observation());
    match attempt.into_outcome()? {
        super::super::IntegrityAdmittedRecoveryArtifact::PageFrame(admitted) => Ok(admitted),
        _ => unreachable!("page ingress returns its family-specific admitted variant"),
    }
}

impl<'media> IntegrityAdmittedPageFrame<'media> {
    pub(in crate::integrity_ingress) fn bind(
        source: ObservedRecoverySource<'media>,
        validated: IntegrityValidatedPageFrame<'media>,
    ) -> Result<Self, RecoveryIntegrityIngressRejection> {
        require_observed_recovery_source(&source, validated.scope(), |input| {
            validated.matches_input(input)
        })?;
        Ok(Self { source, validated })
    }

    pub(crate) fn project(
        &self,
        counters: &mut RecoveryIntegrityIngressCounters,
    ) -> PageFrameProjection {
        counters.record_owner_projection();
        counters.record_owner_decoder();
        let input = self
            .source
            .input()
            .expect("an admitted page retains its exact C.4 observation");
        PageFrameProjection {
            page_lsn: self.validated.page_lsn(),
            encoded_digest: Sha256::digest(input.bytes()).into(),
        }
    }
}

#[cfg(test)]
pub(super) fn owner_valid_compile_contract() {
    fn bind<'media>(
        source: ObservedRecoverySource<'media>,
        validated: IntegrityValidatedPageFrame<'media>,
    ) {
        let _ = IntegrityAdmittedPageFrame::bind(source, validated);
    }
    let _ = bind;
}
