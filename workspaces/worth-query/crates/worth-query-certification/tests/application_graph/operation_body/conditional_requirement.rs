//! Authored Foundation expressions drive collection guidance and actual admission.
use super::{
    draft::{DraftInput, DraftIntent},
    program, request_scope,
    requirement_control::UnlinkIntent,
    schema::*,
};
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome,
        WorthQueryApplicationProgramMutationPreparation as Preparation,
        WorthQueryApplicationRequestExt,
    },
    primary_graph::{
        WorthQueryApplicationUncommitted, WorthQueryOperationBodyDenial,
        WorthQueryOperationBodyRequirementStatus as Status,
    },
    runtime::ExecutionAllocationPolicy,
};
type App<const MODE: u64> =
    worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<
        DocumentRetentionSchema<MODE>,
        program::BodyProgram<MODE>,
    >;
type Projection<const MODE: u64> =
    worth_query_host::facade::primary_graph::WorthQueryApplicationInvariantProjectionAuthority<
        DocumentRetentionSchema<MODE>,
    >;

fn guidance<const MODE: u64>(app: &App<MODE>, projection: &Projection<MODE>) -> Status {
    let scope = request_scope();
    let principal = super::authenticate_operator(app.runtime().installed_schema(), &scope);
    let assessed = app
        .request(&principal, &scope)
        .mutate(DraftIntent::<MODE, RequireDraftNote>::new(""))
        .assess_current_body_requirements(app, projection)
        .expect("authorized native assessment");
    assert_eq!(assessed.findings().len(), 1);
    let finding = &assessed.findings()[0];
    assert_eq!(
        finding.identity(),
        Some(if MODE == 28 {
            "nonempty-note"
        } else {
            "members-minimum"
        })
    );
    assert_eq!(finding.field().is_none(), MODE != 28);
    finding.status()
}

