#[path = "query_runtime_world/backend.rs"]
mod backend;
#[path = "query_runtime_world/bridge.rs"]
mod bridge;
#[path = "query_runtime_world/domain.rs"]
mod domain;
#[path = "query_runtime_world/hostile_binding.rs"]
mod hostile_binding;
#[path = "query_runtime_world/providers.rs"]
mod providers;
#[path = "query_runtime_world/source.rs"]
mod source;

pub use backend::{
    DenyingWriteAuthority, InspectorEvidence, PreviewBasis, PrimarySnapshotAdapter, SchemaAdapter,
    SignalSink, SubscriptionActivation,
};
pub use bridge::{mapping_identity_for_dependency, runtime_bridge_for_dependencies};
pub use domain::ConsumerProfile;
pub use domain::{graph_definition, resource_support, PrimaryGraph, PrimaryGraphProvider};
pub use hostile_binding::{
    assert_foreign_primary_source_is_denied_at_build, build_with_foreign_snapshot_adapter,
};
pub use source::{IntentSourceProjection, SourceObservations};

use std::sync::Arc;

use worth_query::facade::runtime::WorthQueryPrimaryGraphSourceAdapter;
use worth_query::facade::{domain as query_domain, read, runtime};
use worth_query_host::facade::domain::APPLICATION_EXECUTION_SAFE_POINT_FAMILY;

use crate::contract::{TemporalDomain, TemporalDomainFamily, TemporalDomainOperation};
use providers::{ConditionalCompute, EligibleProvider};

pub struct PrimaryQueryWorld {
    pub workspace: runtime::WorthQueryWorkspace,
    pub live: query_domain::WorthQueryLiveBoundDomainProjection<
        TemporalDomain,
        TemporalDomainOperation,
        TemporalDomainFamily,
        worth_query::facade::foundation::ObservationLaneWitness,
    >,
    pub observations: Arc<SourceObservations>,
}

pub type PrimarySharedLease = query_domain::WorthQuerySharedLiveProjectionLease<
    TemporalDomain,
    TemporalDomainOperation,
    TemporalDomainFamily,
    worth_query::facade::foundation::ObservationLaneWitness,
>;

pub struct SharedPrimaryQueryWorld {
    pub workspace: runtime::WorthQueryWorkspace,
    pub subject: PrimarySharedLease,
    pub candidate: PrimarySharedLease,
}

#[derive(Clone, Copy, Default)]
pub struct PrimaryQueryScale {
    pub unrelated_bridge_mappings: usize,
    pub unrelated_signal_subscribers: usize,
    pub install_unrelated_query: bool,
}

pub fn build_primary_query_world(host: &crate::host_world::CourtroomWorld) -> PrimaryQueryWorld {
    build_primary_query_world_with_profile(host, domain::ConsumerProfile::ValuePatch)
}

pub fn build_primary_query_world_with_profile(
    host: &crate::host_world::CourtroomWorld,
    profile: domain::ConsumerProfile,
) -> PrimaryQueryWorld {
    build_primary_query_world_with_scale(host, profile, 0)
}

pub fn build_primary_query_world_with_scale(
    host: &crate::host_world::CourtroomWorld,
    profile: domain::ConsumerProfile,
    unrelated_bridge_mappings: usize,
) -> PrimaryQueryWorld {
    build_primary_query_world_with_dimensions(
        host,
        profile,
        PrimaryQueryScale {
            unrelated_bridge_mappings,
            ..PrimaryQueryScale::default()
        },
    )
}

pub fn build_primary_query_world_with_dimensions(
    host: &crate::host_world::CourtroomWorld,
    profile: domain::ConsumerProfile,
    scale: PrimaryQueryScale,
) -> PrimaryQueryWorld {
    let source_installation = host.application.granular_invalidation_installation();
    try_build_primary_query_world_with_dimensions(
        host,
        profile,
        scale,
        &source_installation,
        &source_installation,
    )
    .expect("the primary-backed Query runtime must build")
}

fn try_build_primary_query_world_with_dimensions(
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
    let runtime = backend.build().map_err(|error| error.to_string())?;
    let mut workspace = runtime
        .workspace("temporal-primary-query")
        .map_err(|error| error.to_string())?;
    if profile == domain::ConsumerProfile::SharedValuePatch {
        drop(settle_primary_projection(&mut workspace));
    }
    let settled = settle_primary_projection(&mut workspace);
    let live = match settled.into_lifecycle().promote(&mut workspace) {
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

pub fn build_shared_primary_query_world(
    host: &crate::host_world::CourtroomWorld,
) -> SharedPrimaryQueryWorld {
    let PrimaryQueryWorld {
        mut workspace,
        live,
        observations: _,
    } = build_primary_query_world_with_profile(host, domain::ConsumerProfile::SharedValuePatch);
    let candidate = settle_primary_projection(&mut workspace).into_lifecycle();
    let shared = match live.share_with(candidate, &mut workspace) {
        query_domain::WorthQueryProjectionSharingOutcome::Shared(shared) => shared,
        query_domain::WorthQueryProjectionSharingOutcome::Stopped(stop) => {
            panic!("primary projection sharing stopped: {}", stop.detail())
        }
    };
    let (subject, candidate) = shared.into_leases();
    subject
        .admit_consumer_delivery_policy(
            &mut workspace,
            query_domain::WorthQuerySharedConsumerDeliveryPolicy::new(
                "risk-monitoring",
                "public-risk",
                "public-risk-cursor",
                runtime::DeliveryBackpressurePolicy::RetainWithinWindow,
            )
            .unwrap(),
        )
        .expect("the public consumer policy must bind its current lease");
    candidate
        .admit_consumer_delivery_policy(
            &mut workspace,
            query_domain::WorthQuerySharedConsumerDeliveryPolicy::new(
                "regulatory-risk",
                "governed-risk",
                "governed-risk-cursor",
                runtime::DeliveryBackpressurePolicy::DropWithGapNotice,
            )
            .unwrap(),
        )
        .expect("the governed consumer policy must bind its current lease");
    SharedPrimaryQueryWorld {
        workspace,
        subject,
        candidate,
    }
}

fn settle_primary_projection(
    workspace: &mut runtime::WorthQueryWorkspace,
) -> query_domain::WorthQuerySettledDomainProjection<
    TemporalDomain,
    TemporalDomainOperation,
    TemporalDomainFamily,
    worth_query::facade::foundation::ObservationLaneWitness,
> {
    let installed = workspace.domain(TemporalDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(TemporalDomainFamily)
        .bind(&installed, TemporalDomainOperation)
        .unwrap();
    let consumer = bound.consumer_projection_contract().unwrap();
    let settled = bound
        .admit_execution_resources((), resource_request(), &*workspace)
        .unwrap()
        .execute(workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume(
            consumer,
            read::project_facts().entity_identities().display_field(
                read::ProjectionFactFieldPath::from_canonical_field_path(
                    worth_foundational::facade::CanonicalFieldPath::new(vec![
                        worth_foundational::facade::FieldKey::new("IntentFacts").unwrap(),
                        worth_foundational::facade::FieldKey::new("IntentGateField").unwrap(),
                    ])
                    .unwrap(),
                ),
            ),
        )
        .unwrap()
        .settle()
        .unwrap();
    settled
}

fn resource_request() -> query_domain::WorthQueryExecutionResourceRequest {
    query_domain::WorthQueryExecutionResourceRequest::bounded(
        1_000,
        1_000,
        query_domain::WorthQueryCancellationSafePointFamily::new(
            APPLICATION_EXECUTION_SAFE_POINT_FAMILY,
        )
        .unwrap(),
    )
}
