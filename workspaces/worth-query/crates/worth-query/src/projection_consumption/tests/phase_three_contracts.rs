use super::super::source::{
    ProjectionSourceCapabilityProfile, ProjectionSourceExecutionPosture,
    ProjectionSourceReferenceIdentity, ProjectionWriteReceiptCapabilities,
};
use super::super::{
    declare_projection_consumption, evaluate_projection_consumption_eligibility,
    AdmittedProjectionConsumption, ProjectMaterializedFacts, ProjectionConsumptionBindingContext,
    ProjectionConsumptionDenialReason, ProjectionConsumptionEligibility,
    ProjectionConsumptionSource, ProjectionConsumptionWarningKind, ProjectionContractSourcePosture,
    ProjectionContractSupportPosture, ProjectionSourceFamily,
};

pub(super) fn test_binding(visible_fields: &[&str]) -> ProjectionConsumptionBindingContext {
    ProjectionConsumptionBindingContext::test_only(
        "result-shape:test",
        "authorized-projection:test",
        crate::projection_consumption::test_authorized_field_paths(visible_fields),
    )
}

fn binding_with_policy(
    policy_digest: &str,
    tenant_schema_basis_digest: &str,
) -> ProjectionConsumptionBindingContext {
    ProjectionConsumptionBindingContext::test_only_with_projection_metadata(
        "result-shape:test",
        "query:test",
        "result-shape:test",
        "authorized-projection:test",
        "narrowed-result-shape:test",
        policy_digest,
        tenant_schema_basis_digest,
        crate::projection_consumption::test_authorized_field_paths(&[
            "identity.id",
            "profile.display_name",
        ]),
    )
}

pub(super) fn admitted(
    source: ProjectionConsumptionSource,
    binding: ProjectionConsumptionBindingContext,
    requested: ProjectMaterializedFacts,
) -> AdmittedProjectionConsumption {
    let declaration = declare_projection_consumption(source, binding, requested)
        .expect("declaration should be valid");
    match evaluate_projection_consumption_eligibility(&declaration) {
        ProjectionConsumptionEligibility::Admitted(admitted) => admitted,
        ProjectionConsumptionEligibility::AdmittedWithWarnings(admitted, _) => admitted,
        other => panic!("expected admitted posture, got {other:?}"),
    }
}

pub(super) fn query_read_source() -> ProjectionConsumptionSource {
    ProjectionConsumptionSource::test_only(
        ProjectionSourceFamily::QueryReadReceipt,
        Some("query:test"),
        Some("basis:test"),
        Some("result:test"),
        Some("result-shape:test"),
        "read-graph:test",
    )
}

fn query_write_source_with_source_references() -> ProjectionConsumptionSource {
    ProjectionConsumptionSource::test_only_with_source_references(
        ProjectionSourceFamily::QueryWriteReceipt,
        ProjectionSourceCapabilityProfile::QueryWriteReceipt {
            capabilities: ProjectionWriteReceiptCapabilities::test_only(true, true, true, true),
        },
        None,
        Some("snapshot:test"),
        None,
        None,
        "commit:test",
        vec![
            ProjectionSourceReferenceIdentity::test_only(
                "bridge_provenance_execution_record",
                "bridge-record:test",
            ),
            ProjectionSourceReferenceIdentity::test_only(
                "symbolic_target_reference",
                "$same_batch_target",
            ),
        ],
    )
}

#[test]
fn visibility_admission_requires_exact_native_aspect_field_key() {
    let declaration = declare_projection_consumption(
        query_read_source(),
        test_binding(&["profile.display_name"]),
        ProjectMaterializedFacts::declare().display_field_path(
            crate::projection_consumption::projection_fact_field_path_from_segments([
                worth_foundational::facade::FieldKey::new("profile")
                    .expect("projection fact field segment should admit"),
                worth_foundational::facade::FieldKey::new("display_name")
                    .expect("projection fact field segment should admit"),
                worth_foundational::facade::FieldKey::new("extra")
                    .expect("projection fact field segment should admit"),
            ]),
        ),
    )
    .expect("declaration should be valid");

    let denied = match evaluate_projection_consumption_eligibility(&declaration) {
        ProjectionConsumptionEligibility::Denied(denied) => denied,
        other => panic!("expected exact native field-key visibility denial, got {other:?}"),
    };

    assert_eq!(
        denied.reason(),
        &ProjectionConsumptionDenialReason::FactFamilyNotVisible {
            field_key: "profile.display_name.extra".to_string(),
        }
    );
}

#[test]
fn admitted_read_receipt_binds_query_owned_contract() {
    let admitted = admitted(
        query_read_source(),
        test_binding(&["identity.id", "profile.display_name"]),
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
    );

    let contract = admitted.bind_contract().expect("fixture digest must fit");

    assert_eq!(
        contract.source_posture(),
        ProjectionContractSourcePosture::QueryOwnedReceiptSource
    );
    assert_eq!(
        contract.source_family(),
        ProjectionSourceFamily::QueryReadReceipt
    );
    assert_eq!(contract.query_digest(), Some("query:test"));
    assert_eq!(contract.basis_digest(), Some("basis:test"));
    assert_eq!(contract.result_digest(), Some("result:test"));
    assert_eq!(
        contract.canonical_result_shape_digest(),
        "result-shape:test"
    );
    assert_eq!(
        contract.narrowed_result_shape_digest(),
        "narrowed-result-shape:test"
    );
    assert_eq!(contract.policy_digest(), "policy:test");
    assert_eq!(contract.tenant_schema_basis_digest(), "tenant-schema:test");
    assert_eq!(
        contract.support_posture(),
        &ProjectionContractSupportPosture::Admitted
    );
    assert_eq!(contract.fact_families().len(), 2);
    assert!(!contract.contract_digest().is_empty());
}

