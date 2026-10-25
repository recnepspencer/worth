//! Caller cancellation during real invariant preparation must not fail a shared row.
use super::installation::{Application, OwnerSchema, ANCHOR};
use super::*;
use crate::facade::application_invariants::*;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_decl::facade::application_query::ApplicationQueryParameterSet;

#[test]
fn cancelling_one_caller_during_producer_commit_keeps_the_other_callers_row_live() {
    let interruption = Interruption::default();
    let app = installation::install_with_interruption(interruption.clone());
    let runtime = &app;
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(None, cancellation.token());
    let external = primary_graph::tests::fixture::authenticate_external(
        runtime.installed_schema(),
        "alice",
        Duration::from_secs(600),
        &scope,
    );
    let (mut survivor, stopped) = runtime
        .with_application_advancement(&scope, |phase| {
            use crate::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
            let execution = phase
                .execution_request_for(&runtime.product_runtime)
                .unwrap();
            let admit = || {
                runtime
                    .admit_required_output_demand::<RegionOutputFamily>(
                        &execution,
                        source(runtime, &phase, &external, &scope),
                        runtime.output_demand_resource_profile().limits(),
                    )
                    .unwrap()
            };
            let mut cancelled = admit();
            let survivor = admit();
            let disclosure = source(runtime, &phase, &external, &scope);
            *interruption.lock().unwrap() = Some(cancellation);
            let stop = runtime
                .advance_program_output_demand(
                    &execution,
                    &phase,
                    &mut cancelled,
                    &external,
                    &scope,
                    runtime.current_world(),
                    disclosure,
                )
                .err()
                .expect("the cancelled producer commit must stop");
            assert!(
                interruption.lock().unwrap().is_none(),
                "the actual invariant prepared during producer commit"
            );
            assert_eq!(
                stop.commit_execution_denial(),
                Some((
                    WorthQueryApplicationCommitDenialStage::InvariantExecution,
                    Err(Control::Cancelled),
                ))
            );
            // A correctly typed local denial alone is insufficient if the shared
            // registry has already retained a row failure: observe the other caller first.
            (survivor, stop)
        })
        .unwrap();
    let fresh = primary_graph::tests::fixture::live_scope();
    let external = primary_graph::tests::fixture::authenticate_external(
        runtime.installed_schema(),
        "alice",
        Duration::from_secs(600),
        &fresh,
    );
    runtime
        .with_application_advancement(&fresh, |phase| {
            let execution = phase
                .execution_request_for(&runtime.product_runtime)
                .unwrap();
            let advanced = runtime
                .advance_program_output_demand(
                    &execution,
                    &phase,
                    &mut survivor,
                    &external,
                    &fresh,
                    runtime.current_world(),
                    source(runtime, &phase, &external, &fresh),
                )
                .expect(
                    "another caller can still claim the shared row after a mid-commit cancellation",
                );
            let WorthQueryOutputDemandAdvance::Settled(settled) = advanced else {
                panic!("the live row must settle by its real producer");
            };
            assert_eq!(settled.producer_contacts_in_this_demand(), 1);
            assert!(
                settled.application_commit_receipt().is_some(),
                "the surviving caller performs a real publication"
            );
        })
        .unwrap();
    assert_eq!(stopped.kind(), WorthQueryOutputDemandDenialKind::Cancelled);
}

fn source(
    runtime: &Application,
    phase: &WorthQueryAdvancementPhase<'_>,
    external: &WorthQueryAuthenticatedExternalPrincipal<OwnerSchema>,
    scope: &WorthQueryRequestScope,
) -> WorthQueryApplicationOutputDemandSource<PlanarQuery, PlanarReadResult> {
    let selected = runtime
        .on_branch(runtime.current_world())
        .select(phase)
        .unwrap();
    let binding = runtime
        .installed_schema()
        .principal_binding(ConsumerPrincipalBinding::reference())
        .unwrap();
    let principal = selected
        .resolve_authenticated_principal(
            phase,
            &binding,
            external,
            scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let entity = selected
        .resolve_entity(
            phase,
            BodyKey::reference(),
            ANCHOR.to_owned(),
            scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &entity);
    let binding = runtime
        .installed_schema()
        .installed_query_binding::<PlanarReadBinding<OwnerSchema>>()
        .unwrap();
    let plan = selected
        .admit_application_query(
            phase,
            binding.query(),
            &access,
            ApplicationQueryParameterSet::new(),
            WorthQueryProductQueryControls::new(binding.limits().maximum_results(), scope),
        )
        .unwrap();
    runtime
        .execute_application_query_one_shot(phase, plan)
        .unwrap()
        .into_admitted_disclosed()
        .into_output_demand_source()
}

pub(super) type Interruption = Arc<Mutex<Option<WorthQueryCancellationSource>>>;
pub(super) struct InterruptingRule {
    pub rule: planar_invariant::PositiveTurnRule<OwnerSchema>,
    pub interruption: Interruption,
}
impl WorthQueryApplicationInvariantRule<OwnerSchema> for InterruptingRule {
    type Scope =
        <planar_invariant::PositiveTurnRule<OwnerSchema> as WorthQueryApplicationInvariantRule<
            OwnerSchema,
        >>::Scope;
    fn supports_checked_preparation(&self) -> bool {
        true
    }
    fn prepare_scope_checked(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, OwnerSchema>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<Self::Scope, WorthQueryApplicationInvariantPreparationError>> {
        let scope = self.rule.prepare_scope_checked(planner, budget);
        // Cancel the real admitted request, after the real rule prepared its scope.
        // The next production safe point decides the native interruption.
        if let Some(source) = self.interruption.lock().unwrap().take() {
            source.cancel();
        }
        scope
    }
    fn prepare_scope(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, OwnerSchema>,
    ) -> Result<Self::Scope, WorthQueryApplicationInvariantPreparationError> {
        self.rule.prepare_scope(planner)
    }
    fn evaluate_checked(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, OwnerSchema>,
        scope: &Self::Scope,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<
        Result<WorthQueryApplicationInvariantVerdict, WorthQueryApplicationInvariantExecutionError>,
    > {
        self.rule.evaluate_checked(context, scope, budget)
    }
    fn evaluate(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, OwnerSchema>,
        scope: &Self::Scope,
    ) -> Result<WorthQueryApplicationInvariantVerdict, WorthQueryApplicationInvariantExecutionError>
    {
        self.rule.evaluate(context, scope)
    }
}
