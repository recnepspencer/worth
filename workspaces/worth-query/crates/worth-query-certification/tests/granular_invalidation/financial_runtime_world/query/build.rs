//! build.
use super::*;

pub(super) fn build(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    host: &super::super::host::FinancialCourtroomWorld,
    profile: FinancialQueryProfile,
    establish_sharing_baseline: bool,
    source_record: Option<worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts>,
    unrelated_portfolio_rows: usize,
) -> FinancialQueryWorld {
    let installation = host.application.granular_invalidation_installation();
    let record = source_record.unwrap_or_else(|| host.record_identity());
    let operation = query_domain::operation_definition(profile);
    let node = operation.semantics().conditional_nodes[0].clone();
    let dependencies = node.dependencies().to_vec();
    let default_mapping_identity = match profile {
        FinancialQueryProfile::CurveRisk => "financial-primary-curve",
        FinancialQueryProfile::CurveRecordRisk => "financial-primary-curve-record",
        FinancialQueryProfile::QuoteRisk => "financial-primary-quote",
        FinancialQueryProfile::OrderedPortfolio => "financial-primary-portfolio",
    };
    let mapping_identity = default_mapping_identity;
    let signal_partition = "financial-primary";
    let bridge = crate::query_runtime_world::runtime_bridge_for_dependencies(
        &dependencies,
        record,
        &installation,
        mapping_identity,
        0,
    );
    let mut signal = worth_signal::facade::SignalGraph::new();
    let signal_node = signal.node().build();
    let installed_signals = dependencies
        .iter()
        .map(|_| {
            let worth_proof::TransitionOutcome::Success(installed) =
                signal.admit_installed_node(signal_node)
            else {
                panic!("the financial Query Signal node must admit")
            };
            installed
        })
        .collect::<Vec<_>>();
    let dependency_installations = installation::install_dependencies(
        &dependencies,
        installed_signals,
        record,
        mapping_identity,
        signal_partition,
    );
    let aspect_contracts = installation::unique_aspect_contracts(&dependencies);
    let providers =
        worth_runtime_bridge::facade::BridgeConditionalProviderSet::new().wake(EligibleProvider);
    let providers = match profile {
        FinancialQueryProfile::CurveRisk
        | FinancialQueryProfile::CurveRecordRisk
        | FinancialQueryProfile::OrderedPortfolio => providers,
        FinancialQueryProfile::QuoteRisk => providers.output_comparator(QueryQuoteComparator),
    };
    let source = WorthQueryPrimaryGraphSourceAdapter::new(
        &installation,
        match profile {
            FinancialQueryProfile::CurveRisk | FinancialQueryProfile::CurveRecordRisk => {
                FinancialSourceProjection::curve_risk(record)
            }
            FinancialQueryProfile::QuoteRisk => FinancialSourceProjection::committed_risk(record),
            FinancialQueryProfile::OrderedPortfolio => FinancialSourceProjection::portfolio(
                record,
                host.sibling_curve_record_identity(),
                unrelated_portfolio_rows,
            ),
        },
    );
    let mut workspace = runtime::WorthQueryRuntime::builder(
        worth_query::facade::consumer_kit::in_memory_test_product_world_resources(),
    )
    .primary_runtime_granular_invalidations(installation.clone())
    .domain_package(query_domain::package(profile))
    .expect("the financial Query package must admit")
    .graph_participation(crate::query_runtime_world::graph_definition())
    .graph_participation_provider(
        crate::query_runtime_world::PrimaryGraph,
        crate::query_runtime_world::PrimaryGraphProvider,
    )
    .runtime_bridge(bridge)
    .conditional_execution_resources(
        runtime::WorthQueryConditionalExecutionResources::development(),
    )
    .conditional_signal_graph(signal)
    .conditional_node(
        FinancialDomain,
        FinancialOperation,
        FinancialFamily,
        crate::query_runtime_world::PrimaryGraph,
        domain::WorthQueryConditionalNodeLocation::operation(node.identity()).unwrap(),
        dependency_installations,
        providers,
        ConditionalCompute {
            next_version: Arc::new(std::sync::atomic::AtomicU64::new(1)),
        },
    )
    .domain_operation_executor(
        FinancialDomain,
        FinancialOperation,
        FinancialFamily,
        FinancialExecutor(profile),
    )
    .consumer_support_posture(
        domain::WorthQueryConsumerSupportDimension::ConditionalEvaluation,
        domain::WorthQueryConsumerSupportPosture::Supported,
    )
    .consumer_support_posture(
        domain::WorthQueryConsumerSupportDimension::ConditionalComparator,
        domain::WorthQueryConsumerSupportPosture::Supported,
    )
    .consumer_support_posture(
        domain::WorthQueryConsumerSupportDimension::ConditionalTrigger,
        domain::WorthQueryConsumerSupportPosture::Supported,
    )
    .consumer_support_posture(
        domain::WorthQueryConsumerSupportDimension::ConditionalTemporalOrOnDemand,
        domain::WorthQueryConsumerSupportPosture::Supported,
    )
    .consumer_support_posture(
        domain::WorthQueryConsumerSupportDimension::Live,
        domain::WorthQueryConsumerSupportPosture::Supported,
    )
    .consumer_support_posture(
        domain::WorthQueryConsumerSupportDimension::Sharing,
        domain::WorthQueryConsumerSupportPosture::Supported,
    )
    .consumer_support_posture(
        domain::WorthQueryConsumerSupportDimension::DependencyImpact,
        domain::WorthQueryConsumerSupportPosture::Supported,
    )
    .consumer_support_posture(
        domain::WorthQueryConsumerSupportDimension::Invalidation,
        domain::WorthQueryConsumerSupportPosture::Supported,
    )
    .consumer_support_posture(
        domain::WorthQueryConsumerSupportDimension::CollectionDelivery,
        if profile == FinancialQueryProfile::OrderedPortfolio {
            domain::WorthQueryConsumerSupportPosture::Supported
        } else {
            domain::WorthQueryConsumerSupportPosture::Unsupported
        },
    )
    .consumer_support_posture(
        domain::WorthQueryConsumerSupportDimension::Continuation,
        if profile == FinancialQueryProfile::OrderedPortfolio {
            domain::WorthQueryConsumerSupportPosture::Supported
        } else {
            domain::WorthQueryConsumerSupportPosture::Unsupported
        },
    )
    .aspect_contracts(aspect_contracts)
    .expect("the financial dependency contracts must install")
    .schema_adapter(crate::query_runtime_world::SchemaAdapter)
    .source_adapter(source)
    .snapshot_identity(crate::query_runtime_world::PrimarySnapshotAdapter::new(
        &installation,
    ))
    .write_authority(crate::query_runtime_world::DenyingWriteAuthority)
    .signal_sink(crate::query_runtime_world::SignalSink)
    .subscription_activation(crate::query_runtime_world::SubscriptionActivation)
    .preview_basis(crate::query_runtime_world::PreviewBasis)
    .inspector_evidence(crate::query_runtime_world::InspectorEvidence)
    .build_backend_from_parts()
    .build(request_execution)
    .expect("the financial primary-backed Query runtime must build")
    .workspace("financial-primary-query")
    .expect("the financial Query workspace must open");
    if establish_sharing_baseline {
        drop(settle(&mut workspace, profile));
    }
    let settled = settle(&mut workspace, profile);
    let collection = (profile == FinancialQueryProfile::OrderedPortfolio)
        .then(|| {
            settled.prepare_collection_consumer(
                domain::WorthQueryCollectionWindowBreadth::new(2, 0, 0, 2)
                    .expect("the portfolio court uses two bounded visible rows"),
            )
        })
        .transpose()
        .expect("the ordered portfolio must prepare retained collection state");
    let live = match settled
        .into_lifecycle()
        .promote(request_execution, &mut workspace)
    {
        domain::WorthQueryProjectionPromotionOutcome::Promoted(live) => live,
        _ => panic!("the financial risk projection must promote"),
    };
    FinancialQueryWorld {
        workspace,
        live,
        collection,
    }
}
