//! Exact descriptive union; only installed admission can turn these contents into reads.
use super::super::{
    ApplicationOperationDecisionReadTarget as Target, ApplicationSchemaMember as Member,
};

pub(crate) fn operation_declares_decision_read(
    members: &[Member],
    operation: &str,
    target: &Target,
) -> bool {
    members.iter().any(|member| match member {
        Member::OperationDecisionRead { operation: candidate, target: declared } => candidate == operation && declared == target,
        Member::OperationDecisionReadScope { operation: candidate, scope } if candidate == operation => members.iter().any(|member| matches!(member,
            Member::DecisionReadScope { definition } if definition.identity() == scope && definition.targets().binary_search(target).is_ok())),
        _ => false,
    })
}

/// SDK-owned structural parent reads, derived once for shared compilation just as for inline fields.
pub fn application_decision_read_scope_physical_targets(
    members: &[Member],
    definition: &super::scope::ApplicationDecisionReadScopeMeaning,
) -> Vec<Target> {
    let mut targets = definition.targets().to_vec();
    for target in definition.targets() {
        let Target::Field { entity, aspect, .. } = target else {
            continue;
        };
        if members.iter().any(|member| matches!(member, Member::Field { entity: e, aspect: a, field, .. }
            if e == entity && a == aspect && field == super::super::fragment::record_presence::COLUMN)) {
            targets.push(Target::Field { entity: entity.clone(), aspect: aspect.clone(), field: super::super::fragment::record_presence::COLUMN.to_owned() });
        }
    }
    targets.sort();
    targets.dedup();
    targets
}

pub fn application_operation_decision_read_targets<'a>(
    members: &'a [Member],
    operation: &'a str,
) -> impl Iterator<Item = &'a Target> {
    members.iter().flat_map(move |member| match member {
        Member::OperationDecisionRead {
            operation: candidate,
            target,
        } if candidate == operation => std::slice::from_ref(target),
        Member::OperationDecisionReadScope {
            operation: candidate,
            scope,
        } if candidate == operation => members
            .iter()
            .find_map(|member| match member {
                Member::DecisionReadScope { definition } if definition.identity() == scope => {
                    Some(definition.targets())
                }
                _ => None,
            })
            .unwrap_or(&[]),
        _ => &[],
    })
}

pub fn application_operation_derived_read_relations<'a>(
    members: &'a [Member],
    operation: &'a str,
) -> impl Iterator<Item = &'a str> {
    members
        .iter()
        .flat_map(move |member| match member {
            Member::OperationDerivedRead {
                operation: candidate,
                relation,
            } if candidate == operation => std::slice::from_ref(relation),
            Member::OperationDecisionReadScope {
                operation: candidate,
                scope,
            } if candidate == operation => members
                .iter()
                .find_map(|member| match member {
                    Member::DecisionReadScope { definition } if definition.identity() == scope => {
                        Some(definition.derived_relations())
                    }
                    _ => None,
                })
                .unwrap_or(&[]),
            _ => &[],
        })
        .map(String::as_str)
}
