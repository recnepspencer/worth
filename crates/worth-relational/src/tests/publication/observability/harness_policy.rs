use super::fixtures::*;

#[test]
fn harness_heavy_invariants_are_opt_in() {
    let runtime = runtime_with_declared_aspect_schema_and_invariants(InvariantCatalog {
        registrations: vec![InvariantRegistration::harness_audit_only(
            InvariantRule::unique_entity_aspect_field(aspect_key("name"), field_key("name")),
        )],
    });
    let _ = create_entity(&runtime, "duplicate");
    let _ = create_entity(&runtime, "duplicate");

    let default_results = runtime
        .validation()
        .harness_audit(HarnessAuditMode::Disabled)
        .into_results();
    let enabled_results = runtime
        .validation()
        .harness_audit(HarnessAuditMode::Full)
        .into_results();

    assert!(default_results.is_empty());
    assert_eq!(enabled_results.len(), 1);
    assert_eq!(enabled_results[0].class(), InvariantClass::HarnessHeavy);
    assert!(matches!(
        enabled_results[0].verdict,
        crate::validation::data::InvariantVerdict::Advisory { .. }
    ));
}
