//! try build primary query world with dimensions.
use super::*;

pub(super) fn try_build_primary_query_world_with_dimensions(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    host: &crate::host_world::CourtroomWorld,
    profile: domain::ConsumerProfile,
    scale: PrimaryQueryScale,
    source_installation: &worth_query_execution::facade::primary_graph::WorthQueryGranularInvalidationInstallation,
    snapshot_installation: &worth_query_execution::facade::primary_graph::WorthQueryGranularInvalidationInstallation,
) -> Result<PrimaryQueryWorld, String> {
    let installation = host.application.granular_invalidation_installation();
    let record = host.intent_record_identity();
    let observations = Arc::new(SourceObservations::default());
    let operation = domain::consumer_operation_definition(profile);
    let node = operation.semantics().conditional_nodes[0].clone();
    let dependency = node.dependencies()[0].clone();
    let bridge = bridge::runtime_bridge_with_unrelated_mappings(
        &dependency,
        record,
        &installation,
        scale.unrelated_bridge_mappings,
    );
    let mut signal = worth_signal::facade::SignalGraph::new();
    let signal_node = signal.node().build();
    let worth_proof::TransitionOutcome::Success(installed_signal) =
        signal.admit_installed_node(signal_node)
    else {
        panic!("the certification Signal node must admit")
    };
    let target = worth_runtime_bridge::facade::BridgeSignalAspectTargetDeclaration::allocate(
        worth_runtime_bridge::facade::BridgeAspectRegistrationId::from_stable_name(
            "temporal-primary-intent",
        ),
        worth_signal::facade::PartitionToken::new("temporal-primary"),
        installed_signal,
    );
    for _ in 0..scale.unrelated_signal_subscribers {
        let consumer = signal.node().build();
        signal
            .set_dependencies(
                consumer,
                [worth_signal::facade::DependencyEdge::new(
                    signal_node,
                    worth_signal::facade::Aspect::new(1),
                )],
            )
            .expect("the unrelated Signal subscriber must install");
    }
    let dependency_installation =
        query_domain::WorthQueryConditionalDependencyInstallation::new(Some(record), vec![target]);
    let providers =
        worth_runtime_bridge::facade::BridgeConditionalProviderSet::new().wake(EligibleProvider);
    let conditional_compute = ConditionalCompute {
        next_version: Arc::new(std::sync::atomic::AtomicU64::new(1)),
    };
    let source = WorthQueryPrimaryGraphSourceAdapter::new(
        source_installation,
        IntentSourceProjection::new(record, Arc::clone(&observations)),
    );
    let builder = runtime::WorthQueryRuntime::builder(
        worth_query::facade::consumer_kit::in_memory_test_product_world_resources(),
    )
    .primary_runtime_granular_invalidations(installation.clone())
    .domain_package(domain::package(profile))
    .expect("the temporal consumer package must admit")
    .graph_participation(domain::graph_definition())
    .graph_participation_provider(domain::PrimaryGraph, domain::PrimaryGraphProvider)
    .runtime_bridge(bridge)
    .conditional_execution_resources(
        runtime::WorthQueryConditionalExecutionResources::development(),
    )
    .conditional_signal_graph(signal)
    .conditional_node(
        TemporalDomain,
        TemporalDomainOperation,
        TemporalDomainFamily,
        domain::PrimaryGraph,
        query_domain::WorthQueryConditionalNodeLocation::operation(node.identity()).unwrap(),
        vec![dependency_installation],
        providers,
        conditional_compute,
    )
    .domain_operation_executor(
        TemporalDomain,
        TemporalDomainOperation,
        TemporalDomainFamily,
        domain::OperationExecutor(profile),
    );
    let builder = if scale.install_unrelated_query {
        builder
            .domain_package(domain::unrelated_package())
            .expect("the unrelated query package must admit")
            .domain_operation_executor(
                domain::UnrelatedDomain,
                domain::UnrelatedOperation,
                domain::UnrelatedFamily,
                domain::UnrelatedExecutor,
            )
    } else {
        builder
    };
    let backend = builder
        .consumer_support_posture(
            query_domain::WorthQueryConsumerSupportDimension::ConditionalEvaluation,
            query_domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .consumer_support_posture(
            query_domain::WorthQueryConsumerSupportDimension::ConditionalComparator,
            query_domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .consumer_support_posture(
            query_domain::WorthQueryConsumerSupportDimension::ConditionalTrigger,
            query_domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .consumer_support_posture(
            query_domain::WorthQueryConsumerSupportDimension::ConditionalTemporalOrOnDemand,
            query_domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .consumer_support_posture(
            query_domain::WorthQueryConsumerSupportDimension::Live,
            query_domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .consumer_support_posture(
            query_domain::WorthQueryConsumerSupportDimension::Sharing,
            query_domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .consumer_support_posture(
            query_domain::WorthQueryConsumerSupportDimension::DependencyImpact,
            query_domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .consumer_support_posture(
            query_domain::WorthQueryConsumerSupportDimension::Invalidation,
            query_domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .consumer_support_posture(
            query_domain::WorthQueryConsumerSupportDimension::Continuation,
            if profile == domain::ConsumerProfile::OrderedPortfolio {
                query_domain::WorthQueryConsumerSupportPosture::Supported
            } else {
                query_domain::WorthQueryConsumerSupportPosture::Unsupported
            },
        )
        .aspect_contract(dependency.contract().clone())
        .expect("the temporal dependency contract must install")
        .schema_adapter(backend::SchemaAdapter)
        .source_adapter(source)
        .snapshot_identity(backend::PrimarySnapshotAdapter::new(snapshot_installation))
        .write_authority(backend::DenyingWriteAuthority)
        .signal_sink(backend::SignalSink)
        .subscription_activation(backend::SubscriptionActivation)
        .preview_basis(backend::PreviewBasis)
        .inspector_evidence(backend::InspectorEvidence)
        .build_backend_from_parts();
    let runtime = backend
        .build(request_execution)
        .map_err(|error| error.to_string())?;
    let mut workspace = runtime
        .workspace("temporal-primary-query")
        .map_err(|error| error.to_string())?;
    if profile == domain::ConsumerProfile::SharedValuePatch {
        drop(settle_primary_projection(&mut workspace));
    }
    let settled = settle_primary_projection(&mut workspace);
    let live = match settled
        .into_lifecycle()
        .promote(request_execution, &mut workspace)
    {
        query_domain::WorthQueryProjectionPromotionOutcome::Promoted(live) => live,
        query_domain::WorthQueryProjectionPromotionOutcome::Denied(stop)
        | query_domain::WorthQueryProjectionPromotionOutcome::Deferred(stop)
        | query_domain::WorthQueryProjectionPromotionOutcome::Failed(stop) => panic!(
            "the temporal primary projection must promote: {:?}: {}",
            stop.kind(),
            stop.detail()
        ),
        query_domain::WorthQueryProjectionPromotionOutcome::Stale(_) => {
            panic!("the temporal primary projection unexpectedly became stale")
        }
        query_domain::WorthQueryProjectionPromotionOutcome::RebindRequired(_) => {
            panic!("the temporal primary projection unexpectedly required rebinding")
        }
        query_domain::WorthQueryProjectionPromotionOutcome::AuthorityRevalidationRequired(_) => {
            panic!("the temporal primary projection unexpectedly required authority revalidation")
        }
    };
    Ok(PrimaryQueryWorld {
        workspace,
        live,
        observations,
    })
}