fn invoke<const MODE: u64>(
    app: &App<MODE>,
    key: u64,
) -> Outcome<WorthQueryOperationBodyDenial, ()> {
    let scope = request_scope();
    let principal = super::authenticate_operator(app.runtime().installed_schema(), &scope);
    app.request(&principal, &scope)
        .mutate(DraftIntent::<MODE, RequireDraftNote>::new(""))
        .without_source()
        .idempotency(&key)
        .execute_in_program(app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
}

fn unlink<const MODE: u64>(app: &App<MODE>, key: u64) {
    let scope = request_scope();
    let principal = super::authenticate_operator(app.runtime().installed_schema(), &scope);
    assert!(matches!(
        app.request(&principal, &scope)
            .mutate(UnlinkIntent::<MODE>(DraftInput {
                identity: super::DOCUMENT_IDENTITY.to_owned(),
                note: String::new()
            }))
            .without_source()
            .idempotency(&key)
            .execute_in_program(app, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap(),
        Outcome::Committed { .. }
    ));
}

#[test]
fn authored_minimum_changes_guidance_and_admission_with_fixed_consumers() {
    use worth_query_host::facade::declaration::application_schema::ApplicationSchemaMember as Member;
    let without_changed_body = |members: &[Member]| {
        members.iter().filter(|member| !matches!(member, Member::OperationBody { operation, .. } if operation == "RequireDraftNote")).cloned().collect::<Vec<_>>()
    };
    let first_schema = program::schema::<25>();
    let second_schema = program::schema::<26>();
    assert_eq!(
        without_changed_body(first_schema.erased().members()),
        without_changed_body(second_schema.erased().members())
    );
    let (first, first_projection) = super::install_with_projection::<25>();
    let (second, second_projection) = super::install_with_projection::<26>();
    assert_ne!(
        first.installed_program().revision(),
        second.installed_program().revision()
    );
    assert_eq!(guidance(&first, &first_projection), Status::Met);
    assert_eq!(guidance(&second, &second_projection), Status::Unmet);
    assert!(matches!(invoke(&first, 501), Outcome::Committed { .. }));
    assert_eq!(super::draft_query::read_audit_source(&first), 12);
    assert!(
        matches!(invoke(&second, 501), Outcome::DomainDenied(WorthQueryOperationBodyDenial::DeclaredRequirementUnmet { identity }) if identity == "members-minimum")
    );
    assert_eq!(super::draft_query::read_audit_source(&second), 7);
}

#[test]
fn unlinking_required_member_stales_without_the_target_write() {
    let (app, projection) = super::install_with_projection::<25>();
    assert_eq!(guidance(&app, &projection), Status::Met);
    let scope = request_scope();
    let principal = super::authenticate_operator(app.runtime().installed_schema(), &scope);
    let key = 502;
    let mut request = app
        .request(&principal, &scope)
        .mutate(DraftIntent::<25, RequireDraftNote>::new(""))
        .without_source()
        .idempotency(&key);
    let candidate = match request
        .prepare_in_program(&app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
    {
        Preparation::Prepared(candidate) => candidate,
        Preparation::Settled(_) => panic!("fresh candidate"),
    };
    unlink(&app, 503);
    assert!(matches!(
        candidate.commit(ExecutionAllocationPolicy::SystemAllocation),
        Outcome::Commit(WorthQueryApplicationUncommitted::Stale(_))
    ));
    assert_eq!(super::draft_query::read_audit_source(&app), 7);
    assert_eq!(guidance(&app, &projection), Status::Unmet);
}

#[test]
fn inapplicable_requirement_does_not_retain_satisfaction_only_adjacency() {
    let (app, projection) = super::install_with_projection::<27>();
    assert_eq!(guidance(&app, &projection), Status::NotApplicable);
    let scope = request_scope();
    let principal = super::authenticate_operator(app.runtime().installed_schema(), &scope);
    let key = 504;
    let mut request = app
        .request(&principal, &scope)
        .mutate(DraftIntent::<27, RequireDraftNote>::new(""))
        .without_source()
        .idempotency(&key);
    let candidate = match request
        .prepare_in_program(&app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
    {
        Preparation::Prepared(candidate) => candidate,
        Preparation::Settled(_) => panic!("fresh candidate"),
    };
    unlink(&app, 505);
    assert!(matches!(
        candidate.commit(ExecutionAllocationPolicy::SystemAllocation),
        Outcome::Committed { .. }
    ));
    assert_eq!(super::draft_query::read_audit_source(&app), 12);
    assert_eq!(guidance(&app, &projection), Status::NotApplicable);
}

#[path = "conditional_requirement/portable.rs"]
mod portable;

#[test]
fn runtime_expression_failure_retains_native_denial_and_cannot_write() {
    use worth_query_host::facade::{
        application_entry::{
            WorthQueryApplicationRequestMutationDenial as RequestDenial,
            WorthQueryCurrentBodyRequirementAssessmentDenial as AssessmentDenial,
        },
        primary_graph::{
            HandlerExecutionDenial, MutationHandlerExecutionDenial,
            WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
            WorthQueryOperationBodyAssessmentDenial as NativeDenial,
        },
    };
    let assert_expression_error = |handler: HandlerExecutionDenial| {
        let cause = handler
            .downcast::<WorthQueryApplicationAttemptDenial>()
            .expect("native attempt cause retained");
        assert_eq!(
            cause.kind(),
            WorthQueryApplicationAttemptDenialKind::OperationBodyRequirementExpressionDenied
        );
        assert!(matches!(
            cause.expression().unwrap().detail(),
            worth_foundational::expression_api::ExpressionDenialDetail::DivisionByZero
        ));
    };
    let (app, projection) = super::install_with_projection::<29>();
    let scope = request_scope();
    let principal = super::authenticate_operator(app.runtime().installed_schema(), &scope);
    let assessment = app
        .request(&principal, &scope)
        .mutate(DraftIntent::<29, RequireDraftNote>::new(""))
        .assess_current_body_requirements(&app, &projection);
    match assessment {
        Err(AssessmentDenial::Native(NativeDenial::Observation(cause))) => {
            assert_expression_error(cause)
        }
        _ => panic!("expression error must remain unavailable, never a finding"),
    }
    let invocation = app
        .request(&principal, &scope)
        .mutate(DraftIntent::<29, RequireDraftNote>::new(""))
        .without_source()
        .idempotency(&512)
        .execute_in_program(&app, ExecutionAllocationPolicy::SystemAllocation);
    match invocation {
        Err(RequestDenial::Handler(MutationHandlerExecutionDenial::Handler(cause))) => {
            assert_expression_error(cause)
        }
        other => panic!("expression failure cannot admit effects: {other:?}"),
    }
    assert_eq!(super::draft_query::read_audit_source(&app), 7);
}

#[test]
fn native_optional_scalar_expression_keeps_empty_text_distinct_from_absence() {
    let (app, projection) = super::install_with_projection::<28>();
    assert_eq!(super::draft_query::read_note(&app), Some(String::new()));
    assert_eq!(guidance(&app, &projection), Status::Unmet);
    assert!(matches!(
        invoke(&app, 508),
        Outcome::DomainDenied(WorthQueryOperationBodyDenial::DeclaredRequirementUnmet { .. })
    ));
    let scope = request_scope();
    let principal = super::authenticate_operator(app.runtime().installed_schema(), &scope);
    assert!(matches!(
        app.request(&principal, &scope)
            .mutate(DraftIntent::<28, SetDraftNote>::new("complete"))
            .without_source()
            .idempotency(&509)
            .execute_in_program(&app, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap(),
        Outcome::Committed { .. }
    ));
    assert_eq!(guidance(&app, &projection), Status::Met);
    assert!(matches!(invoke(&app, 510), Outcome::Committed { .. }));
    assert!(matches!(
        app.request(&principal, &scope)
            .mutate(DraftIntent::<28, ClearDraftNote>::new(""))
            .without_source()
            .idempotency(&511)
            .execute_in_program(&app, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap(),
        Outcome::Committed { .. }
    ));
    assert_eq!(super::draft_query::read_note(&app), None);
    assert_eq!(guidance(&app, &projection), Status::NotApplicable);
}
