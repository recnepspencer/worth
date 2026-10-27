//! Execution-side consumption of installed application read and program meaning.

use std::collections::BTreeSet;

use worth_foundational::facade::{CanonicalFieldPath, FieldKey};
use worth_query_installation::facade::{
    ApplicationOperationDecisionReadTarget, ApplicationOperationProgramTarget,
    WorthQueryCompiledApplicationOperationContracts, WorthQueryOperationGraphReadContract,
    WorthQueryOperationGraphReadScope, WorthQueryOperationTouchScope,
};

pub(crate) fn graph_reads_admit_target(
    contract: &WorthQueryOperationGraphReadContract,
    target: &ApplicationOperationDecisionReadTarget,
) -> bool {
    installed_read_scope_for_target(contract, target).is_some()
}

pub(crate) fn installed_read_scope_for_target<'contract>(
    contract: &'contract WorthQueryOperationGraphReadContract,
    target: &ApplicationOperationDecisionReadTarget,
) -> Option<&'contract WorthQueryOperationGraphReadScope> {
    contract
        .roles()
        .iter()
        .flat_map(|role| role.read_scopes())
        .find(|scope| read_scope_admits_target(scope, target))
}

pub(crate) fn graph_reads_exactly_match_targets(
    contract: &WorthQueryOperationGraphReadContract,
    targets: &[ApplicationOperationDecisionReadTarget],
) -> bool {
    let unique = targets.iter().collect::<BTreeSet<_>>();
    unique.len() == targets.len()
        && installed_atomic_read_count(contract) == targets.len()
        && targets
            .iter()
            .all(|target| graph_reads_admit_target(contract, target))
}

pub(crate) fn application_contract_admits_program_target(
    contracts: &WorthQueryCompiledApplicationOperationContracts,
    target: &ApplicationOperationProgramTarget,
) -> bool {
    match target {
        ApplicationOperationProgramTarget::Emit { effect } => contracts
            .emissions()
            .emissions()
            .iter()
            .any(|emission| emission.effect() == effect),
        _ => contracts
            .touches()
            .scopes()
            .iter()
            .any(|scope| touch_scope_admits_target(scope, target)),
    }
}

pub(crate) fn application_contract_exactly_matches_program_targets(
    contracts: &WorthQueryCompiledApplicationOperationContracts,
    targets: &[ApplicationOperationProgramTarget],
) -> bool {
    let installed_count =
        contracts.touches().scopes().len() + contracts.emissions().emissions().len();
    installed_count == targets.len()
        && application_contract_contains_program_targets(contracts, targets)
}

pub(crate) fn application_contract_contains_program_targets(
    contracts: &WorthQueryCompiledApplicationOperationContracts,
    targets: &[ApplicationOperationProgramTarget],
) -> bool {
    let unique = targets.iter().collect::<BTreeSet<_>>();
    unique.len() == targets.len()
        && targets
            .iter()
            .all(|target| application_contract_admits_program_target(contracts, target))
}

fn installed_atomic_read_count(contract: &WorthQueryOperationGraphReadContract) -> usize {
    contract
        .roles()
        .iter()
        .flat_map(|role| role.read_scopes())
        .try_fold(0usize, |count, scope| match scope {
            WorthQueryOperationGraphReadScope::Components(_) => None,
            WorthQueryOperationGraphReadScope::Entity(_)
            | WorthQueryOperationGraphReadScope::Relation(_) => count.checked_add(1),
            WorthQueryOperationGraphReadScope::NativeProjection(scope) => {
                (!scope.projection().mask().is_whole_aspect())
                    .then(|| count.checked_add(scope.projection().mask().paths().len()))
                    .flatten()
            }
        })
        .unwrap_or(usize::MAX)
}

fn read_scope_admits_target(
    scope: &WorthQueryOperationGraphReadScope,
    target: &ApplicationOperationDecisionReadTarget,
) -> bool {
    match (scope, target) {
        (
            WorthQueryOperationGraphReadScope::Entity(scope),
            ApplicationOperationDecisionReadTarget::Entity { entity },
        ) => scope.semantic_key() == entity,
        (
            WorthQueryOperationGraphReadScope::NativeProjection(scope),
            ApplicationOperationDecisionReadTarget::Field {
                entity,
                aspect,
                field,
            },
        ) => FieldKey::new(field.clone()).is_some_and(|field| {
            scope.entity().semantic_key() == entity
                && scope.aspect().as_str() == aspect
                && scope
                    .projection()
                    .mask()
                    .paths()
                    .contains(&CanonicalFieldPath::single(field))
        }),
        (
            WorthQueryOperationGraphReadScope::Relation(scope),
            ApplicationOperationDecisionReadTarget::Relation { relation, from, to },
        ) => scope.relation() == relation && scope.from() == from && scope.to() == to,
        _ => false,
    }
}

fn touch_scope_admits_target(
    scope: &WorthQueryOperationTouchScope,
    target: &ApplicationOperationProgramTarget,
) -> bool {
    match (scope, target) {
        (
            WorthQueryOperationTouchScope::CreateEntity(scope),
            ApplicationOperationProgramTarget::Create { entity },
        )
        | (
            WorthQueryOperationTouchScope::DeleteEntity(scope),
            ApplicationOperationProgramTarget::Delete { entity },
        ) => scope.entity() == entity,
        (
            WorthQueryOperationTouchScope::WriteField(scope),
            ApplicationOperationProgramTarget::Write {
                entity,
                aspect,
                field,
            },
        ) => {
            scope.entity() == entity
                && scope.contract().key().as_str() == aspect
                && scope
                    .field_path()
                    .fields()
                    .first()
                    .is_some_and(|field_key| field_key.as_str() == field)
        }
        (
            WorthQueryOperationTouchScope::LinkRelation(scope),
            ApplicationOperationProgramTarget::Link { relation, from, to },
        )
        | (
            WorthQueryOperationTouchScope::UnlinkRelation(scope),
            ApplicationOperationProgramTarget::Unlink { relation, from, to },
        ) => scope.relation() == relation && scope.from() == from && scope.to() == to,
        _ => false,
    }
}
