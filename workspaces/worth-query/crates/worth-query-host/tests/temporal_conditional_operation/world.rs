use std::sync::Arc;
use std::time::{Duration, Instant};

use worth_query_host::facade::{
    admission, declaration::application_query::ApplicationQueryParameterSet, domain, primary_graph,
    runtime,
};

use super::adapters::{
    ClockController, ClockSource, ContactCounters, CourtroomClock, IdentityAdapter,
    IntentProjector, Invoker, PanicController, Predicate, PrincipalSource, ReplacementPredicate,
};
use super::contract::{self, TemporalReadyNode};
use super::schema::*;

#[path = "world/amendment.rs"]
mod amendment;
#[path = "world/combined_amendment.rs"]
mod combined_amendment;
#[path = "world/resources.rs"]
mod resources;
#[path = "world/scaled_amendment.rs"]
mod scaled_amendment;
#[path = "world/security.rs"]
mod security;
#[path = "world/seed.rs"]
mod seed;

#[allow(dead_code)] // Test targets use different subsets of the shared world fixture.
pub struct CourtroomWorld {
    pub application: primary_graph::WorthQueryPrimaryGraphApplicationRuntime<TemporalHostSchema>,
    pub clock: primary_graph::WorthQueryConditionalClockHandle<
        TemporalHostSchema,
        TemporalReadyNode,
        CourtroomClock,
    >,
    invariant:
        Arc<primary_graph::WorthQueryApplicationInvariantProjectionAuthority<TemporalHostSchema>>,
    pub clock_control: ClockController,
    pub predicate_panic: PanicController,
    pub reconstruction_panic: PanicController,
    pub preconditions_panic: PanicController,
    pub contacts: ContactCounters,
    pub installation: Arc<domain::WorthQueryInstalledPackageIndex>,
    amendment_ordinal: u8,
}

#[allow(dead_code)] // Test targets use different constructors from the shared world fixture.
impl CourtroomWorld {
    pub fn reinstall_conditional_runtime(
        &mut self,
    ) -> Result<
        primary_graph::WorthQueryConditionalRuntimeReinstallationReceipt,
        primary_graph::WorthQueryConditionalRuntimeInstallationDenial,
    > {
        let branch = self.application.current_world();
        self.application.reinstall_conditional_runtime(branch)
    }

    pub fn conditional_clock(
        &self,
    ) -> primary_graph::WorthQueryConditionalClockObservationPort<
        '_,
        TemporalHostSchema,
        TemporalReadyNode,
        CourtroomClock,
    > {
        self.application
            .on_branch(self.application.current_world())
            .select()
            .unwrap()
            .conditional_clock(&self.clock)
            .unwrap()
    }

    pub fn publish(gate: &str) -> Self {
        Self::publish_with_unrelated_rows(gate, 0)
    }

    pub fn publish_with_unrelated_rows(gate: &str, unrelated_row_count: usize) -> Self {
        let contacts = ContactCounters::default();
        let (predicate, predicate_panic) = Predicate::controlled(contacts.clone());
        Self::publish_with_predicate(
            gate,
            unrelated_row_count,
            contacts,
            predicate,
            predicate_panic,
            None,
            None,
            None,
            1,
            true,
        )
    }

    pub fn publish_with_active_snapshot_limit(gate: &str, maximum: usize) -> Self {
        let contacts = ContactCounters::default();
        let (predicate, predicate_panic) = Predicate::controlled(contacts.clone());
        Self::publish_with_predicate(
            gate,
            0,
            contacts,
            predicate,
            predicate_panic,
            Some(maximum),
            None,
            None,
            1,
            true,
        )
    }

    pub fn publish_with_world_history_limit(gate: &str, maximum: u64) -> Self {
        let contacts = ContactCounters::default();
        let (predicate, predicate_panic) = Predicate::controlled(contacts.clone());
        Self::publish_with_predicate(
            gate,
            0,
            contacts,
            predicate,
            predicate_panic,
            None,
            Some(maximum),
            None,
            1,
            true,
        )
    }

