//! A real typed successor and real hold under refused frame storage.
//! Its issued interest, query source, Box and Vec all own production custody.
//! Execution metadata is dormant fixture transport; no producer can execute.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::{
    WorthQueryProducerApplicability, WorthQueryProducerLifecyclePosture,
    WorthQuerySelectedApplicationProducer,
};
use crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandInterest;
use crate::domain_computation::primary_graph::tests::fixture::{
    self, retained_output_capacity::RetainedFamily as Family, IdentityExecutionSchema as Schema,
};
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;

impl WorthQueryOutputDemandInterest {
    pub(in crate::domain_computation::primary_graph) fn source_for_typed_frame_test(
    ) -> crate::domain_computation::primary_graph::WorthQueryObservedSource<
        FamilySourceQuery<Schema, Family>,
    > {
        observed_account_source()
    }

    pub(in crate::domain_computation::primary_graph) fn real_typed_frame_for_registry_test(
        self,
        source: crate::domain_computation::primary_graph::WorthQueryObservedSource<
            FamilySourceQuery<Schema, Family>,
        >,
    ) -> impl FnOnce() {
        let registry = self.registry_for_typed_frame_test();
        let limits = crate::domain_computation::execution_runtime::WorthQueryOutputDemandResourceProfile::standard().limits();
        let mut paid = admission();
        let observed_source = registry
            .retain_readmission_source(source, limits, None, &mut paid)
            .unwrap()
            .without_row();
        let demand = WorthQueryAdmittedOutputDemand::<Schema, Family> {
            runtime_authority: observed_source.runtime_authority,
            schema_binding: observed_source.schema_binding.clone(),
            selected: WorthQuerySelectedApplicationProducer {
                identity: "lifetime".into(), applicability: WorthQueryProducerApplicability::new("retained", WorthQueryProducerLifecyclePosture::Preserve),
                exact_retained_output: false, retained_idempotency_key: None,
                retained_output_binding: None, reuses_live_output_only: false,
            },
            installed_entry: InstalledProducerProvider::lifetime_fixture(),
            observed_source, limits,
            producer_contacts_in_this_demand: 0, checkpoint_readmission_work_units: 0,
            checkpoint_readmission_work_bound: 0, checkpoint_readmission_charged_preparation_bytes: 0,
            settled: false, admission_kind: crate::domain_computation::primary_graph::application_output_demand::DemandAdmissionKind::Required,
            retained_program_basis: None, progression_provenance: Default::default(),
            required_continuations: Default::default(), unpublished_selected_checkpoint: None,
            interest: Some(self), settled_at_observation: None, terminal_caller_denial: None,
        };
        let bytes = std::mem::size_of::<TypedRequiredSuccessor<Schema, Family>>();
        let capacity = registry
            .reserve_required_continuation_capacity(bytes, bytes, &mut paid)
            .unwrap();
        let progress = RequiredFreshProgress {
            contacts: super::super::contacts::ContactAttribution::Unowned,
            outcome: Some(RequiredFreshOutcome::Advanced),
            successor: Box::new(TypedRequiredSuccessor {
                demand: Some(demand),
                pending_claim: None,
                pending_provenance: None,
                predecessor: None,
            }),
            capacity,
        };
        let mut continuations = RequiredContinuations::default();
        let slot = continuations.prepare_slot(&registry, &mut paid).unwrap();
        paid.charge_external_work(slot.installation_work()).unwrap();
        slot.push(progress);
        move || continuations.hold_unfinished(&registry)
    }
}
fn admission() -> InvalidationEditAdmission {
    InvalidationEditAdmission::new(worth_relational::facade::mvcc::CompanionPreflightBudget {
        maximum_work_visits: Some(1_000_000),
        maximum_preparation_bytes: None,
    })
}
fn observed_account_source() -> crate::domain_computation::primary_graph::WorthQueryObservedSource<
    FamilySourceQuery<Schema, Family>,
> {
    let world = fixture::installed_authorization_world(true);
    let request = fixture::live_scope();
    let external = world.authenticate("alice", std::time::Duration::from_secs(60), &request);
    world
        .application
        .with_host_advancement(|phase| {
            let product = world.selected_product(&phase.test_host_request());
            let principal = product
        .resolve_authenticated_principal(
            &phase,
            &world.binding,
            &external,
            &request,
            crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
            let account = product
        .resolve_entity(
            &phase,
            fixture::AccountStatus::reference(),
            "open".to_owned(),
            &request,
            crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
            let query = world
                .application
                .installed_schema()
                .installed_query_binding::<fixture::TestAccountSourceBinding>()
                .unwrap()
                .into_query();
            let access =
        crate::domain_computation::primary_graph::WorthQueryApplicationQueryAccessContext::new(
            &principal, &account,
        );
            let plan = product
                .admit_application_query(
                    &phase,
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new()
                        .bind(fixture::status_parameter(), "open".to_owned())
                        .unwrap(),
                    crate::domain_computation::primary_graph::WorthQueryProductQueryControls::new(
                        std::num::NonZeroUsize::new(10).unwrap(),
                        Some(std::num::NonZeroUsize::new(10_000).unwrap()),
                        &request,
                    ),
                )
                .unwrap();
            world
                .application
                .execute_application_query_one_shot(&phase, plan)
                .unwrap()
                .observed_sources()[0]
                .clone()
        })
        .expect("the typed frame source owns its host request")
}
