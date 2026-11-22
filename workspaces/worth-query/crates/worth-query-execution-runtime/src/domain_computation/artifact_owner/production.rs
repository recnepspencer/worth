use std::sync::atomic::{AtomicU64, Ordering};

use std::sync::Arc;

use super::{
    WorthQueryArtifactDenial, WorthQueryArtifactDenialKind, WorthQueryArtifactDisposition,
    WorthQueryArtifactHandleCore, WorthQueryArtifactOccurrenceScope,
    WorthQueryArtifactProductionAdmission, WorthQueryMoveOnlyArtifactHandle,
    WorthQueryPreparedArtifactResource, WorthQueryRuntimeArtifactBinding,
    WorthQueryRuntimeArtifactOwner,
};

static NEXT_ARTIFACT_OCCURRENCE: AtomicU64 = AtomicU64::new(1);

impl WorthQueryMoveOnlyArtifactHandle {
    pub(super) fn register(
        admission: WorthQueryArtifactProductionAdmission,
        prepared: WorthQueryPreparedArtifactResource,
        occurrence_scope: Option<WorthQueryArtifactOccurrenceScope>,
    ) -> Result<Self, WorthQueryArtifactDenial> {
        if let Err(denial) = validate_production(&admission, &prepared) {
            let release = prepared.release();
            return Err(denial.with_rejected_resource_release(
                super::WorthQueryArtifactProviderReleasePosture::from_evidence(release),
            ));
        }
        let occurrence_ordinal = NEXT_ARTIFACT_OCCURRENCE.fetch_add(1, Ordering::Relaxed);
        let authority = admission.authority();
        let holder_stage = authority.stage_identity.clone();
        let owner = WorthQueryRuntimeArtifactOwner::register(
            runtime_artifact_binding(&admission, occurrence_ordinal),
            prepared,
            occurrence_scope,
            authority.registry.lifecycle_snapshot_gate(),
        );
        let handle = Self {
            core: WorthQueryArtifactHandleCore::new_owner(
                owner,
                holder_stage,
                WorthQueryArtifactDisposition::Produced,
            ),
        };
        if let Err(denial) = authority
            .registry
            .register(&handle, authority.production_generation)
        {
            let release = handle.cancel().provider_release();
            return Err(denial.with_rejected_resource_release(release));
        }
        Ok(handle)
    }
}

fn runtime_artifact_binding(
    admission: &WorthQueryArtifactProductionAdmission,
    occurrence_ordinal: u64,
) -> WorthQueryRuntimeArtifactBinding {
    let authority = admission.authority();
    let contract = authority.contract.contract();
    let owner_identity =
        crate::domain_computation::artifact_identity::hash_artifact_identity_parts(&[
            "worth_query_runtime_artifact_owner_v1".into(),
            format!(
                "runtime:{}",
                authority.domain_authority.runtime_authority().as_u64()
            ),
            format!(
                "generation:{}",
                authority
                    .domain_authority
                    .installation_generation()
                    .ordinal()
            ),
            format!("operation:{}", authority.operation_identity),
            format!("binding:{}", authority.binding_identity),
            format!("run:{}", authority.run_identity),
            format!("stage:{}", authority.stage_identity),
            format!(
                "production-generation:{}",
                authority.production_generation.ordinal()
            ),
            format!("contract:{}", contract.identity().as_str()),
            format!("occurrence-ordinal:{occurrence_ordinal}"),
        ]);
    let occurrence_identity =
        crate::domain_computation::artifact_identity::hash_artifact_identity_parts(&[
            "worth_query_artifact_occurrence_v1".into(),
            format!("owner:{owner_identity}"),
            format!("contract:{}", contract.identity().as_str()),
            format!("run:{}", authority.run_identity),
            format!("stage:{}", authority.stage_identity),
        ]);
    WorthQueryRuntimeArtifactBinding {
        contract: Arc::clone(&authority.contract),
        domain_authority: Arc::clone(&authority.domain_authority),
        operation_identity: authority.operation_identity.clone(),
        binding_identity: authority.binding_identity.clone(),
        run_identity: authority.run_identity.clone(),
        producing_stage: authority.stage_identity.clone(),
        basis_identity: authority.basis_identity.clone(),
        provenance_identity: admission.evidence.provenance_identity().to_owned(),
        dependency_identity: admission.evidence.dependency_identity().to_owned(),
        owner_identity,
        occurrence_identity,
    }
}

fn validate_production(
    admission: &WorthQueryArtifactProductionAdmission,
    prepared: &WorthQueryPreparedArtifactResource,
) -> Result<(), WorthQueryArtifactDenial> {
    let authority = admission.authority();
    let contract = authority.contract.contract();
    if !admission
        .authority()
        .domain_authority
        .is_current_installation_generation()
    {
        return Err(production_denial(
            WorthQueryArtifactDenialKind::StaleInstallationGeneration,
            contract,
            "artifact production uses a stale installation generation",
        ));
    }
    if !admission.evidence.is_valid() {
        return Err(production_denial(
            WorthQueryArtifactDenialKind::InvalidProductionEvidence,
            contract,
            "artifact production evidence is incomplete",
        ));
    }
    if prepared.semantic_projection().bytes().is_empty() {
        return Err(production_denial(
            WorthQueryArtifactDenialKind::EmptySemanticProjection,
            contract,
            "provider returned an empty canonical semantic projection",
        ));
    }
    let ownership = contract.ownership();
    if ownership.provider_family() != Some(prepared.provider_family) {
        return Err(production_denial(
            WorthQueryArtifactDenialKind::ProviderFamilyMismatch,
            contract,
            "provider resource family differs from the installed artifact contract",
        ));
    }
    if ownership.payload_owner() != Some(authority.contract.owner()) {
        return Err(production_denial(
            WorthQueryArtifactDenialKind::PayloadOwnerMismatch,
            contract,
            "installed package owner does not own the declared artifact payload",
        ));
    }
    if !contract
        .producer_roles()
        .iter()
        .any(|role| role == &authority.stage_identity)
    {
        return Err(production_denial(
            WorthQueryArtifactDenialKind::ProducerRoleNotInstalled,
            contract,
            "producer stage is not admitted by the installed artifact contract",
        ));
    }
    Ok(())
}

fn production_denial(
    kind: WorthQueryArtifactDenialKind,
    contract: &worth_query_installation::facade::WorthQueryPortableArtifactContract,
    detail: &'static str,
) -> WorthQueryArtifactDenial {
    WorthQueryArtifactDenial::new(kind, Some(contract.family().as_str()), detail)
}
