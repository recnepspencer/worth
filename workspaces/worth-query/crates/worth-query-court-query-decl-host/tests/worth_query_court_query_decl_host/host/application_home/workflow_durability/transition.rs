//! Home transitions reuse live adoption's workflow inventory, legality and staging.
use super::target_program::TargetProgram;
use super::*;
use document::schema::DocumentRetentionSchema;
use worth_foundational::facade::AspectValue;
use worth_query_host::facade::home_program_transition::QueryHomeWorkflowDispositionDenial;
use worth_query_host::facade::primary_graph::WorthQueryWorkflowDispositions as Choices;
use worth_query_host::facade::{
    WorthQueryApplicationHomeLocation as Location, WorthQueryApplicationHomeRequest,
    WorthQueryHomeProgramTransitionDenial as Denial,
    WorthQueryHomeProgramTransitionOpenPhase as Phase, WorthQueryHomeProgramTransitionPlan as Plan,
    WorthQueryHomeProgramTransitionStep as Step,
};
use worth_relational::facade::{indexes::DerivedIndexMaintenanceBudget, transactions::*};

fn successor(
    location: &Location,
) -> WorthQueryApplicationHomeRequest<'static, DocumentRetentionSchema, TargetProgram> {
    WorthQueryApplicationHomeRequest::new(home_location(location), document::programs::validate::<TargetProgram>(),
        worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster::new()
            .support(document::programs::validate::<super::transition_source::Program>())
            .support(document::actor_generation::validated_actor_program())
            .support(document::model_lifecycle::validated_model_lifecycle_program()),
        DocumentRetentionSchema::declaration().unwrap(), ((),),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        document::host::host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        |_, _, _| panic!("the successor cannot initialize an existing Home"))
}
fn resources() -> DerivedIndexMaintenanceBudget {
    DerivedIndexMaintenanceBudget {
        maximum_work_units: Some(1_000_000),
        maximum_cold_record_slots: Some(100_000),
        maximum_derived_rows: Some(100_000),
    }
}
fn plan(
    source: &worth_query_host::facade::home_program_transition::WorthQueryHomeProgramTransitionReadSource<'_, '_>,
    target: &worth_query_host::facade::home_program_transition::WorthQueryHomeProgramTransitionTarget<'_>,
) -> Result<Plan, worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial>
{
    let rows = source.entities("Document", 100_000)?;
    let field = target
        .planned_field(
            "Document",
            document::schema::DocumentRetentionField::at("DocumentRetentionField").aspect(),
            "DocumentRetentionField",
        )
        .unwrap();
    let steps = [8, 9, 10]
        .into_iter()
        .map(|value| {
            let mut batch = WorkerIntentBatch::new("transition-document-retention");
            for row in rows.records() {
                batch = batch.push(MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                    UpdateEntityFieldsIntent {
                        entity_id: row.entity_id,
                        fields: AspectFieldPatch::from_locator(
                            field.clone(),
                            AspectValue::UInt64(value),
                        ),
                    },
                )));
            }
            Step {
                batches: vec![batch],
            }
        })
        .collect();
    Ok(Plan {
        target: 1,
        schema_transition: None,
        steps,
    })
}
#[test]
fn workflow_home_transition_requires_dispositions_and_persists_retirement_and_cancellation() {
    for proposed in [false, true] {
        let location = Location::Directory(super::super::fixture::location());
        let (home, auth) = super::transition_source::open(&location);
        let (definition, target) = publish(&home);
        let (instance, original) = start(&home, definition);
        if proposed {
            assert!(matches!(
                propose(&home, instance, 1003, &target),
                WorkflowProposalOutcome::Published(_)
            ));
        }
        drop(auth);
        close(home);
        let lease = checkpoint_capture::lease(256 << 20);
        let execution = worth_execution::ExecutionRequest::leased(&lease);
        for choices in [None, Some(Choices::carry_compatible())] {
            let transition = successor(&location).with_program_transition(
                super::transition_source::request(&location),
                resources(),
                plan,
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
            assert!(matches!(
                observation::read(&home, &original),
                WorthQueryWorkflowInstanceRead::Live { .. }
            ));
            drop(auth);
            close(home);
        }
        let transition = successor(&location)
            .with_program_transition(
                super::transition_source::request(&location),
                resources(),
                plan,
            )
            .with_workflow_dispositions(Choices::retire_and_cancel_unperformed());
        let home = with_bootstrap_advancement_from_request(execution, |phase| {
            transition.open_in_advancement(&phase).unwrap()
        })
        .unwrap();
        check_cancelled(home, &original, &execution);
        let home = with_bootstrap_advancement_from_request(execution, |phase| {
            successor(&location).open_in_advancement(&phase).unwrap()
        })
        .unwrap();
        check_cancelled(home, &original, &execution);
    }
}
fn check_cancelled(
    home: worth_query_host::facade::WorthQueryApplicationHome<
        DocumentRetentionSchema,
        TargetProgram,
    >,
    original: &WorthQueryWorkflowInstanceDiscoveryIntent<DocumentRetentionSchema>,
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
    match observation::read(&home, original) {
        WorthQueryWorkflowInstanceRead::Historical { observation, .. } => assert_eq!(
            observation.state(),
            WorthQueryWorkflowObservedState::Cancelled
        ),
        _ => panic!("transition cancelled the same original instance"),
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