    #[allow(dead_code)] // Shared by the certification crate's cost targets.
    pub fn publish_with_graph_work_limit(gate: &str, maximum: usize) -> Self {
        let contacts = ContactCounters::default();
        let (predicate, predicate_panic) = Predicate::controlled(contacts.clone());
        Self::publish_with_predicate(
            gate,
            0,
            contacts,
            predicate,
            predicate_panic,
            None,
            None,
            Some(maximum),
            1,
            true,
        )
    }

    #[allow(dead_code)] // Shared by the certification crate's cost targets.
    pub fn publish_with_intent_population(gate: &str, intent_row_count: usize) -> Self {
        let contacts = ContactCounters::default();
        let (predicate, predicate_panic) = Predicate::controlled(contacts.clone());
        Self::publish_with_predicate(
            gate,
            0,
            contacts,
            predicate,
            predicate_panic,
            None,
            None,
            None,
            intent_row_count,
            false,
        )
    }

    pub fn publish_replacement(gate: &str) -> Self {
        let contacts = ContactCounters::default();
        let (predicate, predicate_panic) = ReplacementPredicate::controlled(contacts.clone());
        Self::publish_with_predicate(
            gate,
            0,
            contacts,
            predicate,
            predicate_panic,
            None,
            None,
            None,
            1,
            true,
        )
    }

