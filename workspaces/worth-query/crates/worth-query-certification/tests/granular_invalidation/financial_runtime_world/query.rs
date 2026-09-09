use std::sync::Arc;

use worth_query::facade::runtime::WorthQueryPrimaryGraphSourceAdapter;
use worth_query::facade::{domain, read, runtime};

use super::contract::{FinancialDomain, FinancialFamily, FinancialOperation};
use super::query_domain::{self, FinancialExecutor, FinancialQueryProfile};
use super::query_source::FinancialSourceProjection;

#[path = "query/installation.rs"]
mod installation;
#[path = "query/providers.rs"]
mod providers;

use providers::{ConditionalCompute, EligibleProvider, QueryQuoteComparator};

pub type FinancialGraph = crate::query_runtime_world::PrimaryGraph;

pub struct FinancialQueryWorld {
    pub workspace: runtime::WorthQueryWorkspace,
    pub live: domain::WorthQueryLiveBoundDomainProjection<
        FinancialDomain,
        FinancialOperation,
        FinancialFamily,
        worth_query::facade::foundation::ObservationLaneWitness,
    >,
    pub collection: Option<domain::WorthQueryCollectionConsumerWindow>,
}

pub type FinancialSharedLease = domain::WorthQuerySharedLiveProjectionLease<
    FinancialDomain,
    FinancialOperation,
    FinancialFamily,
    worth_query::facade::foundation::ObservationLaneWitness,
>;

pub struct SharedFinancialQueryWorld {
    pub workspace: runtime::WorthQueryWorkspace,
    pub subject: FinancialSharedLease,
    pub candidate: FinancialSharedLease,
}

pub fn build_curve(host: &super::host::FinancialCourtroomWorld) -> FinancialQueryWorld {
    build(host, FinancialQueryProfile::CurveRisk, false, None, 0)
}

pub fn build_sibling_curve_record(
    host: &super::host::FinancialCourtroomWorld,
) -> FinancialQueryWorld {
    build(
        host,
        FinancialQueryProfile::CurveRecordRisk,
        false,
        Some(host.sibling_curve_record_identity()),
        0,
    )
}

pub fn build_quote(host: &super::host::FinancialCourtroomWorld) -> FinancialQueryWorld {
    build(host, FinancialQueryProfile::QuoteRisk, false, None, 0)
}

pub fn build_portfolio_with_unrelated_rows(
    host: &super::host::FinancialCourtroomWorld,
    unrelated_rows: usize,
) -> FinancialQueryWorld {
    build(
        host,
        FinancialQueryProfile::OrderedPortfolio,
        false,
        None,
        unrelated_rows,
    )
}

pub fn build_shared_curve(
    host: &super::host::FinancialCourtroomWorld,
) -> SharedFinancialQueryWorld {
    let FinancialQueryWorld {
        mut workspace,
        live,
        collection: _,
    } = build(host, FinancialQueryProfile::CurveRisk, true, None, 0);
    let candidate = settle(&mut workspace, FinancialQueryProfile::CurveRisk).into_lifecycle();
    let shared = match live.share_with(candidate, &mut workspace) {
        domain::WorthQueryProjectionSharingOutcome::Shared(shared) => shared,
        domain::WorthQueryProjectionSharingOutcome::Stopped(stop) => {
            panic!("financial projection sharing stopped: {}", stop.detail())
        }
    };
    let (subject, candidate) = shared.into_leases();
    subject
        .admit_consumer_delivery_policy(
            &mut workspace,
            domain::WorthQuerySharedConsumerDeliveryPolicy::new(
                "desk-risk-monitoring",
                "desk-risk-public",
                "desk-risk-cursor",
                runtime::DeliveryBackpressurePolicy::RetainWithinWindow,
            )
            .unwrap(),
        )
        .unwrap();
    candidate
        .admit_consumer_delivery_policy(
            &mut workspace,
            domain::WorthQuerySharedConsumerDeliveryPolicy::new(
                "regulatory-capital",
                "restricted-capital",
                "regulatory-cursor",
                runtime::DeliveryBackpressurePolicy::DropWithGapNotice,
            )
            .unwrap(),
        )
        .unwrap();
    SharedFinancialQueryWorld {
        workspace,
        subject,
        candidate,
    }
}

fn build(
    host: &super::host::FinancialCourtroomWorld,
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
    let mut workspace = runtime::WorthQueryRuntime::builder()
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
        .build()
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
    let live = match settled.into_lifecycle().promote(&mut workspace) {
        domain::WorthQueryProjectionPromotionOutcome::Promoted(live) => live,
        _ => panic!("the financial risk projection must promote"),
    };
    FinancialQueryWorld {
        workspace,
        live,
        collection,
    }
}

fn settle(
    workspace: &mut runtime::WorthQueryWorkspace,
    profile: FinancialQueryProfile,
) -> domain::WorthQuerySettledDomainProjection<
    FinancialDomain,
    FinancialOperation,
    FinancialFamily,
    worth_query::facade::foundation::ObservationLaneWitness,
> {
    let installed = workspace.domain(FinancialDomain).unwrap();
    let bound = workspace
        .observe_operating_world()
        .unwrap()
        .family(FinancialFamily)
        .bind(&installed, FinancialOperation)
        .unwrap();
    let consumer = bound.consumer_projection_contract().unwrap();
    let published = bound
        .admit_execution_resources((), query_domain::resource_request(), &*workspace)
        .unwrap()
        .execute(workspace)
        .unwrap()
        .publish()
        .unwrap();
    if profile == FinancialQueryProfile::OrderedPortfolio {
        let mut request = consumer.projection_request();
        request
            .select_derived_native_field_name("PortfolioValueField")
            .unwrap();
        request
            .select_derived_native_field_name("PortfolioDeskField")
            .unwrap();
        published
            .consume_bound(request.build().unwrap())
            .unwrap()
            .settle()
            .unwrap()
    } else {
        published
            .consume(
                consumer,
                read::project_facts().entity_identities().display_field(
                    read::ProjectionFactFieldPath::from_canonical_field_path(
                        worth_foundational::facade::CanonicalFieldPath::new(vec![
                            worth_foundational::facade::FieldKey::new("RiskFacts").unwrap(),
                            worth_foundational::facade::FieldKey::new("RiskValueField").unwrap(),
                        ])
                        .unwrap(),
                    ),
                ),
            )
            .unwrap()
            .settle()
            .unwrap()
    }
}
