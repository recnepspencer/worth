use worth_foundational::facade::CanonicalDigestDerivationDenial;

use crate::canonical_digest_derivation::InstallationCanonicalIdentityBasis;
use crate::domain_operation::{
    WorthQueryOperationGraphAccess, WorthQueryOperationGraphParticipation,
};

use super::super::{
    WorthQueryInstalledGraphObligationContract, WorthQueryInstalledGraphObligationResourcePosture,
};

use super::aspect_contract_table::AspectContractTable;
use super::scope_encoding::{encode_read_scope, encode_touch};

pub(super) fn encode_contract<'a>(
    basis: &mut InstallationCanonicalIdentityBasis,
    table: &mut AspectContractTable<'a>,
    index: usize,
    contract: &'a WorthQueryInstalledGraphObligationContract,
) -> Result<(), CanonicalDigestDerivationDenial> {
    let prefix = format!("obligation[{index}]");
    match contract {
        WorthQueryInstalledGraphObligationContract::ManagedIndexRead { meaning } => {
            // The exact installed schema commits the full portable Index
            // recipe/profile. These select its actual Source and Parent views.
            basis.text(format!("{prefix}.kind"), "managed-index-read-v1")?;
            basis.text(
                format!("{prefix}.index"),
                &meaning.definition().parts().name,
            )?;
            basis.text(format!("{prefix}.source-binding"), meaning.source_binding())?;
            basis.digest(
                format!("{prefix}.source-query"),
                *meaning.source_query().digest(),
            )?;
            basis.digest(
                format!("{prefix}.source-read-family"),
                *meaning.source_graph().canonical_planning_basis().digest(),
            )?;
            basis.text(format!("{prefix}.parent-binding"), meaning.parent_binding())?;
            basis.digest(
                format!("{prefix}.parent-query"),
                *meaning.parent_query().digest(),
            )?;
            basis.digest(
                format!("{prefix}.parent-read-family"),
                *meaning.parent_graph().canonical_planning_basis().digest(),
            )?;
        }
        WorthQueryInstalledGraphObligationContract::QueryGraphRead { graph } => {
            basis.text(format!("{prefix}.kind"), "query-graph-read")?;
            basis.digest(format!("{prefix}.graph"), *graph.digest())?;
        }
        WorthQueryInstalledGraphObligationContract::OperationGraphRead { role } => {
            basis.text(format!("{prefix}.kind"), "operation-graph-read")?;
            basis.text(format!("{prefix}.role"), role.role())?;
            match role.participation() {
                WorthQueryOperationGraphParticipation::PrimaryLogicalGraph => {
                    basis.text(format!("{prefix}.participation"), "primary")?;
                }
                WorthQueryOperationGraphParticipation::SeparateAuthority { role } => {
                    basis.text(format!("{prefix}.participation"), "separate-authority")?;
                    basis.text(format!("{prefix}.authority-role"), role)?;
                }
            }
            basis.text(
                format!("{prefix}.access"),
                match role.access() {
                    WorthQueryOperationGraphAccess::Observe => "observe",
                    WorthQueryOperationGraphAccess::Project => "project",
                },
            )?;
            basis.unsigned_usize(
                format!("{prefix}.semantic-read-count"),
                role.read_scopes().len(),
            )?;
            for (read_index, read) in role.read_scopes().iter().enumerate() {
                encode_read_scope(basis, table, &format!("{prefix}.read[{read_index}]"), read)?;
            }
        }
        WorthQueryInstalledGraphObligationContract::PrincipalAuthorization => {
            basis.text(format!("{prefix}.kind"), "principal-authorization")?;
        }
        WorthQueryInstalledGraphObligationContract::AbilityAuthorization { requirements } => {
            basis.text(format!("{prefix}.kind"), "ability-authorization")?;
            basis.unsigned_usize(format!("{prefix}.count"), requirements.len())?;
            for (requirement_index, requirement) in requirements.iter().enumerate() {
                basis.digest(
                    format!("{prefix}.requirement[{requirement_index}]"),
                    *requirement.identity(),
                )?;
            }
        }
        WorthQueryInstalledGraphObligationContract::CapabilityAuthorization { requirements } => {
            basis.text(format!("{prefix}.kind"), "capability-authorization")?;
            basis.unsigned_usize(format!("{prefix}.count"), requirements.len())?;
            for (requirement_index, requirement) in requirements.iter().enumerate() {
                basis.digest(
                    format!("{prefix}.requirement[{requirement_index}]"),
                    *requirement.identity().digest(),
                )?;
            }
        }
        WorthQueryInstalledGraphObligationContract::MutationTouch { contract } => {
            basis.text(format!("{prefix}.kind"), "mutation-touch")?;
            encode_touch(basis, table, &prefix, contract)?;
        }
        WorthQueryInstalledGraphObligationContract::EffectApplication { family } => {
            basis.text(format!("{prefix}.kind"), "effect-application")?;
            basis.text(format!("{prefix}.family"), family.as_str())?;
        }
        WorthQueryInstalledGraphObligationContract::InvariantExecution { requirement } => {
            basis.text(format!("{prefix}.kind"), "invariant-execution")?;
            basis.text(format!("{prefix}.slot"), requirement.slot())?;
            basis.text(format!("{prefix}.family"), requirement.family())?;
            basis.unsigned_u32(format!("{prefix}.version"), requirement.version().get())?;
            basis.text(
                format!("{prefix}.enforcement"),
                requirement.enforcement().as_str(),
            )?;
            basis.text(
                format!("{prefix}.executor-role"),
                requirement.executor_role(),
            )?;
            basis.unsigned_usize(
                format!("{prefix}.state-load-count"),
                requirement.state_load_families().len(),
            )?;
            for (load_index, family) in requirement.state_load_families().iter().enumerate() {
                basis.text(format!("{prefix}.state-load[{load_index}]"), family)?;
            }
            match (requirement.max_state_facts(), requirement.max_work_units()) {
                (Some(state), Some(work)) if state > 0 && work > 0 => {
                    basis.unsigned_usize(format!("{prefix}.maximum-state-facts"), state)?;
                    basis.unsigned_u64(format!("{prefix}.maximum-work-units"), work)?;
                }
                (state, work) => {
                    basis.text(format!("{prefix}.bounds-form"), "optional-v1")?;
                    basis.text(
                        format!("{prefix}.state-presence"),
                        if state.is_some() { "some" } else { "none" },
                    )?;
                    if let Some(state) = state {
                        basis.unsigned_usize(format!("{prefix}.maximum-state-facts"), state)?;
                    }
                    basis.text(
                        format!("{prefix}.work-presence"),
                        if work.is_some() { "some" } else { "none" },
                    )?;
                    if let Some(work) = work {
                        basis.unsigned_u64(format!("{prefix}.maximum-work-units"), work)?;
                    }
                }
            }
        }
    }
    Ok(())
}

