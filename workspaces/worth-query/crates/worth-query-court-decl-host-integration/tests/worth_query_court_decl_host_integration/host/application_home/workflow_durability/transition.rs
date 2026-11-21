//! Home transitions reuse live adoption's workflow inventory, legality and staging.
use document::workflow::home_transition::{self, TargetProgram};
use super::*;
use document::schema::DocumentRetentionSchema;
use worth_query_host::facade::home_program_transition::QueryHomeWorkflowDispositionDenial;
use worth_query_host::facade::primary_graph::WorthQueryWorkflowDispositions as Choices;
use worth_query_host::facade::{
    WorthQueryApplicationHomeLocation as Location, WorthQueryApplicationHomeRequest,
    WorthQueryHomeProgramTransitionDenial as Denial,
    WorthQueryHomeProgramTransitionOpenPhase as Phase,
};

fn successor(
    location: &Location,
) -> WorthQueryApplicationHomeRequest<'static, DocumentRetentionSchema, TargetProgram> {
    home_transition::successor_request(location)
}
#[test]
fn workflow_home_transition_requires_dispositions_and_persists_retirement_and_cancellation() {
    let location = Location::Directory(super::super::fixture::location());
    let (home, auth) = super::transition_source::open(&location);
    let originals = home_transition::seed_workflows(&home);
    drop(auth);
    close(home);
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    for choices in [None, Some(Choices::carry_compatible())] {
        let transition = successor(&location).with_program_transition(
            super::transition_source::request(&location),
            home_transition::resources(),
            home_transition::plan,
        );
        let transition = match choices {
            Some(choices) => transition.with_workflow_dispositions(choices),
            None => transition,
        };
        let refusal = with_bootstrap_advancement_from_request(execution, |phase| {
            transition.open_in_advancement(&phase)
        })
        .unwrap()
        .err()
        .expect("no executable target vocabulary permits implicit carry");
        match refusal.phase() {
            Phase::Transition {
                home,
                cause:
                    Denial::Workflow(
                        QueryHomeWorkflowDispositionDenial::Required
                        | QueryHomeWorkflowDispositionDenial::Rejected,
                    ),
                ..
            } => assert!(home.home_program_transition_intent().is_none()),
            other => panic!("typed before-intent disposition refusal: {other:?}"),
        }
        drop(refusal);
        let (home, auth) = super::transition_source::open(&location);
        for original in &originals {
            assert!(matches!(
                observation::read(&home, original),
                WorthQueryWorkflowInstanceRead::Live { .. }
            ));
        }
        drop(auth);
        close(home);
    }
    let transition = successor(&location)
        .with_program_transition(
            super::transition_source::request(&location),
            home_transition::resources(),
            home_transition::plan,
        )
        .with_workflow_dispositions(Choices::retire_and_cancel_unperformed());
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        transition.open_in_advancement(&phase).unwrap()
    })
    .unwrap();
    check_cancelled(home, &originals, &execution);
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        successor(&location).open_in_advancement(&phase).unwrap()
    })
    .unwrap();
    check_cancelled(home, &originals, &execution);
}
fn check_cancelled(
    home: worth_query_host::facade::WorthQueryApplicationHome<
        DocumentRetentionSchema,
        TargetProgram,
    >,
    originals: &[WorthQueryWorkflowInstanceDiscoveryIntent<DocumentRetentionSchema>],
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let workflow = runtime::install_workflow_spec_for::<TargetProgram, GovernedDocumentWorkflow>(
        home.runtime().installed_schema(),
        home.runtime().installed_program(),
        workflow_resources(),
    );
    let authentication = install_certification_authentication(home.runtime().installed_schema());
    let home = home
        .retain_workflow_spec(workflow, authentication.signing_owner())
        .unwrap();
    for original in originals {
        match observation::read(&home, original) {
            WorthQueryWorkflowInstanceRead::Historical { observation, .. } => assert_eq!(
                observation.state(),
                WorthQueryWorkflowObservedState::Cancelled
            ),
            _ => panic!("transition cancelled the same original instance"),
        }
    }
    {
        use document::workflow::governed_read::MetadataRead;
        use worth_query_decl::facade::application_program::{
            ApplicationWorkflowDefinitionIdentity, ApplicationWorkflowDefinitionReadTarget,
        };
        let scope = request_scope();
        let principal = authenticate_operator(home.runtime().installed_schema(), &scope);
        let request = home.runtime().request(&principal, &scope);
        let basis = request.retain_current_read().unwrap();
        let target = ApplicationWorkflowDefinitionReadTarget::<GovernedDocumentWorkflow>::new(
            ApplicationWorkflowDefinitionIdentity::new("reviewed-document").unwrap(),
        );
        let found = request
            .at(&basis)
            .discover_workflow_definition::<GovernedDocumentWorkflow, MetadataRead>(
                home.vocabulary(),
                &target,
                observation::input(),
            )
            .unwrap();
        assert!(matches!(
            found.definition(),
            WorthQueryWorkflowDefinitionDiscovery::Retired
        ));
    }
    home.close(execution).unwrap().reclaim();
}
