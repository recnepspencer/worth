//! The real three-step Host transition from the seeded A Home to B: the
//! balance becomes 8 under A, 9 with B's target body, then 10 under B.
use super::*;
use crate::facade::host::application_home::{
    self as fixture,
    schema::{Account, AccountIdentity, HomeRecords},
};
use worth_query_decl::facade::application_schema::fragment::FieldSelection;
use worth_query_execution::facade::{
    application_home_integration::{
        WorthQueryHomeProgramTransitionReadSource, WorthQueryHomeProgramTransitionTarget,
    },
    primary_graph::WorthQueryPrimaryGraphInstallationDenial,
};
use worth_query_host::facade::{
    home_program_transition::*, WorthQueryHomeProgramTransition,
    WorthQueryHomeProgramTransitionPlan as Plan, WorthQueryHomeProgramTransitionStep as Step,
};

/// Finite maintenance resources every transition must declare.
pub fn resources() -> DerivedIndexMaintenanceBudget {
    DerivedIndexMaintenanceBudget {
        maximum_work_units: Some(1_000_000),
        maximum_cold_record_slots: Some(100_000),
        maximum_derived_rows: Some(100_000),
    }
}

fn step(id: EntityId, field: AspectFieldLocator, value: u64) -> Step {
    Step {
        batches: vec![
            WorkerIntentBatch::new("authored-balance").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: id,
                    fields: AspectFieldPatch::from_locator(field, AspectValue::UInt64(value)),
                }),
            )),
        ],
    }
}

/// Open B over the existing A Home through the Host transition. `planned`
/// counts author runs; A may never initialize here.
pub fn transition_request(
    location: std::path::PathBuf,
    build: ApplicationBuildIdentity,
    contacts: Arc<AtomicUsize>,
    planned: Arc<AtomicUsize>,
) -> WorthQueryApplicationHomeRequest<
    'static,
    SuccessorSchema,
    ExtendedProgram,
    WorthQueryHomeProgramTransition<
        'static,
        fixture::schema::HomeIdentitySchema,
        fixture::HomeProgram,
        impl FnOnce(
            &WorthQueryHomeProgramTransitionReadSource<'_, '_>,
            &WorthQueryHomeProgramTransitionTarget<'_>,
        ) -> Result<Plan, WorthQueryPrimaryGraphInstallationDenial>,
    >,
> {
    authored_transition_request(location, build, contacts, planned, 8)
}

/// The same transition whose first step writes `first` instead of 8: a
/// recomputed plan with a changed body (TransitionPlanDrift on resume).
pub fn authored_transition_request(
    location: std::path::PathBuf,
    build: ApplicationBuildIdentity,
    contacts: Arc<AtomicUsize>,
    planned: Arc<AtomicUsize>,
    first: u64,
) -> WorthQueryApplicationHomeRequest<
    'static,
    SuccessorSchema,
    ExtendedProgram,
    WorthQueryHomeProgramTransition<
        'static,
        fixture::schema::HomeIdentitySchema,
        fixture::HomeProgram,
        impl FnOnce(
            &WorthQueryHomeProgramTransitionReadSource<'_, '_>,
            &WorthQueryHomeProgramTransitionTarget<'_>,
        ) -> Result<Plan, WorthQueryPrimaryGraphInstallationDenial>,
    >,
> {
    let predecessor =
        fixture::request_with_initializer(location.clone(), build, contacts.clone(), |_, _, _| {
            panic!("A cannot initialize during transition")
        });
    request(location, build, contacts).with_program_transition(
        predecessor,
        resources(),
        move |source, target| {
            planned.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let rows = source.entities("Account", 10_000)?;
            let [row] = rows.records() else {
                panic!("the author reads the original seeded A row");
            };
            let before = source
                .planned_field(
                    "Account",
                    FieldSelection::<HomeRecords, Account, AccountIdentity, u64>::at("Balance")
                        .aspect(),
                    "Balance",
                )
                .unwrap();
            let after = target
                .planned_field(
                    "Account",
                    FieldSelection::<HomeRecords, Account, AccountIdentity, u64>::at("Balance")
                        .aspect(),
                    "Balance",
                )
                .unwrap();
            let mut target_body = step(row.entity_id, after.clone(), 9);
            target_body.batches.push(body(target));
            Ok(Plan {
                target: 1,
                schema_transition: Some((schema_change(source, target), None)),
                steps: vec![
                    step(row.entity_id, before, first),
                    target_body,
                    step(row.entity_id, after, 10),
                ],
            })
        },
    )
}
