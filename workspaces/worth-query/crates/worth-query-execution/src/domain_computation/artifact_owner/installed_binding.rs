use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct WorthQueryInstalledWorkflowArtifactContracts {
    input:
        Option<Arc<worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority>>,
    output:
        Option<Arc<worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority>>,
    evidence:
        Option<Arc<worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority>>,
}

impl WorthQueryInstalledWorkflowArtifactContracts {
    pub(crate) const fn empty() -> Self {
        Self {
            input: None,
            output: None,
            evidence: None,
        }
    }

    pub(crate) fn with_output(
        output: Arc<worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority>,
    ) -> Self {
        Self {
            input: None,
            output: Some(output),
            evidence: None,
        }
    }

    pub fn input(
        &self,
    ) -> Option<&Arc<worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority>>
    {
        self.input.as_ref()
    }

    pub fn output(
        &self,
    ) -> Option<&Arc<worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority>>
    {
        self.output.as_ref()
    }

    pub fn evidence(
        &self,
    ) -> Option<&Arc<worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority>>
    {
        self.evidence.as_ref()
    }
}

pub(crate) fn compile_workflow_artifact_contracts(
    owner: &str,
    stages: &[worth_query_installation::facade::WorthQueryPortableWorkflowStage],
    portable_index: &worth_query_installation::facade::WorthQueryInstalledPackageIndex,
) -> BTreeMap<String, WorthQueryInstalledWorkflowArtifactContracts> {
    stages
        .iter()
        .map(|stage| {
            let input = installed_contract(
                owner,
                &stage.semantics().input,
                portable_index,
                "workflow input artifact contract must be installed",
            );
            let output = installed_contract(
                owner,
                &stage.semantics().output,
                portable_index,
                "workflow output artifact contract must be installed",
            );
            let evidence =
                installed_evidence_contract(owner, &stage.semantics().evidence, portable_index);
            (
                stage.identity().to_owned(),
                WorthQueryInstalledWorkflowArtifactContracts {
                    input,
                    output,
                    evidence,
                },
            )
        })
        .collect()
}

fn installed_evidence_contract(
    owner: &str,
    evidence: &worth_query_installation::facade::WorthQueryDomainEvidenceContract,
    portable_index: &worth_query_installation::facade::WorthQueryInstalledPackageIndex,
) -> Option<Arc<worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority>> {
    let worth_query_installation::facade::WorthQueryDomainEvidenceContract::InstalledArtifact(
        reference,
    ) = evidence
    else {
        return None;
    };
    Some(installed_reference(
        owner,
        reference,
        portable_index,
        "workflow evidence artifact contract must be installed",
    ))
}

fn installed_contract(
    owner: &str,
    value: &worth_query_installation::facade::WorthQueryWorkflowValueContract,
    portable_index: &worth_query_installation::facade::WorthQueryInstalledPackageIndex,
    message: &'static str,
) -> Option<Arc<worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority>> {
    let worth_query_installation::facade::WorthQueryWorkflowValueContract::InstalledArtifact(
        reference,
    ) = value
    else {
        return None;
    };
    Some(installed_reference(
        owner,
        reference,
        portable_index,
        message,
    ))
}

fn installed_reference(
    owner: &str,
    reference: &worth_query_installation::facade::WorthQueryArtifactContractReference,
    portable_index: &worth_query_installation::facade::WorthQueryInstalledPackageIndex,
    message: &'static str,
) -> Arc<worth_query_installation::facade::WorthQueryInstalledArtifactContractAuthority> {
    let authority = portable_index
        .artifact_contract(
            owner,
            reference.family().as_str(),
            reference.schema_version(),
            reference.protocol_version(),
        )
        .expect(message);
    portable_index
        .validate_artifact_contract(&authority)
        .expect("newly minted workflow artifact authority must validate");
    Arc::new(authority)
}

mod court_doors_for_test;
