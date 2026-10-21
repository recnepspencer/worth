// Exact existing readiness source contract for both fixture compositions.
use super::runtime_domain as domain;
use worth_query_decl::facade::{
    authoring::{
        AspectFieldSelector, AuthoredQueryBundleRequest, AuthoredResultShapeField,
        DetailQueryBuilder, DetailResultShapeBuilder, RootEntityKey,
    },
    binding::QueryBindingDescriptor,
    canonicalization::canonicalize_request,
};

pub(super) fn output_change_dependency() -> domain::WorthQuerySemanticTruthDependency {
    domain::WorthQuerySemanticTruthDependency::new(
        domain::WorthQueryConditionalGraphReadRole::new("primary").unwrap(),
        output_contract(),
        output_change_mask(),
        domain::AspectBinding::EntityField {
            field: domain::FieldKey::new("Geometry").unwrap(),
        },
        domain::WorthQuerySemanticLocality::SourceRecord,
        [domain::AuthoritativeAspectChangeKind::FieldSet],
    )
    .unwrap()
}

fn output_contract() -> domain::AspectContract {
    let field = |name, scalar| {
        domain::FieldDeclaration::new(
            domain::FieldKey::new(name).unwrap(),
            scalar,
            domain::FieldRequirement::Required,
            domain::AbsenceLaw::Required,
            domain::AspectEvolutionPolicy::AdditiveFieldsAllowed,
        )
        .unwrap()
    };
    domain::AspectContract::struct_aspect(
        domain::AspectKey::new("Geometry").unwrap(),
        domain::AspectIdentity(0x9174_1001),
        domain::AspectContractRevision(1),
        domain::StructAspectShape::new([field("Length", domain::ScalarAspectType::UInt64)])
            .unwrap(),
    )
}

fn output_change_mask() -> domain::AspectMask<domain::ProjectionMask> {
    domain::AspectMask::new([domain::CanonicalFieldPath::single(
        domain::FieldKey::new("Length").unwrap(),
    )])
}

pub(super) fn output_change_projection() -> domain::WorthQueryOperationNativeProjectionContract {
    domain::WorthQueryOperationNativeProjectionContract::new(
        output_contract(),
        output_change_mask(),
    )
    .unwrap()
}

pub(super) fn canonical_query() -> worth_query_decl::facade::canonicalization::CanonicalQueryBundle
{
    let query = DetailQueryBuilder::new(RootEntityKey::new("Body").unwrap())
        .project(AspectFieldSelector::new("Geometry", "Length").unwrap())
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(AuthoredResultShapeField::new("Geometry", "Length", "length").unwrap())
        .build()
        .unwrap()
        .into_raw();
    canonicalize_request(
        AuthoredQueryBundleRequest::for_ordinary_read(query, shape, QueryBindingDescriptor::new())
            .unwrap(),
    )
    .unwrap()
}
