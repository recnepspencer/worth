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
    let observation = application
        .request(&external, &request)
        .retain_read()
        .unwrap();
    let security = observation.select_on(application).unwrap();
    let retained = RefCell::new(observation.select_on(application).unwrap());
    let binding = application
        .installed_schema()
        .installed_query_binding::<TemporalIntentCurrentReadBinding>()
        .unwrap();
    let principal = security
        .resolve_authenticated_principal(
            binding.principal_binding(),
            &external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let limits = application.resolve_application_query_limits(binding.limits());
    let execute = |identity: &str, batch: &worth_query_host::facade::primary_graph::WorthQueryApplicationQueryBatchAdmission| {
        let scope = security.resolve_entity(super::super::schema::IntentIdentityField::reference(), identity.to_owned(), &request, WorthQueryPrincipalResolutionMode::Ordinary).unwrap();
        let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
        let plan = security.admit_retained_application_query_from_selection(&mut retained.borrow_mut(), binding.query(), &access, ApplicationQueryParameterSet::new(), WorthQueryProductQueryControls::new(limits.maximum_results(), limits.maximum_work(), &request), batch)?;
        application.execute_application_query_one_shot_in_batch(plan, batch).map_err(|error| match error {
            worth_query_host::facade::primary_graph::WorthQueryApplicationBatchReadDenial::Resource(error) => WorthQueryRetainedBatchQueryAdmissionDenial::Resource(error),
            error => panic!("ordinary read failed: {error:?}"),
        })
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
    assert_eq!(empty.observe().retained_bytes(), 0);
    assert_eq!(empty.observe().read_work_units(), 0);
    let first_plan = application
        .plan_application_query_batch_reads(&binding, 1)
        .unwrap();
    let batch = application
        .application_query_batch_from_installed_reads(&[first_plan])
        .unwrap();
    let first = execute("intent-1", &batch).unwrap();
    let prefix = batch.observe();
    assert!(prefix.read_work_units() > 0);
    assert!(prefix.retained_bytes() > 0);
    let second_plan = application
        .plan_application_query_batch_reads(&binding, 1)
        .unwrap();
    application
        .extend_application_query_batch_once(&batch, &[second_plan])
        .unwrap();
    assert_eq!(
        batch.observe(),
        prefix,
        "extension cannot reset a spent prefix or old claims"
    );
    let second = execute("intent-2", &batch).unwrap();
    assert_eq!(
        first.result().receipt().basis_identity(),
        second.result().receipt().basis_identity()
    );
    assert_eq!(
        batch.observe().read_work_units(),
        first.result().receipt().work().total_work_units()
            + second.result().receipt().work().total_work_units()
    );
    let previous = batch.observe();
    assert!(matches!(
        execute("intent-1", &batch),
        Err(WorthQueryRetainedBatchQueryAdmissionDenial::Resource(
            Resource::UnplannedRead
        ))
    ));
    assert_eq!(batch.observe(), previous);
    assert_eq!(
        application.extend_application_query_batch_once(&batch, &[]),
        Err(Resource::PlanningFrozen)
    );
    let source = first.result().observed_sources()[0].clone();
    drop(first);
    drop(second);
    let retained_bytes = batch.observe().retained_bytes();
    assert!(
        retained_bytes > 0,
        "genuine source keeps its original custody"
    );
    drop(source);
    drop(retained);
    assert_eq!(batch.observe().retained_bytes(), 0);
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
        .plan_application_query_batch_reads(&binding, 0)
        .unwrap();
    let batch = world
        .application
        .application_query_batch_from_installed_reads(&[zero])
        .unwrap();
    assert_eq!(batch.observe().read_work_units(), 0);
    assert_eq!(batch.observe().retained_bytes(), 0);
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
    assert_eq!(batch.observe().read_work_units(), 0);
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
