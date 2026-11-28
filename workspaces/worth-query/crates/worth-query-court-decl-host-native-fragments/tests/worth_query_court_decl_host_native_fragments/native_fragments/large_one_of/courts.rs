//! Installed reads and portable revisions certify an unbounded joined roster.
use super::*;
use std::collections::BTreeMap;

#[test]
fn joined_thirty_two_case_one_of_installs_and_reads_every_case_through_view() {
    let world = World::new();
    macro_rules! assert_case {
        ($($case:ident: $n:literal: $name:literal),+) => { $(
            assert_eq!(world.read($n, Form).unwrap(), Value::$case(Row(1_000 + $n)));
            assert_eq!(world.read($n, $case).unwrap(), Some(Row(1_000 + $n)));
        )+ };
    }
    each_case!(assert_case);
}

#[test]
fn joined_one_of_refuses_two_present_cases_at_native_publication() {
    use worth_relational::facade::transactions::{
        ConflictClass, InvariantViolationFields, TransactionCommitError,
    };

    let Err(denial) = World::try_with_two_cases() else {
        panic!("two cases must refuse publication");
    };
    let WorthQueryConditionalRuntimeInstallationDenialKind::PrimaryGraphPublication(cause) =
        denial.kind()
    else {
        panic!("expected the primary publication cause: {denial:?}");
    };
    let Some(TransactionCommitError::Conflict { error, .. }) = cause.native_commit_denial() else {
        panic!("expected the original Native conflict: {cause:?}");
    };
    assert!(matches!(
        &error.class,
        ConflictClass::InvariantViolation {
            fields: InvariantViolationFields::EntityAspectExclusion {
                required: true,
                active_count: 2,
                ..
            },
            ..
        }
    ));
}

fn decode(name: &str, number: Option<u64>) -> Result<Value, RecordRefusal> {
    let values = number
        .map(|n| {
            (
                "selected.number".to_owned(),
                U64ApplicationValueBinding::encode(&n).unwrap(),
            )
        })
        .into_iter()
        .collect();
    let mut work = |_, _| Ok(());
    let mut admission = |_, _| Ok(());
    <OrderedCases as CaseRows<Bindings, Value>>::decode(
        name,
        &values,
        "selected",
        &mut work,
        &mut admission,
    )
}
#[test]
fn joined_one_of_unknown_name_is_typed_and_selected_case_failure_never_falls_through() {
    let unknown = decode("not_a_case", None).unwrap_err();
    assert!(matches!(unknown.kind(), RecordRefusalKind::UnknownCase));
    assert_eq!(unknown.path(), "selected");
    let selected = decode("case01", Some(0)).unwrap_err();
    let RecordRefusalKind::Constructor(cause) = selected.kind() else {
        panic!("a selected row failure must survive dispatch: {selected:?}")
    };
    assert!(cause.downcast_ref::<ZeroValue>().is_some());
}

fn revisions<S>(
    declaration: ApplicationSchemaDeclaration<S>,
) -> BTreeMap<String, AspectContractRevision> {
    declaration
        .erased()
        .members()
        .iter()
        .filter_map(|member| match member {
            ApplicationSchemaMember::Aspect {
                entity,
                aspect,
                revision,
                ..
            } if entity == "entry" => Some((aspect.clone(), *revision)),
            _ => None,
        })
        .collect()
}
#[test]
fn joined_one_of_group_contract_ignores_case_order_and_join_parentheses() {
    let original = revisions(Fixture::declaration().unwrap());
    let reordered = revisions(Root::<Reordered>::declaration().unwrap());
    assert_eq!(original.len(), 33);
    assert_eq!(original, reordered);
}
