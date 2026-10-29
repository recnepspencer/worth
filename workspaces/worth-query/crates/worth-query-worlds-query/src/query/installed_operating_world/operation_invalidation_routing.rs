pub use worth_query::facade::{domain, foundation, runtime};

pub use super::installed_operation_fixture::collection_impact::{
    impact_collection_invalidation_workspace, ImpactCollectionRead,
};
pub use super::installed_operation_fixture::configured_runtime;
pub use super::installed_operation_fixture::{
    consume_empty_invalidation_epoch as consume_empty_epoch, settle_native,
    InvalidationLease as Lease,
};
pub use super::installed_operation_fixture::{GeometryDomain, ReadFamily};

pub fn shared_pair(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut runtime::WorthQueryWorkspace,
) -> (Lease, Lease) {
    let live = match settle_native(workspace)
        .into_lifecycle()
        .promote(request_execution, workspace)
    {
        domain::WorthQueryProjectionPromotionOutcome::Promoted(live) => live,
        _ => panic!("indexed routing subject did not promote"),
    };
    let candidate = settle_native(workspace).into_lifecycle();
    match live.share_with(candidate, workspace) {
        domain::WorthQueryProjectionSharingOutcome::Shared(shared) => shared.into_leases(),
        domain::WorthQueryProjectionSharingOutcome::Stopped(stop) => {
            panic!("indexed routing sharing stopped: {}", stop.detail())
        }
    }
}

pub fn settle_installed_ordering(
    workspace: &mut runtime::WorthQueryWorkspace,
) -> domain::WorthQuerySettledDomainProjection<
    GeometryDomain,
    ImpactCollectionRead,
    ReadFamily,
    foundation::ObservationLaneWitness,
> {
    let installed = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, ImpactCollectionRead)
        .unwrap();
    let consumer = bound.consumer_projection_contract().unwrap();
    bound
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
        .consume(
            consumer,
            worth_query::facade::read::project_facts().entity_identities(),
        )
        .unwrap()
        .settle()
        .unwrap()
}

pub fn ordinary_name_view(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    workspace: &mut runtime::WorthQueryWorkspace,
    ordinal: usize,
) -> runtime::WorthQueryLiveView<runtime::WorthQueryUnrefinedLiveShape> {
    let request = foundation::DeclarativeLiveQueryRequest::new(
        "Vertex",
        foundation::DeclarativeLiveViewShape::table(),
    )
    .project(
        foundation::DeclarativeProjectionField::new(
            foundation::AspectFieldKey::from_authoring_parts("identity", "name").unwrap(),
        )
        .delivered_as("identity.name"),
    );
    let schema = runtime::QuerySchemaView::new(
        format!("same-aspect-name-{ordinal}"),
        [
            runtime::SchemaFieldView::new(
                foundation::AspectName::new("identity").unwrap(),
                foundation::FieldName::new("id").unwrap(),
                runtime::ScalarAspectType::String,
            ),
            runtime::SchemaFieldView::new(
                foundation::AspectName::new("identity").unwrap(),
                foundation::FieldName::new("name").unwrap(),
                runtime::ScalarAspectType::String,
            ),
        ],
        [],
    );
    workspace
        .live_view_request(
            request_execution,
            format!("ordinary-name-{ordinal}"),
            request,
            schema,
        )
        .unwrap()
}
