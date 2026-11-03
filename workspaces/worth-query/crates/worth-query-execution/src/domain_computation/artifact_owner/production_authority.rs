use std::sync::Arc;

use super::{
    artifact_authority_denial_detail, WorthQueryArtifactAuthorityMatch, WorthQueryArtifactDenial,
    WorthQueryArtifactDenialKind, WorthQueryArtifactOccurrenceScope,
    WorthQueryArtifactProductionAdmission, WorthQueryArtifactProductionEvidence,
    WorthQueryArtifactProductionGeneration, WorthQueryArtifactProviderReleasePosture,
    WorthQueryArtifactProviderResource, WorthQueryGuardedArtifactResource,
    WorthQueryMoveOnlyArtifactHandle, WorthQueryWorkflowArtifactRegistry,
};

pub struct WorthQueryArtifactProductionAuthority {
    pub(super) contract:
        Arc<worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority>,
    pub(super) domain_authority:
        Arc<crate::domain_computation::WorthQueryInstalledDomainExecutionAuthority>,
    pub(super) operation_identity: String,
    pub(super) binding_identity: String,
    pub(super) run_identity: String,
    pub(super) stage_identity: String,
    pub(super) basis_identity: String,
    pub(super) registry: Arc<WorthQueryWorkflowArtifactRegistry>,
    pub(super) production_generation: WorthQueryArtifactProductionGeneration,
}

pub(crate) struct WorthQueryArtifactProductionAuthorityParts {
    pub(crate) contract:
        Arc<worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority>,
    pub(crate) domain_authority:
        Arc<crate::domain_computation::WorthQueryInstalledDomainExecutionAuthority>,
    pub(crate) operation_identity: String,
    pub(crate) binding_identity: String,
    pub(crate) run_identity: String,
    pub(crate) stage_identity: String,
    pub(crate) basis_identity: String,
    pub(crate) registry: Arc<WorthQueryWorkflowArtifactRegistry>,
    pub(crate) production_generation: WorthQueryArtifactProductionGeneration,
}

impl WorthQueryArtifactProductionAuthority {
    pub(crate) fn mint(parts: WorthQueryArtifactProductionAuthorityParts) -> Arc<Self> {
        Arc::new(Self {
            contract: parts.contract,
            domain_authority: parts.domain_authority,
            operation_identity: parts.operation_identity,
            binding_identity: parts.binding_identity,
            run_identity: parts.run_identity,
            stage_identity: parts.stage_identity,
            basis_identity: parts.basis_identity,
            registry: parts.registry,
            production_generation: parts.production_generation,
        })
    }

    pub fn admit(
        authority: &Arc<Self>,
        evidence: WorthQueryArtifactProductionEvidence,
    ) -> WorthQueryArtifactProductionAdmission {
        WorthQueryArtifactProductionAdmission::mint(Arc::clone(authority), evidence)
    }

    pub fn validate_admission(
        expected: &Arc<Self>,
        admission: &WorthQueryArtifactProductionAdmission,
    ) -> Result<(), WorthQueryArtifactDenial> {
        let candidate = admission.authority();
        if Arc::ptr_eq(expected, candidate) {
            return Ok(());
        }
        let kind = WorthQueryArtifactAuthorityMatch {
            runtime: expected.domain_authority.runtime_authority()
                == candidate.domain_authority.runtime_authority(),
            generation: expected.domain_authority.installation_generation()
                == candidate.domain_authority.installation_generation()
                && candidate
                    .domain_authority
                    .is_current_installation_generation(),
            operation: expected.operation_identity == candidate.operation_identity
                && expected.binding_identity == candidate.binding_identity,
            run: expected.run_identity == candidate.run_identity,
            stage: expected.stage_identity == candidate.stage_identity,
            basis: expected.basis_identity == candidate.basis_identity,
            payload_owner: expected.contract.owner() == candidate.contract.owner(),
            contract: expected.contract.contract().identity()
                == candidate.contract.contract().identity(),
        }
        .denial_kind()
        .unwrap_or(WorthQueryArtifactDenialKind::StageExecutionMismatch);
        Err(WorthQueryArtifactDenial::new(
            kind,
            Some(candidate.contract.contract().family().as_str()),
            if kind == WorthQueryArtifactDenialKind::StageExecutionMismatch {
                "artifact production admission was not minted for this stage execution"
            } else {
                artifact_authority_denial_detail(kind)
            },
        ))
    }

    pub fn register_exact<R: WorthQueryArtifactProviderResource>(
        expected: &Arc<Self>,
        admission: WorthQueryArtifactProductionAdmission,
        resource: R,
    ) -> Result<WorthQueryMoveOnlyArtifactHandle, WorthQueryArtifactDenial> {
        let guarded = WorthQueryGuardedArtifactResource::new(resource);
        if let Err(denial) = Self::validate_admission(expected, &admission) {
            let release = guarded.release();
            return Err(denial.with_rejected_resource_release(
                WorthQueryArtifactProviderReleasePosture::from_evidence(release),
            ));
        }
        if let Err(denial) = admission
            .authority()
            .registry
            .admit_registration(admission.authority().production_generation)
        {
            let release = guarded.release();
            return Err(denial.with_rejected_resource_release(
                WorthQueryArtifactProviderReleasePosture::from_evidence(release),
            ));
        }
        WorthQueryMoveOnlyArtifactHandle::register(admission, guarded.prepare(), None)
    }

    pub(crate) fn register_tracked<R: WorthQueryArtifactProviderResource>(
        expected: &Arc<Self>,
        admission: WorthQueryArtifactProductionAdmission,
        resource: R,
        occurrence_scope: WorthQueryArtifactOccurrenceScope,
    ) -> Result<WorthQueryMoveOnlyArtifactHandle, WorthQueryArtifactDenial> {
        let guarded = WorthQueryGuardedArtifactResource::new(resource);
        if let Err(denial) = Self::validate_admission(expected, &admission) {
            let release = guarded.release();
            return Err(denial.with_rejected_resource_release(
                WorthQueryArtifactProviderReleasePosture::from_evidence(release),
            ));
        }
        if let Err(denial) = admission
            .authority()
            .registry
            .admit_registration(admission.authority().production_generation)
        {
            let release = guarded.release();
            return Err(denial.with_rejected_resource_release(
                WorthQueryArtifactProviderReleasePosture::from_evidence(release),
            ));
        }
        WorthQueryMoveOnlyArtifactHandle::register(
            admission,
            guarded.prepare(),
            Some(occurrence_scope),
        )
    }
}