pub(super) fn encode_resources(
    basis: &mut InstallationCanonicalIdentityBasis,
    resources: &WorthQueryInstalledGraphObligationResourcePosture,
) -> Result<(), CanonicalDigestDerivationDenial> {
    match resources {
        WorthQueryInstalledGraphObligationResourcePosture::ApplicationInspection {
            meaning,
            maximum_authorization_facts,
        } => {
            basis.text("resources.kind", "application-inspection")?;
            basis.digest("resources.inspection-meaning", *meaning)?;
            basis.unsigned_usize(
                "resources.maximum-authorization-facts",
                *maximum_authorization_facts,
            )?;
        }
        WorthQueryInstalledGraphObligationResourcePosture::ApplicationManagedIndex {
            maximum_source_results,
        } => {
            basis.text("resources.kind", "application-managed-index")?;
            basis.unsigned_usize("resources.maximum-source-results", *maximum_source_results)?;
        }
        WorthQueryInstalledGraphObligationResourcePosture::ApplicationQuery {
            maximum_traversal_depth,
            maximum_result_count,
            maximum_authorization_facts,
        } => {
            basis.text("resources.kind", "application-query")?;
            basis.unsigned_usize(
                "resources.maximum-traversal-depth",
                *maximum_traversal_depth,
            )?;
            basis.unsigned_usize("resources.maximum-result-count", *maximum_result_count)?;
            basis.unsigned_usize(
                "resources.maximum-authorization-facts",
                *maximum_authorization_facts,
            )?;
        }
        WorthQueryInstalledGraphObligationResourcePosture::ApplicationResource {
            maximum_authorization_facts,
        } => {
            basis.text("resources.kind", "application-resource")?;
            basis.unsigned_usize(
                "resources.maximum-authorization-facts",
                *maximum_authorization_facts,
            )?;
        }
        WorthQueryInstalledGraphObligationResourcePosture::ApplicationOperation(contract) => {
            basis.text("resources.kind", "application-operation")?;
            basis.text("resources.contract", contract.canonical_identity())?;
        }
    }
    Ok(())
}
