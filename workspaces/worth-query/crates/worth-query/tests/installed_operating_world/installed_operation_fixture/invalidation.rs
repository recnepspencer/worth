use worth_foundational::facade::{
    profiles, AdmissionReadinessProfile, CertificationPostureProfile, CompatibilityPostureProfile,
    DiagnosticRichnessProfile, ExecutionObjectiveProfile, FieldKey,
    MaterializedFoundationalProfileSet, ObservationActivationProfile, RetentionDeliveryProfile,
    SupportPostureProfile,
};
use worth_proof::TransitionOutcome;
use worth_query::facade::{domain, foundation, runtime};

use super::{configured_runtime, GeometryDomain, ReadExecutionInput, ReadFamily, ReadVertex};

pub(crate) type InvalidationLease = domain::WorthQuerySharedLiveProjectionLease<
    GeometryDomain,
    ReadVertex,
    ReadFamily,
    foundation::ObservationLaneWitness,
>;

pub(crate) fn shared_native_leases(
    name: &str,
) -> (
    runtime::WorthQueryWorkspace,
    InvalidationLease,
    InvalidationLease,
) {
    shared_native_leases_with_invalidation(
        name,
        domain::WorthQueryConsumerSupportPosture::Supported,
    )
}

pub(crate) fn shared_native_leases_with_invalidation(
    name: &str,
    invalidation: domain::WorthQueryConsumerSupportPosture,
) -> (
    runtime::WorthQueryWorkspace,
    InvalidationLease,
    InvalidationLease,
) {
    let mut workspace = configured_runtime()
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::Sharing,
            domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::Invalidation,
            invalidation,
        )
        .consumer_support_posture(
            domain::WorthQueryConsumerSupportDimension::DependencyImpact,
            domain::WorthQueryConsumerSupportPosture::Supported,
        )
        .workspace(name)
        .unwrap();
    let live = match settle_native(&mut workspace)
        .into_lifecycle()
        .promote(&mut workspace)
    {
        domain::WorthQueryProjectionPromotionOutcome::Promoted(live) => live,
        _ => panic!("native subject did not promote"),
    };
    let candidate = settle_native(&mut workspace).into_lifecycle();
    let shared = match live.share_with(candidate, &mut workspace) {
        domain::WorthQueryProjectionSharingOutcome::Shared(shared) => shared,
        domain::WorthQueryProjectionSharingOutcome::Stopped(stop) => {
            panic!("native sharing stopped: {}", stop.detail())
        }
    };
    let (subject, candidate) = shared.into_leases();
    (workspace, subject, candidate)
}

pub(crate) fn settle_native(
    workspace: &mut runtime::WorthQueryWorkspace,
) -> super::super::operation_sharing::SettledProjection {
    settle_native_lane(workspace, false)
}

pub(crate) fn settle_native_derived(
    workspace: &mut runtime::WorthQueryWorkspace,
) -> super::super::operation_sharing::SettledProjection {
    settle_native_lane(workspace, true)
}

fn settle_native_lane(
    workspace: &mut runtime::WorthQueryWorkspace,
    derived: bool,
) -> super::super::operation_sharing::SettledProjection {
    let installed = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, ReadVertex)
        .unwrap();
    let mut builder = bound
        .consumer_projection_contract()
        .unwrap()
        .projection_request();
    if derived {
        builder
            .select_derived_native_field(FieldKey::new("id").unwrap())
            .unwrap();
    } else {
        builder
            .select_display_native_field(FieldKey::new("id").unwrap())
            .unwrap();
    }
    let request = builder.build().unwrap();
    bound
        .admit_execution_resources(
            ReadExecutionInput::default(),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &*workspace,
        )
        .unwrap()
        .execute(workspace)
        .unwrap()
        .publish()
        .unwrap()
        .consume_bound(request)
        .unwrap()
        .settle()
        .unwrap()
}

pub(crate) fn consume_empty_invalidation_epoch(
    workspace: &mut runtime::WorthQueryWorkspace,
    first: &InvalidationLease,
    second: &InvalidationLease,
) {
    assert!(first.drain(workspace).unwrap().delivery().is_empty());
    assert!(second.drain(workspace).unwrap().delivery().is_empty());
}

pub(crate) fn materialized_invalidation_profile() -> MaterializedFoundationalProfileSet {
    let requested = profiles()
        .set()
        .diagnostic_richness(DiagnosticRichnessProfile::Standard)
        .support_posture(SupportPostureProfile::SupportReady)
        .compatibility_posture(CompatibilityPostureProfile::CompatibilityLowered)
        .admission_readiness(AdmissionReadinessProfile::Admitted)
        .retention_delivery(RetentionDeliveryProfile::Retained)
        .certification_posture(CertificationPostureProfile::Uncertified)
        .execution_objective(ExecutionObjectiveProfile::Balanced)
        .observation_activation(ObservationActivationProfile::Continuous)
        .request()
        .unwrap();
    let TransitionOutcome::Success(admitted) = profiles().progression().admit_same(requested)
    else {
        panic!("static invalidation profile was not admitted")
    };
    let TransitionOutcome::Success(materialized) =
        profiles().progression().materialize_same(admitted)
    else {
        panic!("static invalidation profile was not materialized")
    };
    *materialized.payload()
}
