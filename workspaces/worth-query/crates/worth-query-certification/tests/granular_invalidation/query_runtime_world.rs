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

pub fn build_primary_query_world(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    host: &crate::host_world::CourtroomWorld,
) -> PrimaryQueryWorld {
    build_primary_query_world_with_profile(
        request_execution,
        host,
        domain::ConsumerProfile::ValuePatch,
    )
}

pub fn build_primary_query_world_with_profile(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    host: &crate::host_world::CourtroomWorld,
    profile: domain::ConsumerProfile,
) -> PrimaryQueryWorld {
    build_primary_query_world_with_scale(request_execution, host, profile, 0)
}

pub fn build_primary_query_world_with_scale(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    host: &crate::host_world::CourtroomWorld,
    profile: domain::ConsumerProfile,
    unrelated_bridge_mappings: usize,
) -> PrimaryQueryWorld {
    build_primary_query_world_with_dimensions(
        request_execution,
        host,
        profile,
        PrimaryQueryScale {
            unrelated_bridge_mappings,
            ..PrimaryQueryScale::default()
        },
    )
}

pub fn build_primary_query_world_with_dimensions(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    host: &crate::host_world::CourtroomWorld,
    profile: domain::ConsumerProfile,
    scale: PrimaryQueryScale,
) -> PrimaryQueryWorld {
    let source_installation = host.application.granular_invalidation_installation();
    try_build_primary_query_world_with_dimensions(
        request_execution,
        host,
        profile,
        scale,
        &source_installation,
        &source_installation,
    )
    .expect("the primary-backed Query runtime must build")
}

#[path = "query_runtime_world/try_build_primary_query_world_with_dimensions.rs"]
mod try_build_primary_query_world_with_dimensions;
use try_build_primary_query_world_with_dimensions::try_build_primary_query_world_with_dimensions;

pub fn build_shared_primary_query_world(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    host: &crate::host_world::CourtroomWorld,
) -> SharedPrimaryQueryWorld {
    let PrimaryQueryWorld {
        mut workspace,
        live,
        observations: _,
    } = build_primary_query_world_with_profile(
        request_execution,
        host,
        domain::ConsumerProfile::SharedValuePatch,
    );
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

    bound
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
        .unwrap()
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
