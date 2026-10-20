//! Actual Query reads and installed producer completion.
use super::installation::{Application, ANCHOR};
use super::*;
use std::{num::NonZeroUsize, sync::Arc, time::Duration};
use worth_query_decl::facade::application_query::ApplicationQueryParameterSet;

pub(super) fn demand<Schema: BirthSchema>(
    runtime: &Application<Schema>,
    required: bool,
) -> Result<Arc<WorthQueryOutputDemandSettlement>, WorthQueryOutputDemandDenial> {
    demand_with_contacts(
        runtime,
        required,
        1,
        WorthQueryOutputSettlementPosture::Performed,
    )
}
pub(super) fn demand_with_contacts<Schema: BirthSchema>(
    runtime: &Application<Schema>,
    required: bool,
    contacts: usize,
    posture: WorthQueryOutputSettlementPosture,
) -> Result<Arc<WorthQueryOutputDemandSettlement>, WorthQueryOutputDemandDenial> {
    demand_expected(runtime, required, contacts, posture, Expected::Immediate)
        .map(|settlement| settlement.expect("ordinary completed journey"))
}
enum Expected {
    Immediate,
    Materialized,
    RequiredAdmission,
}
pub(super) fn materializing_demand(
    runtime: &Application<own_effect::MaterializingSchema>,
) -> Arc<WorthQueryOutputDemandSettlement> {
    demand_expected(
        runtime,
        true,
        2,
        WorthQueryOutputSettlementPosture::Performed,
        Expected::Materialized,
    )
    .unwrap()
    .expect("materialization successor completed")
}
pub(super) fn required_successor_admission(runtime: &Application<own_effect::MaterializingSchema>) {
    assert!(demand_expected(
        runtime,
        true,
        1,
        WorthQueryOutputSettlementPosture::Performed,
        Expected::RequiredAdmission
    )
    .unwrap()
    .is_none());
}
fn demand_expected<Schema: BirthSchema>(
    runtime: &Application<Schema>,
    required: bool,
    contacts: usize,
    posture: WorthQueryOutputSettlementPosture,
    expected: Expected,
) -> Result<Option<Arc<WorthQueryOutputDemandSettlement>>, WorthQueryOutputDemandDenial> {
    let _observation = matches!(expected, Expected::Materialized | Expected::RequiredAdmission).then(
        crate::domain_computation::execution_observation::WorthQueryExecutionObservation::install_on_this_thread);
    let scope = primary_graph::tests::fixture::live_scope();
    let external = primary_graph::tests::fixture::authenticate_external(
        runtime.installed_schema(),
        "alice",
        Duration::from_secs(60),
        &scope,
    );
    let binding = runtime
        .installed_schema()
        .installed_query_binding::<PlanarReadBinding<Schema>>()
        .unwrap();
    runtime
        .with_application_advancement(&scope, |phase| {
            let execution = phase
                .execution_request_for(&runtime.product_runtime)
                .unwrap();
            let branch = runtime.current_world();
            let principal_binding = runtime
                .installed_schema()
                .principal_binding(ConsumerPrincipalBinding::reference())
                .unwrap();
            let selected = runtime.on_branch(branch).select(&phase).unwrap();
            let entity = selected
                .resolve_entity(
                    &phase,
                    BodyKey::reference(),
                    ANCHOR.to_owned(),
                    &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let root = entity.entity_id();
            let read_source = || {
                let selected = runtime.on_branch(branch).select(&phase).unwrap();
                let principal = selected.resolve_authenticated_principal(
                    &phase, &principal_binding, &external, &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary).unwrap();
                let entity = selected.resolve_entity(&phase, BodyKey::reference(), ANCHOR.to_owned(), &scope,
                    WorthQueryPrincipalResolutionMode::Ordinary).unwrap();
                let access = WorthQueryApplicationQueryAccessContext::new(&principal, &entity);
                let plan = selected
                    .admit_application_query(
                        &phase,
                        binding.query(),
                        &access,
                        ApplicationQueryParameterSet::new(),
                        WorthQueryProductQueryControls::new(
                            NonZeroUsize::new(1).unwrap(),
                            Some(NonZeroUsize::new(100_000).unwrap()),
                            &scope,
                        ),
                    )
                    .unwrap();
                runtime
                    .execute_application_query_one_shot(&phase, plan)
                    .unwrap()
                    .into_admitted_disclosed()
                    .into_output_demand_source()
            };
            let mut demand = if required {
                runtime.admit_required_output_demand::<Family>(
                    &execution,
                    read_source(),
                    runtime.output_demand_resource_profile().limits(),
                )
            } else {
                runtime.admit_output_demand::<Family>(
                    &phase,
                    read_source(),
                    runtime.output_demand_resource_profile().limits(),
                )
            }?;
            let disclosure = read_source();
            let mut advanced = runtime.advance_program_output_demand(
                &execution,
                &phase,
                &mut demand,
                &external,
                &scope,
                branch,
                disclosure,
            )?;
            let mut original_roster = None;
            if matches!(expected, Expected::Materialized | Expected::RequiredAdmission) {
                assert!(matches!(advanced, WorthQueryOutputDemandAdvance::Pending), "born-stale output cannot settle current");
                let locator = runtime.primary_provider.graph.layout.field_locator("Body", "SourceNotes", "SourceSelection").unwrap();
                demand.assert_born_stale_absence_for_test(runtime, &phase, &execution, root, locator, |receipt| {
                    original_roster = Some(court::observe_performed_receipt(runtime, &phase, receipt, WorthQueryApplicationOutputPosture::Create));
                }, matches!(expected, Expected::RequiredAdmission).then(|| (read_source(), &external, &scope)));
                if matches!(expected, Expected::RequiredAdmission) { return Ok(None); }
                // This is a new authenticated current disclosure on the SAME demand,
                // only after the exact original born-stale transition was proven.
                advanced = runtime.advance_program_output_demand(
                    &execution, &phase, &mut demand, &external, &scope, branch, read_source(),
                )?;
            }
            let WorthQueryOutputDemandAdvance::Settled(settlement) = advanced else {
                panic!("the synchronous real MODE2 handler must settle")
            };
            if let Some(original) = original_roster {
                demand.assert_preserve_successor_for_test(runtime, root);
                assert_eq!(settlement.producer_identity(), <Preserve<Schema> as WorthQueryApplicationProducerBinding<Schema>>::IDENTITY);
                assert_eq!(court::observe_performed_receipt(runtime, &phase, settlement.application_commit_receipt().expect("real successor receipt"), WorthQueryApplicationOutputPosture::Preserve), original, "all four native IDs/generations and real incidence survive the executed Preserve");
            }
            assert_eq!(settlement.producer_contacts_in_this_demand(), contacts);
            assert_eq!(settlement.posture(), posture);
            assert_eq!(
                settlement.is_stably_reused(),
                matches!(
                    posture,
                    WorthQueryOutputSettlementPosture::StableReused
                        | WorthQueryOutputSettlementPosture::RecoveredStableReused
                )
            );
            Ok(Some(settlement))
        })
        .unwrap()
}
