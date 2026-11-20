use worth_query_declaration::facade::authoring::{
    AspectFieldSelector, AuthoredResultShapeField, GuidedAuthoringPath, RawAuthoredQuery,
    RawAuthoredResultShape, RootEntityKey, TraversalSelector,
};
use worth_query_declaration::facade::canonicalization::CanonicalQueryBundle;

use super::super::policy_basis::{
    admit_policy_tenant_context, BranchAccessGrant, PolicyEpoch, PolicyExecutionModeRequest,
    PolicyRuleSnapshot,
};
use super::super::tenant_basis::{SchemaVariantSnapshot, TenantBasisEpoch, TenantBindingSnapshot};

use super::{
    admit_relationship_proofs, RelationshipProofBudget, RelationshipProofDescriptor,
    RelationshipProofDescriptorSet, RelationshipProofFailureClass, RelationshipProofTopologyClass,
};

fn canonical_query() -> CanonicalQueryBundle {
    let query = RawAuthoredQuery::detail_builder(RootEntityKey::new("user").unwrap())
        .project(AspectFieldSelector::new("identity", "id").unwrap())
        .build()
        .unwrap();
    let result_shape = RawAuthoredResultShape::detail_builder()
        .field(AuthoredResultShapeField::new("identity", "id", "id").unwrap())
        .build()
        .unwrap();
    GuidedAuthoringPath::canonicalize_detail(query, result_shape).unwrap()
}

fn admitted(
    canonical: &CanonicalQueryBundle,
) -> super::super::policy_basis::AdmittedPolicyTenantContext {
    let policy = PolicyRuleSnapshot::synthetic_authority(
        "runtime-policy",
        "rules-v1",
        PolicyEpoch::Synthetic(7),
    );
    let tenant = TenantBindingSnapshot::synthetic_direct(
        "tenant-a",
        "branch-a",
        "schema-a",
        TenantBasisEpoch::Synthetic(3),
    );
    let branch = BranchAccessGrant::synthetic_granted("branch-a", &policy);
    let schema = SchemaVariantSnapshot::synthetic_authority("tenant-a", "schema-a", "compatible");
    admit_policy_tenant_context(
        canonical.query(),
        policy,
        tenant,
        branch,
        schema,
        PolicyExecutionModeRequest::CurrentRead,
    )
    .unwrap()
}

#[test]
fn direct_edge_descriptor_admits_without_truth_touch() {
    let query = RawAuthoredQuery::detail_builder(RootEntityKey::new("user").unwrap())
        .project(AspectFieldSelector::new("identity", "id").unwrap())
        .traverse(TraversalSelector::bounded("manager", 1).unwrap())
        .build()
        .unwrap();
    let result_shape = RawAuthoredResultShape::detail_builder()
        .field(AuthoredResultShapeField::new("identity", "id", "id").unwrap())
        .build()
        .unwrap();
    let canonical = GuidedAuthoringPath::canonicalize_detail(query, result_shape).unwrap();
    let admitted = admitted(&canonical);
    let descriptors = RelationshipProofDescriptorSet::new(
        vec![RelationshipProofDescriptor::direct_edge(
            "manager",
            admitted.bundle().policy_digest(),
        )],
        RelationshipProofBudget::bounded(1, 1),
    );

    let (admission, counters) =
        admit_relationship_proofs(canonical.query(), &admitted, &descriptors).unwrap();

    assert_eq!(admission.descriptor_count(), 1);
    assert_eq!(counters.relationship_proof_admission_count(), 1);
    assert_eq!(counters.truth_touch_count(), 0);
}

#[test]
fn unbounded_recursive_descriptor_denies() {
    let canonical = canonical_query();
    let admitted = admitted(&canonical);
    let descriptors = RelationshipProofDescriptorSet::new(
        vec![RelationshipProofDescriptor::unbounded_recursive_walk_for_test("manager")],
        RelationshipProofBudget::bounded(1, 1),
    );

    let error = admit_relationship_proofs(canonical.query(), &admitted, &descriptors).unwrap_err();

    assert_eq!(
        error.failure_class(),
        RelationshipProofFailureClass::UnboundedRecursiveWalk
    );
    assert_eq!(
        error
            .counters()
            .relationship_proof_recursive_broadening_denial_count(),
        1
    );
}

#[test]
fn tenant_membership_descriptor_binds_tenant_schema_basis() {
    let canonical = canonical_query();
    let admitted = admitted(&canonical);
    let descriptors = RelationshipProofDescriptorSet::new(
        vec![RelationshipProofDescriptor::tenant_membership(
            admitted.bundle().tenant_schema_basis_digest(),
        )],
        RelationshipProofBudget::bounded(1, 1),
    );

    let (admission, counters) =
        admit_relationship_proofs(canonical.query(), &admitted, &descriptors).unwrap();

    assert_eq!(
        admission.topology_classes(),
        &[RelationshipProofTopologyClass::TenantMembership]
    );
    assert_eq!(counters.relationship_proof_admission_count(), 1);
    assert_eq!(counters.truth_touch_count(), 0);
}

