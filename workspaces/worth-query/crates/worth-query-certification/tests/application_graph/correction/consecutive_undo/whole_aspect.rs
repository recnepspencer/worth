//! Genuine created-output custody observes the complete installed NumberFacts.
use super::*;
use model::{
    authored_identity_model::{Create, Created, CreatedOutputs, Retired, RetiredOutputs},
    edit::Edit,
    other_edit::OtherEdit,
    query::{Read, Row},
};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

const NAME: &str = "whole-aspect-subject";

fn create(host: &host::Host) -> WorthQueryApplicationCommitReceipt {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let key = "whole-aspect-create".to_owned();
    let outcome = host
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(Create {
            key: "numeric".into(),
            name: NAME.into(),
            payload: 41,
            inverse_target: None,
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("genuine created output: {outcome:?}");
    };
    receipt
}

fn primary(host: &host::Host, value: u64, key: &str) -> WorthQueryApplicationCommitReceipt {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let key = key.to_owned();
    let outcome = host
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(Edit {
            key: NAME.into(),
            value: Value::Primary(value),
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("actual authored field posting: {outcome:?}");
    };
    receipt
}

fn secondary(host: &host::Host, value: u64, key: &str) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let key = key.to_owned();
    let outcome = host
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(OtherEdit {
            key: NAME.into(),
            value: Value::Secondary(value),
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "independent field edit: {outcome:?}"
    );
}

fn read(host: &host::Host) -> Vec<Row> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    host.runtime()
        .request(&principal, &scope)
        .query(Read { key: NAME.into() })
        .execute()
        .unwrap()
        .rows()
        .to_vec()
}

#[test]
fn whole_aspect_support_blocks_field_noops_until_their_performed_inverse() {
    let host = host::install();
    // This ordinary Create binds the complete NumberFacts aspect in its actual
    // native output witness. No caller constructs a Fact or supplies revisions.
    let source = create(&host);
    let _source_pin = pin_current(&host, &source);
    let original = source
        .outputs_of::<CreatedOutputs>()
        .unwrap()
        .entity::<Created>()
        .unwrap()
        .entity_id();
    assert_ne!(original.generation_value(), 0);
    assert_eq!(
        read(&host),
        [Row {
            primary: 41,
            secondary: 3
        }]
    );

    let equal = primary(&host, 41, "whole-aspect-equal-posting");
    let _equal_pin = pin_current(&host, &equal);
    assert!(!equal.is_same_authoritative_commit(&source));
    assert_eq!(
        read(&host),
        [Row {
            primary: 41,
            secondary: 3
        }]
    );
    diverged(&host, &source);

    let undone = reverse(&host, &equal, "whole-aspect-reverse-equal");
    let _undone_pin = pin_current(&host, &undone);
    assert_eq!(
        undone
            .aftermath_causality()
            .unwrap()
            .recorded_source()
            .relational(),
        equal.commit_reference()
    );
    assert_eq!(
        read(&host),
        [Row {
            primary: 41,
            secondary: 3
        }]
    );
    // Change the actual stored content before restoring it through its own
    // recorded inverse. Original revision equality cannot supply this result.
    let changed = primary(&host, 53, "whole-aspect-changed-primary");
    let _changed_pin = pin_current(&host, &changed);
    assert_eq!(
        read(&host),
        [Row {
            primary: 53,
            secondary: 3
        }]
    );
    let restored = reverse(&host, &changed, "whole-aspect-restore-primary");
    let _restored_pin = pin_current(&host, &restored);
    assert_eq!(
        restored
            .aftermath_causality()
            .unwrap()
            .recorded_source()
            .relational(),
        changed.commit_reference()
    );
    assert_eq!(
        read(&host),
        [Row {
            primary: 41,
            secondary: 3
        }]
    );
    let retired = reverse(&host, &source, "whole-aspect-reverse-created");
    assert_eq!(
        retired
            .aftermath_causality()
            .unwrap()
            .recorded_source()
            .relational(),
        source.commit_reference()
    );
    assert_eq!(
        retired
            .outputs_of::<RetiredOutputs>()
            .unwrap()
            .entity::<Retired>()
            .unwrap()
            .entity_id(),
        original
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    match host.runtime().request(&principal, &scope).query(Read { key: NAME.into() }).execute() {
        Err(WorthQueryApplicationRequestQueryDenial::ScopeResolution(cause)) => assert_eq!(cause.kind(), worth_query_host::facade::primary_graph::WorthQueryEntityResolutionDenialKind::Refused(worth_query_host::facade::primary_graph::WorthQueryEntityResolutionRefusalKind::UnknownEntity)),
        Err(other) => panic!("retired subject read failed for another cause: {other:?}"),
        Ok(_) => panic!("the exact original output remained live after its reversal"),
    }
}

#[test]
fn whole_aspect_support_refuses_changed_content_and_independent_write_back() {
    let host = host::install();
    let source = create(&host);
    let _source_pin = pin_current(&host, &source);
    let later = primary(&host, 53, "whole-aspect-later-primary");
    let _later_pin = pin_current(&host, &later);
    let undone = reverse(&host, &later, "whole-aspect-reverse-later");
    let _undone_pin = pin_current(&host, &undone);
    assert_eq!(
        undone
            .aftermath_causality()
            .unwrap()
            .recorded_source()
            .relational(),
        later.commit_reference()
    );
    assert_eq!(
        read(&host),
        [Row {
            primary: 41,
            secondary: 3
        }]
    );

    // The independently changed field is outside the later Primary inverse.
    // Comparing only that primary value would miss the changed whole aspect.
    secondary(&host, 19, "whole-aspect-changed-secondary");
    assert_eq!(
        read(&host),
        [Row {
            primary: 41,
            secondary: 19
        }]
    );
    diverged(&host, &source);
    secondary(&host, 3, "whole-aspect-secondary-write-back");
    assert_eq!(
        read(&host),
        [Row {
            primary: 41,
            secondary: 3
        }]
    );
    diverged(&host, &source);
}