    pub(super) fn publish_with_predicate<Provider>(
        gate: &str,
        unrelated_row_count: usize,
        contacts: ContactCounters,
        predicate: Provider,
        predicate_panic: PanicController,
        maximum_active_snapshots: Option<usize>,
        maximum_world_history: Option<u64>,
        maximum_concurrent_graph_work: Option<usize>,
        intent_row_count: usize,
        include_live_relations: bool,
    ) -> Self
    where
        Provider: domain::WorthQueryHostConditionalPredicateProvider<TemporalReadyNode> + 'static,
    {
        let declaration = TemporalHostSchema::declaration().unwrap();
        let conditional_binding = contract::conditional_binding();
        let package = domain::WorthQueryPortableDomainPackage::new(
            domain::WorthQueryPortableDomainIdentity::new("temporal_host_courtroom", 1, 0),
        )
        .application_schema(declaration.clone())
        .domain_operation(contract::operation_definition().into_portable())
        .conditional_application_operation(conditional_binding.clone())
        .validate()
        .unwrap();
        let admitted = domain::WorthQueryInstallationAdmissionProfile::new("host", "courtroom")
            .admit(package)
            .unwrap();
        let query_resources = runtime::WorthQueryApplicationQueryResourceProfile::bounded(
            5_120,
            2_048,
            usize::MAX,
            maximum_concurrent_graph_work.unwrap_or(128),
        )
        .unwrap();
        let installation = runtime::WorthQueryExecutionRuntimeInstaller::new()
            .application_query_resources(query_resources)
            .install(
                domain::WorthQueryInstallationGeneration::initial(),
                [admitted],
            )
            .unwrap();
        let (runtime, authority) = installation.into_parts();
        let installed_packages = runtime.retain_installed_packages();
        let schema = runtime
            .installed_packages()
            .bind_application_schema(declaration)
            .unwrap();
        let principal_binding = schema
            .principal_binding(TemporalPrincipalBinding::reference())
            .unwrap();
        let authentication = admit_identity_adapter(&schema);
        let operation = schema
            .installed_operation(ExecuteTemporal::reference())
            .unwrap();
        let query = schema
            .application_query(TemporalIntentQuery::reference())
            .unwrap();
        let (clock_source, clock_control) = ClockSource::due();
        let conditional = runtime
            .installed_packages()
            .bind_conditional_application_operation(operation, &conditional_binding)
            .unwrap()
            .bind_node(TemporalReadyNode::reference())
            .unwrap()
            .bind_host_predicate_provider(predicate)
            .unwrap()
            .bind_named_clock::<CourtroomClock, _>(clock_source)
            .unwrap()
            .bind_temporal_intent_projection(
                query,
                ApplicationQueryParameterSet::new(),
                IntentProjector,
                domain::WorthQueryTemporalIntentBounds::new(8, 8, 8).unwrap(),
            )
            .unwrap();

        let mut graph = match (maximum_active_snapshots, maximum_world_history) {
            (Some(maximum_active_snapshots), None) => {
                worth_query_execution::facade::integration::prepare_primary_graph_with_active_snapshot_limit_for_test(
                    &authority,
                    &runtime,
                    &schema,
                    maximum_active_snapshots,
                    resources::product_world_resources(1_024),
                )
                .unwrap()
            }
            (None, Some(maximum_world_history)) => {
                authority
                    .prepare_primary_graph(
                        &runtime,
                        &schema,
                        resources::product_world_resources(maximum_world_history),
                    )
                    .unwrap()
            }
            (None, None) => authority
                .prepare_primary_graph(
                    &runtime,
                    &schema,
                    resources::product_world_resources(1_024),
                )
                .unwrap(),
            (Some(_), Some(_)) => panic!("one installation capacity boundary per courtroom"),
        };
        seed::seed_graph(
            &mut graph,
            &principal_binding,
            gate,
            unrelated_row_count,
            intent_row_count,
            include_live_relations,
        );
        let invariant = Arc::new(graph.retain_invariant_projection_authority());
        let (invoker, preconditions_panic) = Invoker::controlled(contacts.clone());
        let execution = primary_graph::WorthQueryTemporalOperationExecution::with_authorization(
            Arc::clone(&invariant),
            invoker,
            IntentIdentityField::reference(),
            IntentRevisionField::reference(),
            IntentLifecycleField::reference(),
            "active".to_string(),
            "completed".to_string(),
            primary_graph::WorthQueryPublicTemporalOperationAuthorization,
        )
        .unwrap();
        let (principal_source, reconstruction_panic) = PrincipalSource::controlled(authentication);
        let reconstruction = primary_graph::WorthQueryTemporalReconstructionAccess::new(
            principal_binding,
            principal_source,
            IntentIdentityField::reference(),
            "intent-1".to_string(),
        )
        .unwrap();
        let mut conditional_installation = graph
            .conditional_application_runtime_installation(
                runtime,
                authority,
                schema,
                primary_graph::SignalConditionalEvaluationBudget::development(),
            )
            .unwrap();
        let clock = conditional_installation
            .bind_temporal_operation(conditional, execution, reconstruction)
            .unwrap();
        let application = conditional_installation.publish().unwrap();
        Self {
            application,
            clock,
            invariant,
            clock_control,
            predicate_panic,
            reconstruction_panic,
            preconditions_panic,
            contacts,
            installation: installed_packages,
            amendment_ordinal: 0,
        }
    }
}

pub(super) fn admit_identity_adapter(
    schema: &domain::WorthQueryInstalledApplicationSchema<TemporalHostSchema>,
) -> admission::authenticated_principal::WorthQueryAdmittedAuthenticationAdapter<
    TemporalHostSchema,
    IdentityAdapter,
> {
    admission::authenticated_principal::admit_authentication_adapter(
        schema,
        admission::authenticated_principal::WorthQueryAuthenticationAdapterAdmission::new(
            admission::authenticated_principal::WorthQueryAuthenticationAudience::new("host")
                .unwrap(),
            admission::authenticated_principal::WorthQueryAuthenticationMethod::new("test")
                .unwrap(),
        ),
        IdentityAdapter,
    )
    .unwrap()
}

pub fn request_scope() -> admission::authenticated_principal::WorthQueryRequestScope {
    let cancellation = admission::authenticated_principal::WorthQueryCancellationSource::new();
    admission::authenticated_principal::WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(60),
        cancellation.token(),
    )
}