#[test]
fn bounded_ancestor_descriptor_requires_nonzero_bound() {
    let error = RelationshipProofDescriptor::bounded_ancestor("manager", 0, "policy-a")
        .expect_err("zero-depth bounded ancestors must fail before admission");

    assert!(matches!(
        error,
        worth_query_declaration::facade::authoring::AuthoringError::UnsupportedTraversalDepth {
            depth: 0
        }
    ));
}

#[test]
fn query_shape_mismatch_denies_before_truth_touch() {
    let canonical = canonical_query();
    let admitted = admitted(&canonical);
    let descriptors = RelationshipProofDescriptorSet::new(
        vec![RelationshipProofDescriptor::query_shape_mismatch_for_test(
            "different-query-digest",
        )],
        RelationshipProofBudget::bounded(1, 1),
    );

    let error = admit_relationship_proofs(canonical.query(), &admitted, &descriptors).unwrap_err();

    assert_eq!(
        error.failure_class(),
        RelationshipProofFailureClass::QueryShapeMismatch
    );
    assert_eq!(error.counters().truth_touch_count(), 0);
}

#[test]
fn duplicate_proof_cannot_cover_two_distinct_traversals() {
    let query = RawAuthoredQuery::detail_builder(RootEntityKey::new("user").unwrap())
        .project(AspectFieldSelector::new("identity", "id").unwrap())
        .traverse(TraversalSelector::bounded("manager", 1).unwrap())
        .traverse(TraversalSelector::bounded("owner", 1).unwrap())
        .build()
        .unwrap();
    let result_shape = RawAuthoredResultShape::detail_builder()
        .field(AuthoredResultShapeField::new("identity", "id", "id").unwrap())
        .build()
        .unwrap();
    let canonical = GuidedAuthoringPath::canonicalize_detail(query, result_shape).unwrap();
    let admitted = admitted(&canonical);
    let descriptors = RelationshipProofDescriptorSet::new(
        vec![
            RelationshipProofDescriptor::direct_edge("manager", admitted.bundle().policy_digest()),
            RelationshipProofDescriptor::direct_edge("manager", admitted.bundle().policy_digest()),
        ],
        RelationshipProofBudget::bounded(2, 2),
    );

    let error = admit_relationship_proofs(canonical.query(), &admitted, &descriptors).unwrap_err();

    assert_eq!(
        error.failure_class(),
        RelationshipProofFailureClass::QueryShapeMismatch
    );
    assert_eq!(error.counters().truth_touch_count(), 0);
}

fn manager_query(projected_field: &str) -> CanonicalQueryBundle {
    let query = RawAuthoredQuery::detail_builder(RootEntityKey::new("user").unwrap())
        .project(AspectFieldSelector::new("identity", projected_field).unwrap())
        .traverse(TraversalSelector::bounded("manager", 1).unwrap())
        .build()
        .unwrap();
    let result_shape = RawAuthoredResultShape::detail_builder()
        .field(AuthoredResultShapeField::new("identity", projected_field, projected_field).unwrap())
        .build()
        .unwrap();
    GuidedAuthoringPath::canonicalize_detail(query, result_shape).unwrap()
}

#[test]
fn context_admitted_for_another_query_with_identical_traversal_denies() {
    let query_a = manager_query("id");
    let query_b = manager_query("name");
    assert_ne!(query_a.query().digest(), query_b.query().digest());
    assert_eq!(query_a.query().traversal(), query_b.query().traversal());
    let admitted_for_a = admitted(&query_a);
    let descriptors = RelationshipProofDescriptorSet::new(
        vec![RelationshipProofDescriptor::direct_edge(
            "manager",
            admitted_for_a.bundle().policy_digest(),
        )],
        RelationshipProofBudget::bounded(1, 1),
    );

    let error = admit_relationship_proofs(query_b.query(), &admitted_for_a, &descriptors)
        .expect_err("a context admitted for query A must not admit proofs for query B");
    assert_eq!(
        error.failure_class(),
        RelationshipProofFailureClass::QueryShapeMismatch
    );
    assert_eq!(error.counters().relationship_proof_admission_count(), 0);
    assert_eq!(error.counters().truth_touch_count(), 0);

    let (admission, _) = admit_relationship_proofs(query_a.query(), &admitted_for_a, &descriptors)
        .expect("the same context still admits proofs for query A");
    assert_eq!(admission.descriptor_count(), 1);
}
