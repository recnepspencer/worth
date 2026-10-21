use super::{
    account_parameter, admitted_requirements, installed_query, ApplicationQueryParameterSet,
    WorthQueryApplicationQueryLane, WorthQueryGraphReadAccessRequirementKind,
    WorthQueryGraphReadOrderingPosture, WorthQueryGraphReadResultPressure,
};
use crate::facade::application_query::admit_application_query_parameters;

#[test]
fn installed_application_graph_uses_canonical_requirement_derivation() {
    let query = installed_query();
    let parameters = admit_application_query_parameters(
        &query,
        ApplicationQueryParameterSet::new()
            .bind(account_parameter(), 7_u64)
            .unwrap(),
    )
    .unwrap();
    let requirements = admitted_requirements(
        query.read_graph(),
        WorthQueryApplicationQueryLane::OneShot,
        32,
        parameters.identity(),
    );

    for kind in [
        WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency,
        WorthQueryGraphReadAccessRequirementKind::PredicateSupport,
        WorthQueryGraphReadAccessRequirementKind::OrderingSupport,
        WorthQueryGraphReadAccessRequirementKind::ProofSupport,
        WorthQueryGraphReadAccessRequirementKind::ResultBuffer,
        WorthQueryGraphReadAccessRequirementKind::MaterializationLifecycle,
    ] {
        assert!(requirements.contains_kind(&kind), "missing {kind:?}");
    }
    assert!(
        !requirements.contains_kind(&WorthQueryGraphReadAccessRequirementKind::TraversalWorkset)
    );
    assert!(!requirements.contains_kind(&WorthQueryGraphReadAccessRequirementKind::VisitedSet));
    assert!(!requirements
        .contains_kind(&WorthQueryGraphReadAccessRequirementKind::LiveMaintenanceSupport));
    assert!(requirements
        .rows()
        .iter()
        .all(|row| row.maximum_cardinality() == Some(32)));
    let ordering = requirements
        .rows()
        .iter()
        .find(|row| row.kind() == &WorthQueryGraphReadAccessRequirementKind::OrderingSupport)
        .unwrap();
    assert_eq!(
        ordering.ordering_posture(),
        Some(&WorthQueryGraphReadOrderingPosture::BoundedProjectedCollection)
    );
    assert_eq!(
        ordering.ordering_field_authorities()[0].collection_path(),
        "root/relation[0]"
    );
    let result_buffer = requirements
        .rows()
        .iter()
        .find(|row| row.kind() == &WorthQueryGraphReadAccessRequirementKind::ResultBuffer)
        .unwrap();
    assert_eq!(
        requirements
            .rows()
            .iter()
            .filter(|row| row.kind() == &WorthQueryGraphReadAccessRequirementKind::ResultBuffer)
            .count(),
        1
    );
    assert_eq!(
        result_buffer.result_pressure(),
        Some(&WorthQueryGraphReadResultPressure::CollectionWide)
    );
}

#[test]
fn installed_graph_admission_preserves_native_support_requirements_without_quotes() {
    use crate::graph_read_access::{
        review_graph_read_access, WorthQueryGraphIndexInventory,
        WorthQueryGraphIndexLifecycleClass as Lifecycle,
        WorthQueryGraphIndexLifecycleOwner as Owner, WorthQueryGraphIndexPosture as Posture,
        WorthQueryGraphIndexSupportRow, WorthQueryGraphIndexSupportState as State,
        WorthQueryGraphReadAccessAdmissionPosture as Admission,
        WorthQueryGraphReadPlanReviewDenialKind as Denial,
    };
    let query = installed_query();
    let parameters = admit_application_query_parameters(
        &query,
        ApplicationQueryParameterSet::new()
            .bind(account_parameter(), 7_u64)
            .unwrap(),
    )
    .unwrap();
    let requirements = admitted_requirements(
        query.read_graph(),
        WorthQueryApplicationQueryLane::OneShot,
        32,
        parameters.identity(),
    );
    let inventory = WorthQueryGraphIndexInventory::from_current_runtime_support();
    let accepted = review_graph_read_access(requirements.clone(), inventory.clone());
    assert!(accepted.is_admitted(), "{accepted:?}");
    assert_eq!(accepted.posture(), &Admission::InlineIndexed);
    assert_eq!(accepted.requirements(), &requirements);
    assert_eq!(accepted.inventory(), &inventory);

    for (owner, lifecycle, posture, state, expected) in [
        (
            Owner::StoreOwned,
            Lifecycle::StoreOwnedRequired,
            Posture::RequiresStoreBackedPersistentIndex,
            State::StoreOwnedUnavailable,
            Denial::RequiredPersistentIndex,
        ),
        (
            Owner::DomainRegistration,
            Lifecycle::AccessCapabilityRegistrationRequired,
            Posture::RequiresAccessCapabilityRegistration,
            State::Declared,
            Denial::RequiredAccessCapabilityRegistration,
        ),
        (
            Owner::LowerRuntime,
            Lifecycle::TemporarilyUnavailable,
            Posture::TemporarilyUnavailable,
            State::TemporarilyUnavailable,
            Denial::RequiredAsyncMaterialization,
        ),
        (
            Owner::LowerRuntime,
            Lifecycle::Unsupported,
            Posture::Denied,
            State::Unsupported,
            Denial::UnsupportedGraphIndexSupport,
        ),
    ] {
        let kind = WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency;
        let mut rows = inventory.rows().to_vec();
        rows.retain(|row| row.requirement_kind() != &kind);
        rows.push(
            WorthQueryGraphIndexSupportRow::with_runtime_support_posture(
                kind, owner, lifecycle, posture, state, None,
            ),
        );
        let denied = review_graph_read_access(
            requirements.clone(),
            WorthQueryGraphIndexInventory::from_rows(rows),
        );
        assert!(!denied.is_admitted());
        assert_eq!(denied.denial().unwrap().kind(), expected);
        assert_eq!(denied.posture(), &Admission::Denied);
        assert_eq!(denied.requirements(), &requirements);
    }
}