#[test]
fn warning_bearing_query_context_contract_carries_warning_posture() {
    let source = ProjectionConsumptionSource::test_only_with_source_references(
        ProjectionSourceFamily::QueryContextExecution,
        ProjectionSourceCapabilityProfile::QueryContextExecution {
            execution_posture: ProjectionSourceExecutionPosture::PreviewDerived,
        },
        Some("query:test"),
        Some("basis:test"),
        Some("result:test"),
        Some("result-shape:test"),
        "query-context:test",
        vec![ProjectionSourceReferenceIdentity::test_only(
            "query_context_preview_provenance",
            "preview-provenance:test",
        )],
    );
    let admitted = admitted(
        source,
        test_binding(&["profile.display_name"]),
        ProjectMaterializedFacts::declare().display_field_path(
            crate::projection_consumption::projection_fact_field_path_from_segments([
                worth_foundational::facade::FieldKey::new("profile")
                    .expect("projection fact field segment should admit"),
                worth_foundational::facade::FieldKey::new("display_name")
                    .expect("projection fact field segment should admit"),
            ]),
        ),
    );

    let contract = admitted.bind_contract().expect("fixture digest must fit");

    assert_eq!(
        contract.support_posture(),
        &ProjectionContractSupportPosture::AdmittedWithWarnings(vec![
            ProjectionConsumptionWarningKind::PreviewDerivedContext,
        ])
    );
    assert_eq!(contract.source_reference_identities().len(), 1);
    assert_eq!(
        contract.source_reference_identities()[0].label(),
        "query_context_preview_provenance"
    );
}

#[test]
fn write_receipt_contract_binds_source_reference_identities() {
    let admitted = admitted(
        query_write_source_with_source_references(),
        binding_with_policy("policy:write", "tenant-schema:write"),
        ProjectMaterializedFacts::declare()
            .target_identity()
            .source_references()
            .effect_continuity_facts()
            .relation_endpoints(),
    );

    let contract = admitted.bind_contract().expect("fixture digest must fit");

    assert_eq!(
        contract.source_posture(),
        ProjectionContractSourcePosture::QueryOwnedReceiptSource
    );
    assert_eq!(contract.query_digest(), None);
    assert_eq!(contract.basis_digest(), Some("snapshot:test"));
    assert_eq!(contract.policy_digest(), "policy:write");
    assert_eq!(contract.tenant_schema_basis_digest(), "tenant-schema:write");
    assert_eq!(contract.source_reference_identities().len(), 2);
    assert_eq!(
        contract.source_reference_identities()[0].label(),
        "bridge_provenance_execution_record"
    );
    assert_eq!(
        contract.source_reference_identities()[1].identity(),
        "$same_batch_target"
    );
}

#[test]
fn relational_and_bridge_sources_bind_distinct_contract_postures() {
    let relational_contract = admitted(
        ProjectionConsumptionSource::test_only(
            ProjectionSourceFamily::RelationalRowSet,
            None,
            Some("snapshot:test"),
            None,
            None,
            "relational-row-set:test",
        ),
        test_binding(&["identity.id"]),
        ProjectMaterializedFacts::declare().entity_identities(),
    )
    .bind_contract()
    .expect("fixture digest must fit");
    let bridge_contract = admitted(
        ProjectionConsumptionSource::test_only(
            ProjectionSourceFamily::BridgeTruthViewRowSet,
            None,
            Some("snapshot:test"),
            None,
            None,
            "bridge-row-set:test",
        ),
        test_binding(&["identity.id"]),
        ProjectMaterializedFacts::declare().entity_identities(),
    )
    .bind_contract()
    .expect("fixture digest must fit");

    assert_eq!(
        relational_contract.source_posture(),
        ProjectionContractSourcePosture::RelationalAuthoritySource
    );
    assert_eq!(
        bridge_contract.source_posture(),
        ProjectionContractSourcePosture::BridgeAuthoritySource
    );
    assert_ne!(
        relational_contract.contract_digest(),
        bridge_contract.contract_digest()
    );
}

#[test]
fn equivalent_contracts_normalize_to_the_same_digest() {
    let left = admitted(
        query_read_source(),
        test_binding(&["identity.id", "profile.display_name"]),
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
    let right = admitted(
        query_read_source(),
        test_binding(&["identity.id", "profile.display_name"]),
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

    assert_eq!(left.contract_digest(), right.contract_digest());
}

#[test]
fn contract_digest_changes_for_fact_inventory_and_policy_basis() {
    let baseline = admitted(
        query_read_source(),
        binding_with_policy("policy:a", "tenant-schema:a"),
        ProjectMaterializedFacts::declare().entity_identities(),
    )
    .bind_contract()
    .expect("fixture digest must fit");
    let widened_facts = admitted(
        query_read_source(),
        binding_with_policy("policy:a", "tenant-schema:a"),
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
    let different_policy = admitted(
        query_read_source(),
        binding_with_policy("policy:b", "tenant-schema:a"),
        ProjectMaterializedFacts::declare().entity_identities(),
    )
    .bind_contract()
    .expect("fixture digest must fit");

    assert_ne!(baseline.contract_digest(), widened_facts.contract_digest());
    assert_ne!(
        baseline.contract_digest(),
        different_policy.contract_digest()
    );
}
