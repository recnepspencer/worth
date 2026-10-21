//! Installed policy, one spent census prefix, and a single frozen extension.
use super::super::schema::current_read::TemporalIntentCurrentReadBinding;
use super::*;
use std::cell::RefCell;
use worth_query_host::facade::{
    declaration::application_query::ApplicationQueryParameterSet,
    primary_graph::{
        WorthQueryApplicationQueryAccessContext,
        WorthQueryApplicationQueryBatchResourceDenial as Resource,
        WorthQueryPrincipalResolutionMode, WorthQueryProductQueryControls,
        WorthQueryRetainedBatchQueryAdmissionDenial,
    },
};

#[test]
fn installed_profile_extends_once_without_resetting_spent_prefix_or_source_custody() {
    let world = CourtroomWorld::publish_with_intent_population("ready", 2);
    let application = &world.application;
    let request = world::request_scope();
    let auth = world::admit_identity_adapter(application.installed_schema());
    let external = block_on(auth.authenticate((), &request)).unwrap();
    let observation = world
        .application
        .with_host_advancement(|phase| {
            application
                .request(&external, &request)
                .retain_read(&phase)
                .unwrap()
        })
        .expect("installed host call opens");
    let security = application
        .with_host_advancement(|phase| observation.select_on(&phase, application).unwrap())
        .expect("installed host call opens");
    let retained = RefCell::new(
        application
            .with_host_advancement(|phase| observation.select_on(&phase, application).unwrap())
            .expect("installed host call opens"),
    );
    let binding = application
        .installed_schema()
        .installed_query_binding::<TemporalIntentCurrentReadBinding>()
        .unwrap();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            security
                .resolve_authenticated_principal(
                    &phase,
                    binding.principal_binding(),
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("installed host call opens");
    let limits = application.resolve_application_query_limits(binding.limits());
    let execute = |identity: &str, batch: &worth_query_host::facade::primary_graph::WorthQueryApplicationQueryBatchAdmission| {
        let scope = world.application.with_host_advancement(|phase| security.resolve_entity(&phase, super::super::schema::IntentIdentityField::reference(), identity.to_owned(), &request, WorthQueryPrincipalResolutionMode::Ordinary).unwrap()).expect("installed host call opens");
        let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
        let plan = world.application.with_host_advancement(|phase| security.admit_retained_application_query_from_selection(&phase, &mut retained.borrow_mut(), binding.query(), &access, ApplicationQueryParameterSet::new(), WorthQueryProductQueryControls::new(limits.maximum_results(), &request), batch)).expect("installed host call opens")?;
        application.with_host_advancement(|phase| application.execute_application_query_one_shot_in_batch(&phase, plan, batch).map_err(|error| match error {
            worth_query_host::facade::primary_graph::WorthQueryApplicationBatchReadDenial::Resource(error) => WorthQueryRetainedBatchQueryAdmissionDenial::Resource(error),
            error => panic!("ordinary read failed: {error:?}"),
        })).expect("installed host call opens")
    };
    let empty = application
        .application_query_batch_from_installed_reads(&[application
            .plan_application_query_batch_reads(&binding, 0)
            .unwrap()])
        .unwrap();
    empty.freeze_planned_reads().unwrap();
    assert!(matches!(
        execute("intent-1", &empty),
        Err(WorthQueryRetainedBatchQueryAdmissionDenial::Resource(
            Resource::UnplannedRead
        ))
    ));

    let first_plan = application
        .plan_application_query_batch_reads(&binding, 1)
        .unwrap();
    let batch = application
        .application_query_batch_from_installed_reads(&[first_plan])
        .unwrap();
    let first = execute("intent-1", &batch).unwrap();

    let second_plan = application
        .plan_application_query_batch_reads(&binding, 1)
        .unwrap();
    application
        .extend_application_query_batch_once(&batch, &[second_plan])
        .unwrap();

    let second = execute("intent-2", &batch).unwrap();
    assert_eq!(
        first.result().receipt().basis_identity(),
        second.result().receipt().basis_identity()
    );

    assert!(matches!(
        execute("intent-1", &batch),
        Err(WorthQueryRetainedBatchQueryAdmissionDenial::Resource(
            Resource::UnplannedRead
        ))
    ));

    assert_eq!(
        application.extend_application_query_batch_once(&batch, &[]),
        Err(Resource::PlanningFrozen)
    );
    let source = first.result().observed_sources()[0].clone();
    drop(first);
    drop(second);

    drop(source);
    drop(retained);
}

#[test]
fn zero_foreign_and_overflow_plans_refuse_without_funding_another_runtime() {
    let world = CourtroomWorld::publish("ready");
    let foreign = CourtroomWorld::publish("ready");
    let binding = world
        .application
        .installed_schema()
        .installed_query_binding::<TemporalIntentCurrentReadBinding>()
        .unwrap();
    let zero = world
        .application
        .plan_application_query_batch_reads(&binding, 1)
        .unwrap();
    let batch = world
        .application
        .application_query_batch_from_installed_reads(&[zero])
        .unwrap();

    assert!(world
        .application
        .plan_application_query_batch_reads(&binding, usize::MAX)
        .is_ok());
    let overflow = world
        .application
        .plan_application_query_batch_reads(&binding, usize::MAX)
        .unwrap();
    assert!(matches!(
        world
            .application
            .extend_application_query_batch_once(&batch, &[overflow]),
        Err(Resource::CounterOverflow)
    ));

    let foreign_plan = foreign
        .application
        .plan_application_query_batch_reads(
            &foreign
                .application
                .installed_schema()
                .installed_query_binding::<TemporalIntentCurrentReadBinding>()
                .unwrap(),
            1,
        )
        .unwrap();
    assert_eq!(
        world
            .application
            .extend_application_query_batch_once(&batch, &[foreign_plan]),
        Err(Resource::ForeignPlan)
    );
    batch.freeze_planned_reads().unwrap();
    assert_eq!(batch.freeze_planned_reads(), Err(Resource::PlanningFrozen));
    let wrong_binding = foreign
        .application
        .installed_schema()
        .installed_query_binding::<TemporalIntentCurrentReadBinding>()
        .unwrap();
    assert!(matches!(
        world
            .application
            .plan_application_query_batch_reads(&wrong_binding, 1),
        Err(Resource::ForeignPlan)
    ));
}
