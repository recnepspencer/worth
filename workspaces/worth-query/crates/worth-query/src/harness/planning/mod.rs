use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
mod basis_rejections;
mod execution;
mod plan_shapes;
mod request_context;

use crate::authoring::{RawAuthoredQuery, RawAuthoredResultShape};
use crate::facade::foundation::{
    canonicalize_request, AuthoredResultShapeField, BasisAuthorityFamily, ExecutionBasisIntent,
    IdentityBindingDescriptor, QueryBindingDescriptor, QueryBindingSlot, QueryBindingSubject,
    ResolvedSnapshotIdentity, RootEntityKey, SnapshotLineageClass,
};
use crate::facade::runtime::validate_canonical_bundle;

fn direct_validated_bundle() -> crate::facade::runtime::ValidatedQueryBundle {
    let query = RawAuthoredQuery::detail_builder(RootEntityKey::new("user").unwrap())
        .project(crate::facade::foundation::AspectFieldSelector::new("identity", "id").unwrap())
        .build()
        .unwrap();
    let shape = RawAuthoredResultShape::detail_builder()
        .field(AuthoredResultShapeField::new("identity", "id", "id").unwrap())
        .build()
        .unwrap();

    let request =
        crate::facade::foundation::GuidedAuthoringPath::pair_detail(query, shape).unwrap();
    let canonical = canonicalize_request(request).unwrap();
    validate_canonical_bundle(
        canonical,
        crate::harness::fixtures::schema_view::detail_schema_view(),
    )
    .unwrap()
}

fn bound_validated_bundle() -> crate::facade::runtime::ValidatedQueryBundle {
    let query = RawAuthoredQuery::detail_builder(RootEntityKey::new("user").unwrap())
        .project(crate::facade::foundation::AspectFieldSelector::new("identity", "id").unwrap())
        .build()
        .unwrap();
    let shape = RawAuthoredResultShape::detail_builder()
        .field(AuthoredResultShapeField::new("identity", "id", "id").unwrap())
        .build()
        .unwrap();
    let bindings = QueryBindingDescriptor::new().with_identity(IdentityBindingDescriptor::new(
        QueryBindingSlot::new("root").unwrap(),
        QueryBindingSubject::RootEntity,
    ));

    let request = crate::facade::foundation::GuidedAuthoringPath::pair_detail_with_bindings(
        query, shape, bindings,
    )
    .unwrap();
    let canonical = canonicalize_request(request).unwrap();
    validate_canonical_bundle(
        canonical,
        crate::harness::fixtures::schema_view::detail_schema_view(),
    )
    .unwrap()
}

fn expanded_validated_bundle() -> crate::facade::runtime::ValidatedQueryBundle {
    crate::harness::fixtures::validated_bundles::legal_detail_bundle()
}

fn collection_validated_bundle() -> crate::facade::runtime::ValidatedQueryBundle {
    let query = RawAuthoredQuery::collection_builder(RootEntityKey::new("user").unwrap())
        .project(crate::facade::foundation::AspectFieldSelector::new("identity", "id").unwrap())
        .project(
            crate::facade::foundation::AspectFieldSelector::new("profile", "display_name").unwrap(),
        )
        .order_by(
            crate::facade::foundation::OrderingSelector::ascending("profile", "display_name")
                .unwrap(),
        )
        .traverse(crate::facade::foundation::TraversalSelector::bounded("manager", 1).unwrap())
        .build()
        .unwrap();
    let shape = RawAuthoredResultShape::collection_builder()
        .field(AuthoredResultShapeField::new("identity", "id", "id").unwrap())
        .field(AuthoredResultShapeField::new("profile", "display_name", "display_name").unwrap())
        .build()
        .unwrap();

    let request =
        crate::facade::foundation::GuidedAuthoringPath::pair_collection(query, shape).unwrap();
    let canonical = canonicalize_request(request).unwrap();
    validate_canonical_bundle(
        canonical,
        crate::harness::fixtures::schema_view::detail_schema_view(),
    )
    .unwrap()
}

fn descending_collection_validated_bundle() -> crate::facade::runtime::ValidatedQueryBundle {
    let query = RawAuthoredQuery::collection_builder(RootEntityKey::new("user").unwrap())
        .project(crate::facade::foundation::AspectFieldSelector::new("identity", "id").unwrap())
        .project(
            crate::facade::foundation::AspectFieldSelector::new("profile", "display_name").unwrap(),
        )
        .order_by(
            crate::facade::foundation::OrderingSelector::descending("profile", "display_name")
                .unwrap(),
        )
        .traverse(crate::facade::foundation::TraversalSelector::bounded("manager", 1).unwrap())
        .build()
        .unwrap();
    let shape = RawAuthoredResultShape::collection_builder()
        .field(AuthoredResultShapeField::new("identity", "id", "id").unwrap())
        .field(AuthoredResultShapeField::new("profile", "display_name", "display_name").unwrap())
        .build()
        .unwrap();

    let request =
        crate::facade::foundation::GuidedAuthoringPath::pair_collection(query, shape).unwrap();
    let canonical = canonicalize_request(request).unwrap();
    validate_canonical_bundle(
        canonical,
        crate::harness::fixtures::schema_view::detail_schema_view(),
    )
    .unwrap()
}

fn runtime_basis_intent() -> ExecutionBasisIntent {
    ExecutionBasisIntent::new(
        BasisAuthorityFamily::Runtime,
        SnapshotLineageClass::CurrentHead,
        false,
    )
}

fn runtime_resolved_identity(
    schema_basis: crate::facade::foundation::SchemaBasisDigest,
) -> Result<ResolvedSnapshotIdentity, IdentityDerivationDenial> {
    Ok(ResolvedSnapshotIdentity::new(
        BasisAuthorityFamily::Runtime,
        Some("workspace-main".to_string()),
        crate::memory_workspace::admit_external_snapshot_label("snapshot-1")?.evidence_identity(),
        schema_basis,
        SnapshotLineageClass::CurrentHead,
    ))
}
