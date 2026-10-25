use std::collections::BTreeSet;

use worth_query_decl::facade::application_schema::{
    ApplicationOperationDecisionReadTarget, ApplicationOperationProgramTarget,
    ApplicationSchemaMember,
};

pub(super) fn names(
    members: &[ApplicationSchemaMember],
    select: fn(&ApplicationSchemaMember) -> Option<&str>,
) -> BTreeSet<&str> {
    members.iter().filter_map(select).collect()
}

pub(super) fn entity_name(member: &ApplicationSchemaMember) -> Option<&str> {
    match member {
        ApplicationSchemaMember::Entity { entity } => Some(entity),
        _ => None,
    }
}

pub(super) fn relation_name(member: &ApplicationSchemaMember) -> Option<&str> {
    match member {
        ApplicationSchemaMember::Relation { relation, .. } => Some(relation),
        _ => None,
    }
}

pub(super) fn aspect_name(member: &ApplicationSchemaMember) -> Option<&str> {
    match member {
        ApplicationSchemaMember::Aspect { aspect, .. } => Some(aspect),
        _ => None,
    }
}

pub(super) fn field_name(member: &ApplicationSchemaMember) -> Option<&str> {
    match member {
        ApplicationSchemaMember::Field { field, .. } => Some(field),
        _ => None,
    }
}

pub(super) fn operation_name(member: &ApplicationSchemaMember) -> Option<&str> {
    match member {
        ApplicationSchemaMember::Operation { operation, .. } => Some(operation),
        _ => None,
    }
}

pub(super) fn application_query_name(member: &ApplicationSchemaMember) -> Option<&str> {
    match member {
        ApplicationSchemaMember::ApplicationQuery { definition } => Some(definition.name()),
        _ => None,
    }
}

pub(super) fn application_capability_name(member: &ApplicationSchemaMember) -> Option<&str> {
    match member {
        ApplicationSchemaMember::ApplicationCapability { contract } => Some(contract.name()),
        _ => None,
    }
}

pub(super) fn policy_name(member: &ApplicationSchemaMember) -> Option<&str> {
    match member {
        ApplicationSchemaMember::Policy { policy } => Some(policy),
        _ => None,
    }
}

pub(super) fn unit_name(member: &ApplicationSchemaMember) -> Option<&str> {
    match member {
        ApplicationSchemaMember::Unit { unit } => Some(unit),
        _ => None,
    }
}

pub(super) fn effect_name(member: &ApplicationSchemaMember) -> Option<&str> {
    match member {
        ApplicationSchemaMember::Effect { effect, .. } => Some(effect),
        _ => None,
    }
}

pub(super) fn assert_money_program(members: &[ApplicationSchemaMember], operation: &str) {
    let mut expected = vec![
        "create:JournalEntry",
        "create:Posting",
        "emit:AccountActivityEffect",
        "link:JournalPosting:JournalEntry->Posting",
        "link:PostingAccount:Posting->Account",
        "write:Account/AccountState/AccountingRevision",
        "write:JournalEntry/JournalIdentity/JournalIdentityField",
        "write:JournalEntry/JournalIdentity/JournalInstitutionContext",
        "write:JournalEntry/JournalState/JournalPurpose",
        "write:Posting/PostingIdentity/PostingIdentityField",
        "write:Posting/PostingValue/PostingAccountSequence",
        "write:Posting/PostingValue/PostingAmount",
        "write:Posting/PostingValue/Purpose",
    ];
    if operation == "ReverseJournalOperation" {
        expected.push("link:JournalReversal:JournalEntry->JournalEntry");
    }
    assert_program(members, operation, &expected);
}

pub(super) fn assert_program(
    members: &[ApplicationSchemaMember],
    operation: &str,
    expected_targets: &[&str],
) {
    let actual = members
        .iter()
        .filter_map(|member| match member {
            ApplicationSchemaMember::OperationProgram {
                operation: installed,
                target,
            } if installed == operation => Some(program_target(target)),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let expected = expected_targets
        .iter()
        .copied()
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    assert_eq!(actual, expected, "operation program drift: {operation}");
}

pub(super) fn assert_decision_reads(
    members: &[ApplicationSchemaMember],
    operation: &str,
    expected_targets: &[&str],
) {
    let actual = members
        .iter()
        .filter_map(|member| match member {
            ApplicationSchemaMember::OperationDecisionRead {
                operation: installed,
                target,
            } if installed == operation => Some(decision_read_target(target)),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let expected = expected_targets
        .iter()
        .copied()
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        actual, expected,
        "decision-read contract drift: {operation}"
    );
}

fn decision_read_target(target: &ApplicationOperationDecisionReadTarget) -> String {
    match target {
        ApplicationOperationDecisionReadTarget::Entity { entity } => {
            format!("entity:{entity}")
        }
        ApplicationOperationDecisionReadTarget::Field {
            entity,
            aspect,
            field,
        } => format!("field:{entity}/{aspect}/{field}"),
        ApplicationOperationDecisionReadTarget::Relation { relation, from, to } => {
            format!("relation:{relation}:{from}->{to}")
        }
    }
}

fn program_target(target: &ApplicationOperationProgramTarget) -> String {
    match target {
        ApplicationOperationProgramTarget::Create { entity } => format!("create:{entity}"),
        ApplicationOperationProgramTarget::Delete { entity } => format!("delete:{entity}"),
        ApplicationOperationProgramTarget::Write {
            entity,
            aspect,
            field,
        } => format!("write:{entity}/{aspect}/{field}"),
        ApplicationOperationProgramTarget::Link { relation, from, to } => {
            format!("link:{relation}:{from}->{to}")
        }
        ApplicationOperationProgramTarget::Unlink { relation, from, to } => {
            format!("unlink:{relation}:{from}->{to}")
        }
        ApplicationOperationProgramTarget::Emit { effect } => format!("emit:{effect}"),
    }
}

pub(super) fn expected<'a>(values: &'a [&'a str]) -> BTreeSet<&'a str> {
    values.iter().copied().collect()
}
