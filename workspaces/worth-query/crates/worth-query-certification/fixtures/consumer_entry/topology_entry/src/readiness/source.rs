// Exact existing readiness source contract for both fixture compositions.
use worth_query_decl::facade::{
    authoring::{
        AspectFieldSelector, AuthoredQueryBundleRequest, AuthoredResultShapeField,
        DetailQueryBuilder, DetailResultShapeBuilder, RootEntityKey,
    },
    binding::QueryBindingDescriptor,
    canonicalization::canonicalize_request,
};

pub(super) fn canonical_query() -> worth_query_decl::facade::canonicalization::CanonicalQueryBundle
{
    let query = DetailQueryBuilder::new(RootEntityKey::new(delivery_field().entity()).unwrap())
        .project(
            AspectFieldSelector::new(delivery_field().aspect(), delivery_field().field()).unwrap(),
        )
        .build()
        .unwrap()
        .into_raw();
    let shape = DetailResultShapeBuilder::new()
        .field(
            AuthoredResultShapeField::new(
                delivery_field().aspect(),
                delivery_field().field(),
                "length",
            )
            .unwrap(),
        )
        .build()
        .unwrap()
        .into_raw();
    canonicalize_request(
        AuthoredQueryBundleRequest::for_ordinary_read(query, shape, QueryBindingDescriptor::new())
            .unwrap(),
    )
    .unwrap()
}

pub(super) fn delivery_field() -> super::Length {
    super::Length::at("Length")
}
