pub use worth_foundational::facade::{AspectValue, CanonicalFieldPath, FieldKey, InternedString};
pub use worth_proof::TransitionOutcome;
pub use worth_query::facade::{domain, foundation, read, runtime};

pub use super::fixture::{bind, insert_matrix_value, matrix_workspace};
pub use super::samples::{matrix_aspect_key, matrix_value_with_order, sample_field};

pub type BoundCollection = domain::WorthQueryBoundCollection<
    crate::facade::query::suite::installed_operation_fixture::GeometryDomain,
    super::fixture::NativeMatrixRead,
    crate::facade::query::suite::installed_operation_fixture::ReadFamily,
    foundation::ObservationLaneWitness,
>;

pub fn bound_collection(
    workspace: &mut runtime::WorthQueryWorkspace,
) -> (BoundCollection, domain::WorthQueryNativeAccessKey) {
    let (settled, order_key) = settled_with_order_key(workspace);
    let collection = settled.into_bound_collection().unwrap();
    (collection, order_key)
}

pub fn settled_with_order_key(
    workspace: &mut runtime::WorthQueryWorkspace,
) -> (
    domain::WorthQuerySettledDomainProjection<
        crate::facade::query::suite::installed_operation_fixture::GeometryDomain,
        super::fixture::NativeMatrixRead,
        crate::facade::query::suite::installed_operation_fixture::ReadFamily,
        foundation::ObservationLaneWitness,
    >,
    domain::WorthQueryNativeAccessKey,
) {
    settled_with_native_field(workspace, 15)
}

pub fn settled_with_native_field(
    workspace: &mut runtime::WorthQueryWorkspace,
    field: usize,
) -> (
    domain::WorthQuerySettledDomainProjection<
        crate::facade::query::suite::installed_operation_fixture::GeometryDomain,
        super::fixture::NativeMatrixRead,
        crate::facade::query::suite::installed_operation_fixture::ReadFamily,
        foundation::ObservationLaneWitness,
    >,
    domain::WorthQueryNativeAccessKey,
) {
    let bound = bind(workspace);
    let mut request = bound
        .consumer_projection_contract()
        .unwrap()
        .projection_request();
    let order = request
        .select_derived_native_field(sample_field(field))
        .unwrap();
    let request = request.build().unwrap();
    let order_key = request.resolve_native_key(&order).unwrap().into_key();
    let settled = bound
        .admit_execution_resources(
            (),
            crate::facade::query::suite::installed_operation_fixture::execution_resource_request(),
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
        .unwrap();
    (settled, order_key)
}

pub fn first_window(
    collection: &BoundCollection,
    width: u32,
) -> domain::WorthQueryBoundCollectionWindow {
    let admitted = collection
        .declare_window(collection.beginning_cursor(), breadth(width))
        .unwrap();
    collection.resolve_window(admitted).unwrap()
}

pub fn breadth(width: u32) -> domain::WorthQueryCollectionWindowBreadth {
    domain::WorthQueryCollectionWindowBreadth::new(width, 0, 0, width).unwrap()
}

pub fn order_values(
    collection: &BoundCollection,
    window: &domain::WorthQueryBoundCollectionWindow,
    key: &domain::WorthQueryNativeAccessKey,
) -> Vec<String> {
    window
        .rows()
        .iter()
        .map(|row| {
            let value = collection.native_value(row, key).unwrap();
            match value.value().scalar() {
                Some(AspectValue::String(InternedString::Raw(value))) => value.clone(),
                other => panic!("unexpected order value: {other:?}"),
            }
        })
        .collect()
}

#[allow(dead_code)]
pub fn order_touch() -> runtime::WorthQueryAspectTouch {
    runtime::WorthQueryAspectTouch::aspect_field_path(
        matrix_aspect_key(),
        CanonicalFieldPath::single(FieldKey::new("f15").unwrap()),
    )
}
