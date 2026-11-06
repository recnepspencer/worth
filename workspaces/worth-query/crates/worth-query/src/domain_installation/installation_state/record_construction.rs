use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;

use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::runtime::WorthQueryRuntimeAuthorityIdentity;

use super::{
    WorthQueryDomainInstallationConstructionCounters, WorthQueryDomainInstallationGeneration,
    WorthQueryDomainInstallationGenerationLease, WorthQueryDomainInstallationReceipt,
    WorthQueryDomainInstalledDefinitionCounts, WorthQueryInstalledDomainArtifact,
    WorthQueryInstalledDomainAuthority, WorthQueryInstalledDomainExecutionIndex,
};

pub(super) struct WorthQueryInstalledDomainRecord {
    pub(super) artifact: WorthQueryInstalledDomainArtifact,
    pub(super) authority: Arc<WorthQueryInstalledDomainAuthority>,
    pub(super) receipt: WorthQueryDomainInstallationReceipt,
}

pub(super) fn construct_installed_domain_records(
    artifacts: Vec<WorthQueryInstalledDomainArtifact>,
    runtime_authority: WorthQueryRuntimeAuthorityIdentity,
    generation: WorthQueryDomainInstallationGeneration,
    generation_lease: &WorthQueryDomainInstallationGenerationLease,
    execution_index: &WorthQueryInstalledDomainExecutionIndex,
    portable_index: &worth_query_installation::facade::WorthQueryInstalledPackageIndex,
) -> (Vec<WorthQueryInstalledDomainRecord>, HashMap<TypeId, usize>) {
    let mut records_by_marker_type = HashMap::new();
    let records = artifacts
        .into_iter()
        .enumerate()
        .map(|(record_index, artifact)| {
            records_by_marker_type.insert(artifact.marker_type, record_index);
            construct_installed_domain_record(
                artifact,
                runtime_authority,
                generation,
                generation_lease,
                execution_index,
                portable_index,
            )
        })
        .collect();
    (records, records_by_marker_type)
}

fn construct_installed_domain_record(
    artifact: WorthQueryInstalledDomainArtifact,
    runtime_authority: WorthQueryRuntimeAuthorityIdentity,
    generation: WorthQueryDomainInstallationGeneration,
    generation_lease: &WorthQueryDomainInstallationGenerationLease,
    execution_index: &WorthQueryInstalledDomainExecutionIndex,
    portable_index: &worth_query_installation::facade::WorthQueryInstalledPackageIndex,
) -> WorthQueryInstalledDomainRecord {
    let installation_identity = installation_identity(
        runtime_authority,
        generation,
        &artifact,
        execution_index.identity(),
    );
    let receipt = installation_receipt(&artifact, generation, installation_identity.clone());
    let authority = installed_authority(
        &artifact,
        runtime_authority,
        generation,
        generation_lease,
        installation_identity,
        execution_index,
        portable_index,
    );
    WorthQueryInstalledDomainRecord {
        artifact,
        authority,
        receipt,
    }
}

fn installation_receipt(
    artifact: &WorthQueryInstalledDomainArtifact,
    generation: WorthQueryDomainInstallationGeneration,
    installation_identity: WorthQueryEvidenceIdentity,
) -> WorthQueryDomainInstallationReceipt {
    WorthQueryDomainInstallationReceipt::new(
        artifact.domain_owner.clone(),
        artifact.semantic_version,
        artifact.package_identity.clone(),
        installation_identity,
        generation,
        WorthQueryDomainInstallationConstructionCounters::for_package(
            artifact.invariant_definitions.len(),
            artifact.graph_read_operations.len(),
            artifact.declaration_families.len(),
            artifact.domain_operations.len(),
            artifact.contribution_policy.len(),
        ),
        installed_definition_counts(artifact),
    )
}

fn installed_definition_counts(
    artifact: &WorthQueryInstalledDomainArtifact,
) -> WorthQueryDomainInstalledDefinitionCounts {
    WorthQueryDomainInstalledDefinitionCounts::new([
        artifact.required_capabilities.len(),
        artifact.required_configuration.len(),
        artifact.operating_requirements.len(),
        artifact.invariant_definitions.len(),
        artifact.graph_read_operations.len(),
        artifact.declaration_families.len(),
        artifact.domain_operations.len(),
        artifact.contribution_policy.len(),
    ])
}

fn installed_authority(
    artifact: &WorthQueryInstalledDomainArtifact,
    runtime_authority: WorthQueryRuntimeAuthorityIdentity,
    generation: WorthQueryDomainInstallationGeneration,
    generation_lease: &WorthQueryDomainInstallationGenerationLease,
    installation_identity: WorthQueryEvidenceIdentity,
    execution_index: &WorthQueryInstalledDomainExecutionIndex,
    portable_index: &worth_query_installation::facade::WorthQueryInstalledPackageIndex,
) -> Arc<WorthQueryInstalledDomainAuthority> {
    let semantics = execution_index
        .domain_semantics(artifact.marker_type)
        .expect("every installed artifact has one derived semantic index");
    let portable_authority = portable_index
        .domain(&artifact.domain_owner)
        .expect("every installed artifact has portable package authority");
    portable_index
        .validate(&portable_authority)
        .expect("newly minted portable package authority must validate");
    Arc::new(WorthQueryInstalledDomainAuthority::new(
        super::super::WorthQueryInstalledDomainAuthorityInputs {
            runtime_authority,
            generation,
            generation_lease: generation_lease.clone(),
            marker_type: artifact.marker_type,
            domain_key: artifact.marker_domain_key,
            display_name: artifact.marker_display_name,
            domain_owner: artifact.domain_owner.clone(),
            package_identity: artifact.package_identity.clone(),
            installation_identity,
            support_snapshot: artifact.support_snapshot.clone(),
            required_capabilities: artifact.required_capabilities.clone(),
            required_configuration: artifact.required_configuration.clone(),
            operating_requirements: artifact.operating_requirements.clone(),
            semantics,
            portable_authority,
        },
    ))
}

fn installation_identity(
    runtime_authority: WorthQueryRuntimeAuthorityIdentity,
    generation: WorthQueryDomainInstallationGeneration,
    artifact: &WorthQueryInstalledDomainArtifact,
    execution_index_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::DomainInstallation)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("package"),
                artifact.package_identity.evidence_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("admission"),
                &artifact.admission_identity,
            )
            .field_value(
                WorthQueryEvidenceTag::new("runtime_authority"),
                runtime_authority.as_u64().to_string(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("generation"),
                generation.ordinal().to_string(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("execution_index"),
                execution_index_identity,
            )
            .seal()?,
    )
}
