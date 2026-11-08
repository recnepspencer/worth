//! Actual canonical readiness query for a fixture output field.
pub(in super::super) fn readiness_query(
    entity: &str,
    aspect: &str,
    field: &str,
) -> worth_query_decl::facade::canonicalization::CanonicalQueryBundle {
    use worth_query_decl::facade::{
        authoring::*, binding::QueryBindingDescriptor, canonicalization::canonicalize_request,
    };
    let query = DetailQueryBuilder::new(RootEntityKey::new(entity).unwrap())
        .project(AspectFieldSelector::new(aspect, field).unwrap())
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(AuthoredResultShapeField::new(aspect, field, "value").unwrap())
        .build()
        .unwrap()
        .into_raw();
    canonicalize_request(
        AuthoredQueryBundleRequest::for_ordinary_read(query, shape, QueryBindingDescriptor::new())
            .unwrap(),
    )
    .unwrap()
}
