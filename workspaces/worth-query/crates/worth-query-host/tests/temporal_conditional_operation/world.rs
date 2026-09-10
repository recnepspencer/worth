use std::sync::Arc;
use std::time::{Duration, Instant};

use worth_query_host::facade::{
    admission, declaration, declaration::application_query::ApplicationQueryParameterSet, domain,
    primary_graph, runtime,
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
#[path = "world/security.rs"]
mod security;

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
        )
    }

    pub fn publish_replacement(gate: &str) -> Self {
        let contacts = ContactCounters::default();
        let (predicate, predicate_panic) = ReplacementPredicate::controlled(contacts.clone());
        Self::publish_with_predicate(gate, 0, contacts, predicate, predicate_panic, None, None)
    }

    pub(super) fn publish_with_predicate<Provider>(
        gate: &str,
        unrelated_row_count: usize,
        contacts: ContactCounters,
        predicate: Provider,
        predicate_panic: PanicController,
        maximum_active_snapshots: Option<usize>,
        maximum_world_history: Option<u64>,
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
        let installation = runtime::WorthQueryExecutionRuntimeInstaller::new()
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
        seed_graph(&mut graph, &principal_binding, gate, unrelated_row_count);
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

fn seed_graph(
    graph: &mut primary_graph::WorthQueryPrimaryGraphBootstrap<TemporalHostSchema>,
    principal_binding: &domain::WorthQueryInstalledPrincipalBinding<
        TemporalHostSchema,
        TemporalPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
    >,
    gate: &str,
    unrelated_row_count: usize,
) {
    graph
        .bind_principal(
            principal_binding,
            primary_graph::WorthQueryApplicationPrincipalKey::new("temporal-host").unwrap(),
            1_u64,
            declaration::authentication::WorthQueryExternalPrincipalIdentity::new(
                "https://issuer.example",
                "temporal-host",
            )
            .unwrap(),
            declaration::authentication::WorthQueryPrincipalMappingStatus::Enabled,
        )
        .unwrap();
    graph
        .bind_entity(
            primary_graph::WorthQueryApplicationEntitySeed::new(
                TemporalIntent::reference(),
                primary_graph::WorthQueryApplicationEntityKey::new("intent-row-1").unwrap(),
            )
            .field(IntentIdentityField::reference(), "intent-1".to_string())
            .field(IntentRevisionField::reference(), 1_u64)
            .field(IntentDueField::reference(), 5_u64)
            .field(IntentLifecycleField::reference(), "active".to_string())
            .field(IntentInputField::reference(), "payload".to_string())
            .field(IntentGateField::reference(), gate.to_string())
            .field(IntentEffectField::reference(), "pending".to_string()),
        )
        .unwrap();
    graph
        .bind_relation(primary_graph::WorthQueryApplicationRelationSeed::new(
            IntentLiveTarget::reference(),
            "intent-live-target-1",
            primary_graph::WorthQueryApplicationEntityKey::new("intent-row-1").unwrap(),
            primary_graph::WorthQueryApplicationEntityKey::new("intent-row-1").unwrap(),
        ))
        .unwrap();
    for ordinal in 0..unrelated_row_count {
        graph
            .bind_entity(
                primary_graph::WorthQueryApplicationEntitySeed::new(
                    UnrelatedRecord::reference(),
                    primary_graph::WorthQueryApplicationEntityKey::new(format!(
                        "unrelated-{ordinal}"
                    ))
                    .unwrap(),
                )
                .field(UnrelatedValueField::reference(), ordinal as u64),
            )
            .unwrap();
    }
}

pub fn request_scope() -> admission::authenticated_principal::WorthQueryRequestScope {
    let cancellation = admission::authenticated_principal::WorthQueryCancellationSource::new();
    admission::authenticated_principal::WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(60),
        cancellation.token(),
    )
}
