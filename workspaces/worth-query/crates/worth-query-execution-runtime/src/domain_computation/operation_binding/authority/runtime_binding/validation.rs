use std::collections::{BTreeMap, BTreeSet};

use worth_query_installation::facade::{
    WorthQueryInstalledDomainOperationAuthority, WorthQueryInstalledGraphParticipationAuthority,
    WorthQueryInstalledPackageAuthority, WorthQueryOperationGraphParticipation,
    WorthQueryOperationWorkflowContract,
};

use super::{direct_topology, workflow_stage_resources};
use crate::domain_computation::artifact_owner::WorthQueryInstalledWorkflowArtifactContracts;
use crate::domain_computation::execution_runtime::{
    WorthQueryExecutionInstallationAuthority, WorthQueryExecutionRuntime,
};
use crate::domain_computation::operation_binding::authority::topology::resource_topology;
use crate::domain_computation::operation_binding::{
    WorthQueryExecutionCommitPosture, WorthQueryExecutionOperationBindingDenial,
    WorthQueryInstalledOperationExecutionSupport,
};

pub(super) fn validate_operation_and_dependencies(
    runtime: &WorthQueryExecutionRuntime,
    installation_authority: &WorthQueryExecutionInstallationAuthority,
    operation: &WorthQueryInstalledDomainOperationAuthority,
    graph_authorities: &[&WorthQueryInstalledGraphParticipationAuthority],
    required_domains: &[(&str, &WorthQueryInstalledPackageAuthority)],
    commit_posture: WorthQueryExecutionCommitPosture,
    installed_support: &WorthQueryInstalledOperationExecutionSupport,
    workflow_artifact_contracts: &BTreeMap<String, WorthQueryInstalledWorkflowArtifactContracts>,
) -> Result<(), WorthQueryExecutionOperationBindingDenial> {
    if !installation_authority.belongs_to(runtime) {
        return Err(WorthQueryExecutionOperationBindingDenial::ForeignInstallationAuthority);
    }
    runtime
        .installed_packages()
        .validate_domain_operation(operation)
        .map_err(|denial| {
            WorthQueryExecutionOperationBindingDenial::InstalledOperation(denial.kind())
        })?;
    validate_graph_topology(runtime, operation, graph_authorities)?;
    validate_required_domain_closure(operation, required_domains)?;
    for (_, authority) in required_domains {
        runtime
            .installed_packages()
            .validate(authority)
            .map_err(|denial| {
                WorthQueryExecutionOperationBindingDenial::RequiredDomain(denial.kind())
            })?;
    }
    validate_installed_support(
        operation,
        graph_authorities,
        commit_posture,
        installed_support,
        workflow_artifact_contracts,
    )
}

fn validate_required_domain_closure(
    operation: &WorthQueryInstalledDomainOperationAuthority,
    required_domains: &[(&str, &WorthQueryInstalledPackageAuthority)],
) -> Result<(), WorthQueryExecutionOperationBindingDenial> {
    let declared = operation
        .definition()
        .semantics()
        .required_domains
        .iter()
        .map(|role| role.as_str())
        .collect::<BTreeSet<_>>();
    let supplied = required_domains
        .iter()
        .map(|(role, _)| *role)
        .collect::<BTreeSet<_>>();
    if supplied == declared && supplied.len() == required_domains.len() {
        Ok(())
    } else {
        Err(WorthQueryExecutionOperationBindingDenial::RequiredDomainTopology)
    }
}

