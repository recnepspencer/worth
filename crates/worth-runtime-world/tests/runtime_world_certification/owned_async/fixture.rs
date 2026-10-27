use std::sync::Arc;

use worth_relational::facade::{
    mvcc::{PreparedRelationalCommitCandidate, RelationalTransactionIntent},
    runtime::{RelationalRuntime, RelationalRuntimeApi},
    transactions::WorkerIntentBatch,
};
use worth_runtime_bridge::facade::*;
use worth_runtime_world::facade::*;

pub struct AsyncSourceCourt {
    pub world: RuntimeWorldOwner<(), (), (), (), ()>,
    pub bridge: BridgeSealedRuntimeAssembly,
    pub declaration: LoweredBridgeAsyncSourceDeclaration,
    pub source: RuntimeBridgeRelationalSource,
    pub relational: Arc<RelationalRuntime>,
    pub empty: ProductBranchObservation,
}

struct NoDelivery;
impl InvalidationSink for NoDelivery {
    fn deliver_invalidation(
        &self,
        _: BridgeSignalInvalidationDelivery,
        _: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        panic!("source identity admission must not deliver invalidations")
    }
}

struct Clock;
impl RuntimeWorldClockSource for Clock {
    fn now(&self) -> RuntimeWorldInstant {
        RuntimeWorldInstant::from_ticks(1)
    }
}

impl AsyncSourceCourt {
    pub fn new() -> Self {
        Self::with_queue_capacity(None)
    }

    pub fn managed(capacity: std::num::NonZeroU64) -> Self {
        Self::with_queue_capacity(Some(capacity))
    }

