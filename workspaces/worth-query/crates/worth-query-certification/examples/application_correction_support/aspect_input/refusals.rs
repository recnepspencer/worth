//! Ordinary reads and authoring preserve their distinct installed authority.
use super::super::{
    authentication::*, causal_inspection::capability, host, output_program::OutputProgram,
};
use super::{
    authoring::{Action, AspectEdit},
    courts::perform,
    producer_input,
    program::Root,
    query,
};
use worth_query_host::facade::{
    application_entry::*, primary_graph::*, runtime::ExecutionAllocationPolicy,
};

pub fn incomplete_read_and_mixed_parent_child_writes_refuse_atomically() {
    let app = host::install_with_outputs();
    let _ = perform(
        &app,
        Action::Set {
            bias: 4,
            sibling: 40,
        },
        "aspect-protected-siblings",
        1,
    );
    for (ordinal, action, expected) in [
        (
            0,
            Action::IncompleteRemoval,
            WorthQueryApplicationAttemptDenialKind::UnobservedWrite,
        ),
        (
            1,
            Action::ClearThenWrite,
            WorthQueryApplicationAttemptDenialKind::ConflictingEffectStep,
        ),
        (
            2,
            Action::WriteThenClear,
            WorthQueryApplicationAttemptDenialKind::ConflictingEffectStep,
        ),
        (
            3,
            Action::WrongAspect,
            WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
        ),
    ] {
        let scope = request_scope();
        let principal = authenticate_operator(app.installed_schema(), &scope);
        let request = app.runtime().request(&principal, &scope);
        let key = format!("aspect-refusal-{ordinal}");
        let result = request
            .advanced()
            .mutate(AspectEdit {
                key: "numeric".into(),
                action,
            })
            .without_source()
            .idempotency(&key)
            .execute_performed::<OutputProgram, Root>(
                &app,
                ExecutionAllocationPolicy::SystemAllocation,
            );
        let Err(WorthQueryPerformedMutationExecutionDenial::Mutation(
            WorthQueryApplicationRequestMutationDenial::Handler(
                MutationHandlerExecutionDenial::Handler(cause),
            ),
        )) = result
        else {
            panic!("must refuse the actual candidate construction before any publication")
        };
        let cause = cause
            .downcast::<WorthQueryApplicationAttemptDenial>()
            .unwrap();
        assert_eq!(cause.kind(), expected);
        if ordinal == 0 {
            let coordinates = cause.unobserved_write().expect("exact unread sibling");
            assert_eq!(
                coordinates.locator().field_path().fields()[0].as_str(),
                "Sibling"
            );
            assert_eq!(
                coordinates.locator().aspect().aspect_key().as_str(),
                "InputFacts"
            );
        }
        let read = request
            .query(query::Read {
                key: "numeric".into(),
            })
            .execute_governed(
                capability::Inspect::reference(),
                capability::InspectCausalEvidence::reference(),
                capability::Input::at("numeric"),
            )
            .unwrap();
        assert_eq!(read.rows().len(), 1);
        assert_eq!(
            (
                read.rows()[0].bias,
                read.rows()[0].sibling,
                read.rows()[0].calculated
            ),
            (Some(4), Some(40), 15)
        );
    }
}

pub fn whole_parent_authoring_still_requires_program_delivery() {
    let app = host::install_with_outputs();
    ordinary_data_read_does_not_authorize_inspection(&app);
    let scope = request_scope();
    let principal = authenticate_operator(app.installed_schema(), &scope);
    let request = app.runtime().request(&principal, &scope);
    let key = "aspect-missing-program".to_owned();
    assert!(matches!(
        request
            .advanced()
            .mutate(AspectEdit {
                key: "numeric".into(),
                action: Action::Remove
            })
            .without_source()
            .idempotency(&key)
            .execute(ExecutionAllocationPolicy::SystemAllocation),
        Err(WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired)
    ));
}

fn ordinary_data_read_does_not_authorize_inspection(app: &host::OutputHost) {
    let scope = request_scope();
    let principal = authenticate_operator(app.installed_schema(), &scope);
    let request = app.runtime().request(&principal, &scope);
    let data = request
        .query(producer_input::Read {
            key: "numeric".into(),
        })
        .execute()
        .expect("ordinary producer data uses current READ, not diagnostic authority");
    assert_eq!(data.rows().len(), 1);
    let row = &data.rows()[0];
    assert_eq!((row.input, row.bias, row.sibling), (5, None, None));
    drop(data);
    let Err(WorthQueryApplicationRequestQueryDenial::Admission(denial)) = request
        .query(query::Read {
            key: "numeric".into(),
        })
        .execute()
    else {
        panic!("ordinary data admission cannot grant governed inspection")
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationQueryAdmissionDenialKind::DisclosureGovernanceRequired
    );
    assert_eq!(denial.subject(), "aspect-input-calculation");
    let inspected = request
        .query(query::Read {
            key: "numeric".into(),
        })
        .execute_governed(
            capability::Inspect::reference(),
            capability::InspectCausalEvidence::reference(),
            capability::Input::at("numeric"),
        )
        .expect("the output observation uses its actual governed audience");
    assert_eq!(inspected.rows().len(), 1);
    assert_eq!(inspected.rows()[0].calculated, 0);
}
