use sha2::{Digest, Sha256};
use worth_store::physical_runtime::ObservedRecoveryArtifact;
use worth_store_physical_format::{
    PageGenerationCell, PhysicalPageLsn, PhysicalRecordFormatDeclaration,
};
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
    pub record_format: PhysicalRecordFormatDeclaration,
    pub page_identity: PageGenerationCell,
    pub slot_count: u16,
    pub free_bytes: u32,
    pub page_lsn: PhysicalPageLsn,
    pub encoded_digest: [u8; 32],
}

pub(crate) fn admit_page_projection(
    observed: &ObservedRecoveryArtifact,
    scope: PhysicalArtifactScope,
    expected_segment: worth_store_physical_format::RecordArtifactFile,
    trace: &mut super::super::RecoveryIntegrityIngressTrace,
) -> Result<PageFrameProjection, RecoveryIntegrityIngressRejection> {
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
        super::super::IntegrityAdmittedRecoveryArtifact::PageFrame(admitted) => {
            Ok(admitted.project(trace.counters_mut()))
        }
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
            record_format: self.validated.record_format(),
            page_identity: self.validated.page_identity(),
            slot_count: self.validated.slot_count(),
            free_bytes: self.validated.free_bytes(),
            page_lsn: self.validated.page_lsn(),
            encoded_digest: Sha256::digest(input.bytes()).into(),
        }
    }

    pub(crate) fn scope(&self) -> worth_store_physical_integrity::PhysicalArtifactScope {
        self.source.scope()
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