    fn with_queue_capacity(capacity: Option<std::num::NonZeroU64>) -> Self {
        let relational = Arc::new(RelationalRuntimeApi::builder().build());
        let source = RuntimeBridgeRelationalSource::for_graph_role(
            Arc::clone(&relational),
            "owned-async-source",
        )
        .unwrap();
        let bridge = RuntimeBridge::builder()
            .with_relational_source(source.clone())
            .with_signal_sink(NoDelivery)
            .register_mapping(BridgeMappingRegistration::new(
                BridgeMappingId::from_stable_name("source-value"),
                TruthPatchScope::for_entity_field(
                    MappingSelector::exact("source-record"),
                    worth_foundational::facade::AspectKey::new("value").unwrap(),
                    worth_foundational::facade::FieldKey::new("value").unwrap(),
                ),
                SnapshotReadContract::scalar(
                    worth_foundational::facade::AspectKey::new("value").unwrap(),
                    worth_foundational::facade::ScalarAspectType::String,
                ),
                SignalInvalidationScope::from_stable_name("source-value"),
                CoarseRoutingMode::Direct,
            ))
            .build()
            .unwrap();
        let mut builder = BridgeConditionalRuntimeBuilder::with_owned_signal_graph(
            bridge,
            worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
        )
        .unwrap();
        let declaration =
            BridgeOwnedAsyncRequestResponseDeclaration::new("exact-source", 919, 512, 2, 2, 5);
        let declaration = if let Some(capacity) = capacity {
            declaration.with_managed_queue_capacity(capacity)
        } else {
            declaration
        };
        let declaration = builder
            .install_owned_async_request_response(declaration)
            .unwrap();
        let mut bridge = builder.seal().unwrap();
        let relational_basis = relational
            .observe_branch(&relational.main_branch_identity())
            .unwrap()
            .1;
        let initial = RuntimeWorldBootstrapIntent::new(
            ProductBranchCreationIntent::named("main").unwrap(),
            relational_basis,
            bridge.admitted_signal_basis().clone(),
            bridge.admitted_runtime_world_correspondence_basis().clone(),
        );
        let world = RuntimeWorldOwner::builder()
            .with_relational_services(relational.owner_component_services())
            .with_signal_services(bridge.signal_owner_services())
            .with_signal_definition_publication(
                bridge
                    .take_runtime_world_signal_definition_publication()
                    .unwrap(),
            )
            .with_bridge_correspondence(bridge.runtime_world_correspondence_port())
            .with_budgets(RuntimeWorldBudgets::without_policy_limits())
            .with_clock(RuntimeWorldClock::from_source(Clock))
            .with_execution_policy(worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Serial,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(
                    std::num::NonZeroUsize::MIN,
                    None,
                    Some(8_000_000),
                ),
            ))
            .build()
            .unwrap();
        let RuntimeWorldBootstrapOutcome::Performed(initial) =
            world.lifecycle_port().bootstrap_root(initial).unwrap()
        else {
            panic!("bootstrap must select the real empty source")
        };
        let empty = initial.product_branch().clone();
        Self {
            world,
            bridge,
            declaration,
            source,
            relational,
            empty,
        }
    }

    pub fn prepare(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        head: &ProductBranchObservation,
    ) -> PreparedRelationalCommitCandidate {
        let mut tx = self
            .relational
            .begin_branch_transaction(
                head.basis().relational_basis(),
                RelationalTransactionIntent::ordinary(),
            )
            .unwrap();
        tx.push_batch(
            WorkerIntentBatch::new("metadata-only-source-event"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
        self.relational
            .owner_component_services()
            .preparation_port()
            .prepare_branch_transaction(execution, tx)
            .unwrap()
    }

    pub fn publish(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        head: &ProductBranchObservation,
    ) -> ProductBranchObservation {
        let candidate = self.prepare(&execution, head);
        let cancellation = RuntimeWorldCancellationSource::new().token();
        let publication = self.world.publication_port();
        let prepared = publication
            .prepare_without_signal(
                head.clone(),
                CompositePublicationIntent::without_signal(RelationalTransactionIntent::ordinary())
                    .with_prepared_relational_candidate(candidate),
                &cancellation,
                None,
            )
            .unwrap();
        let RuntimeWorldPublicationOutcome::Performed(performed) =
            publication.execute_without_signal(execution, prepared, &cancellation)
        else {
            panic!("native metadata publication must perform")
        };
        let _performed = performed.consume();
        self.world
            .observation_port()
            .observe_product_branch(head.branch_identity())
            .unwrap()
    }

    pub fn fork(&self, head: &ProductBranchObservation, name: &str) -> ProductBranchObservation {
        let RuntimeWorldBranchCreationOutcome::Performed(fork) = self
            .world
            .branch_port()
            .create_product_branch(
                head.clone(),
                ProductBranchCreationIntent::from_source(
                    name,
                    ProductBranchCreationPlans::new(
                        RelationalBranchCreationPlan::ForkExact {
                            target: worth_relational::facade::history::BranchId(name.into()),
                        },
                        SignalBranchCreationPlan::ReuseExact,
                    ),
                )
                .unwrap(),
                &RuntimeWorldCancellationSource::new().token(),
            )
            .unwrap()
        else {
            panic!("native fork must perform")
        };
        fork
    }

    pub fn lease(&self, head: &ProductBranchObservation) -> RelationalBridgeObservationLease {
        self.source
            .retain_branch_basis_for_bridge(head.basis().relational_basis())
            .unwrap()
    }
}

pub fn source_commit(request: &BridgeOwnedAsyncRequestAdmission) -> &TruthCommitIdentity {
    request
        .request()
        .basis_binding()
        .truth_view_basis()
        .truth_commit_identity()
        .unwrap()
}

pub fn completion(
    request: &BridgeOwnedAsyncRequestAdmission,
) -> worth_signal::facade::RawCompletionEnvelope {
    let request = request.request();
    worth_signal::facade::RawCompletionEnvelope::new(
        request.request_handle().request_id(),
        request.request_handle().generation(),
        request.request_handle().branch_epoch(),
        request.attempt(),
        request
            .lowered()
            .resource_descriptor()
            .unwrap()
            .payload_contract_digest()
            .clone(),
        64,
    )
}
