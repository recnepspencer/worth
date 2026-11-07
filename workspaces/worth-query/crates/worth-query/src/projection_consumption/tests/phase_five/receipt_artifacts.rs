use super::super::super::{
    ProjectMaterializedFacts, ProjectionConsumptionReceipt, ProjectionConsumptionSource,
    ProjectionConsumptionTransitionKind, ProjectionConsumptionTransitionPosture,
    SelfDescribingProjectionConsumptionEnvelope,
};
use super::super::phase_four::support::{admitted, binding, relational_row_set};

#[test]
fn consumed_fact_set_issues_receipt_with_stable_operational_fields() {
    let row_set = relational_row_set();
    let contract = admitted(
        ProjectionConsumptionSource::from_relational_row_set(&row_set),
        binding(&["identity.id", "profile.display_name"]),
        ProjectMaterializedFacts::declare()
            .entity_identities()
            .display_field_path(
                crate::projection_consumption::projection_fact_field_path_from_segments([
                    worth_foundational::facade::FieldKey::new("profile")
                        .expect("projection fact field segment should admit"),
                    worth_foundational::facade::FieldKey::new("display_name")
                        .expect("projection fact field segment should admit"),
                ]),
            ),
    )
    .bind_contract()
    .expect("fixture digest must fit");
    let consumed = contract.extract_from_relational_row_set(&row_set).unwrap();

    let receipt = consumed.issue_receipt().expect("fixture digest must fit");

    assert_eq!(receipt.contract_digest(), contract.contract_digest());
    assert_eq!(receipt.fact_set_digest(), consumed.fact_set_digest());
    assert_eq!(receipt.source_family(), consumed.source_family());
    assert_eq!(receipt.source_identity(), consumed.source_identity());
    assert_eq!(receipt.admitted_fact_family_count(), 2);
    assert_eq!(receipt.extracted_fact_count(), 4);
    assert_eq!(receipt.authority_reopen_count(), 0);
    assert!(!receipt.deferred_neighbors().is_empty());
    assert!(!receipt.integrity_digest().is_empty());
    assert!(!receipt.receipt_digest().is_empty());
}

#[test]
fn projection_consumption_transition_rules_name_implemented_and_deferred_neighbors() {
    let row_set = relational_row_set();
    let contract = admitted(
        ProjectionConsumptionSource::from_relational_row_set(&row_set),
        binding(&["identity.id"]),
        ProjectMaterializedFacts::declare().entity_identities(),
    )
    .bind_contract()
    .expect("fixture digest must fit");
    let consumed = contract.extract_from_relational_row_set(&row_set).unwrap();
    let receipt = consumed.issue_receipt().expect("fixture digest must fit");
    let rules = receipt.transition_rules();

    assert!(rules.rules().iter().any(
        |rule: &crate::projection_consumption::ProjectionConsumptionTransitionRule| {
            rule.kind() == ProjectionConsumptionTransitionKind::InspectReceipt
                && rule.posture() == ProjectionConsumptionTransitionPosture::Implemented
        }
    ));
    assert!(rules.rules().iter().any(
        |rule: &crate::projection_consumption::ProjectionConsumptionTransitionRule| {
            rule.kind() == ProjectionConsumptionTransitionKind::ReloadPersistedReceipt
                && rule.posture() == ProjectionConsumptionTransitionPosture::Deferred
                && rule.deferred_neighbor().is_some()
        }
    ));
    assert!(!rules.rules_digest().is_empty());
}

#[test]
fn receipt_derives_self_describing_projection_consumption_envelope() {
    let row_set = relational_row_set();
    let contract = admitted(
        ProjectionConsumptionSource::from_relational_row_set(&row_set),
        binding(&["identity.id", "profile.display_name"]),
        ProjectMaterializedFacts::declare()
            .entity_identities()
            .display_field_path(
                crate::projection_consumption::projection_fact_field_path_from_segments([
                    worth_foundational::facade::FieldKey::new("profile")
                        .expect("projection fact field segment should admit"),
                    worth_foundational::facade::FieldKey::new("display_name")
                        .expect("projection fact field segment should admit"),
                ]),
            ),
    )
    .bind_contract()
    .expect("fixture digest must fit");
    let consumed = contract.extract_from_relational_row_set(&row_set).unwrap();
    let receipt = consumed.issue_receipt().expect("fixture digest must fit");

    let envelope = receipt.projection_consumption_envelope();

    assert_eq!(envelope.source_family(), receipt.source_family());
    assert_eq!(envelope.source_identity(), receipt.source_identity());
    assert_eq!(envelope.admitted_fact_family_count(), 2);
    assert_eq!(envelope.extracted_fact_count(), 4);
    assert_eq!(
        envelope.sources().receipt_digest(),
        receipt.receipt_digest()
    );
    assert_eq!(
        envelope.sources().fact_set_digest(),
        receipt.fact_set_digest()
    );
    assert_eq!(
        envelope.transition_rules_digest(),
        receipt.transition_rules().rules_digest()
    );
    assert!(!envelope.envelope_digest().is_empty());
}

#[test]
fn receipt_and_envelope_surfaces_are_publicly_reachable() {
    let _receipt_issue: fn(
        &super::super::super::ConsumedProjectionFactSet,
    ) -> ProjectionConsumptionReceipt =
        super::super::super::ConsumedProjectionFactSet::issue_receipt;
    let _envelope_derive: fn(
        &ProjectionConsumptionReceipt,
    ) -> SelfDescribingProjectionConsumptionEnvelope =
        ProjectionConsumptionReceipt::projection_consumption_envelope;
}