fn validate_installed_support(
    operation: &WorthQueryInstalledDomainOperationAuthority,
    graph_authorities: &[&WorthQueryInstalledGraphParticipationAuthority],
    commit_posture: WorthQueryExecutionCommitPosture,
    support: &WorthQueryInstalledOperationExecutionSupport,
    workflow_artifact_contracts: &BTreeMap<String, WorthQueryInstalledWorkflowArtifactContracts>,
) -> Result<(), WorthQueryExecutionOperationBindingDenial> {
    let semantics = operation.definition().semantics();
    match (&semantics.workflow, support) {
        (
            WorthQueryOperationWorkflowContract::NotRequired,
            WorthQueryInstalledOperationExecutionSupport::Direct {
                operation: operation_support,
            },
        ) if direct_topology(operation, graph_authorities, commit_posture)
            .admits(operation_support)
            && operation_support.parallel_admission().is_none() =>
        {
            Ok(())
        }
        (
            WorthQueryOperationWorkflowContract::Declared(workflow),
            WorthQueryInstalledOperationExecutionSupport::Workflow {
                operation: operation_support,
                stages,
            },
        ) if workflow_support_is_exact(
            operation,
            graph_authorities,
            commit_posture,
            workflow,
            operation_support,
            stages,
            workflow_artifact_contracts,
        ) =>
        {
            Ok(())
        }
        _ => Err(WorthQueryExecutionOperationBindingDenial::InstalledSupportTopology),
    }
}

fn workflow_support_is_exact(
    operation: &WorthQueryInstalledDomainOperationAuthority,
    graph_authorities: &[&WorthQueryInstalledGraphParticipationAuthority],
    commit_posture: WorthQueryExecutionCommitPosture,
    workflow: &worth_query_installation::facade::WorthQueryPortableWorkflowDefinition,
    operation_support: &worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot,
    stages: &BTreeMap<
        String,
        worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot,
    >,
    artifact_contracts: &BTreeMap<String, WorthQueryInstalledWorkflowArtifactContracts>,
) -> bool {
    let operation_topology = resource_topology(
        operation
            .definition()
            .semantics()
            .conditional_nodes
            .iter()
            .map(|node| format!("operation:{}", node.identity())),
        &[],
        std::iter::empty(),
        std::iter::empty(),
        WorthQueryExecutionCommitPosture::ReadOnly,
    );
    let parallel_is_exact =
        operation_support.parallel_admission().is_some() == workflow.has_parallel_frontier();
    let expected_stages = workflow_stage_resources(
        operation,
        graph_authorities,
        commit_posture,
        artifact_contracts,
    )
    .expect("declared workflow has stage resource authorities");
    operation_topology.admits(operation_support)
        && parallel_is_exact
        && stages.len() == expected_stages.len()
        && stages.iter().all(|(stage, support)| {
            expected_stages.get(stage.as_str()).is_some_and(|expected| {
                expected.topology.admits(support) && support.parallel_admission().is_none()
            })
        })
}

fn validate_graph_topology(
    runtime: &WorthQueryExecutionRuntime,
    operation: &WorthQueryInstalledDomainOperationAuthority,
    graph_authorities: &[&WorthQueryInstalledGraphParticipationAuthority],
) -> Result<(), WorthQueryExecutionOperationBindingDenial> {
    if graph_authorities.iter().any(|authority| {
        authority.runtime_ordinal() != runtime.installed_packages().runtime_ordinal()
    }) {
        return Err(WorthQueryExecutionOperationBindingDenial::ForeignGraphRuntime);
    }
    let roles = operation
        .definition()
        .semantics()
        .graph_reads
        .domain_roles();
    let declared = roles
        .iter()
        .map(|read| read.role.as_str())
        .collect::<BTreeSet<_>>();
    let required = roles
        .iter()
        .filter(|read| {
            matches!(
                read.participation,
                WorthQueryOperationGraphParticipation::SeparateAuthority { .. }
            )
        })
        .map(|read| read.role.as_str())
        .collect::<BTreeSet<_>>();
    let supplied = graph_authorities
        .iter()
        .map(|authority| authority.role())
        .collect::<BTreeSet<_>>();
    if graph_authorities.len() == supplied.len()
        && required.is_subset(&supplied)
        && supplied.is_subset(&declared)
    {
        Ok(())
    } else {
        Err(WorthQueryExecutionOperationBindingDenial::InstalledGraphTopology)
    }
}
