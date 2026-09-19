//! Certification-only construction for real Query projection worlds.
//!
//! The fixture installs the production WORTH UI domain package and operation
//! executors. It does not construct binding, fact, patch, or authority outcomes.

mod bounded_seed;

pub fn scalar_projection_workspace(
    supports_async_lifecycle: bool,
) -> worth_query::facade::runtime::WorthQueryWorkspace {
    crate::scalar_text_projection_fixture::projection_workspace(supports_async_lifecycle)
}

pub fn remasked_scalar_projection_workspace() -> worth_query::facade::runtime::WorthQueryWorkspace {
    crate::scalar_text_projection_fixture::remasked_projection_workspace()
}

pub fn collection_projection_workspace() -> worth_query::facade::runtime::WorthQueryWorkspace {
    crate::scalar_text_projection_fixture::collection_projection_workspace()
}

pub fn collection_projection_workspace_without_entity_lookup(
) -> worth_query::facade::runtime::WorthQueryWorkspace {
    crate::scalar_text_projection_fixture::collection_projection_workspace_without_entity_lookup()
}

pub fn collection_projection_workspace_without_dependency_impact(
) -> worth_query::facade::runtime::WorthQueryWorkspace {
    crate::scalar_text_projection_fixture::collection_projection_workspace_without_dependency_impact(
    )
}

pub fn partial_collection_projection_workspace() -> worth_query::facade::runtime::WorthQueryWorkspace
{
    crate::scalar_text_projection_fixture::partial_collection_projection_workspace()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiCollectionProjectionSeedPosture {
    Complete,
    Partial,
    ResetOnly,
}

pub fn seeded_collection_projection_workspace(
    rows: Vec<(String, String)>,
    posture: WorthUiCollectionProjectionSeedPosture,
) -> (
    worth_query::facade::runtime::WorthQueryWorkspace,
    Vec<worth_query::facade::foundation::WorthQueryEntityIdentity>,
) {
    if rows.is_empty() {
        return (empty_collection_projection_workspace(posture), Vec::new());
    }
    if rows.len() > bounded_seed::MAX_ATOMIC_SEED_ROWS {
        return bounded_seed::collection_projection_workspace(rows, posture);
    }
    let (workspace, seed) =
        crate::scalar_text_projection_fixture::seeded_collection_projection_workspace(
            rows.clone(),
            posture == WorthUiCollectionProjectionSeedPosture::Partial,
            posture != WorthUiCollectionProjectionSeedPosture::ResetOnly,
            false,
        );
    let entities = rows
        .iter()
        .map(|(identity, _)| {
            seed.entity(identity)
                .expect("every authored projection seed has a Query entity")
                .clone()
        })
        .collect();
    (workspace, entities)
}

fn empty_collection_projection_workspace(
    posture: WorthUiCollectionProjectionSeedPosture,
) -> worth_query::facade::runtime::WorthQueryWorkspace {
    match posture {
        WorthUiCollectionProjectionSeedPosture::Complete => collection_projection_workspace(),
        WorthUiCollectionProjectionSeedPosture::Partial => {
            partial_collection_projection_workspace()
        }
        WorthUiCollectionProjectionSeedPosture::ResetOnly => {
            collection_projection_workspace_without_entity_lookup()
        }
    }
}

pub fn seeded_collection_projection_workspace_with_item_keys(
    rows: Vec<(String, String, u64)>,
    posture: WorthUiCollectionProjectionSeedPosture,
) -> (
    worth_query::facade::runtime::WorthQueryWorkspace,
    Vec<worth_query::facade::foundation::WorthQueryEntityIdentity>,
) {
    let identities = rows
        .iter()
        .map(|(identity, _, _)| identity.clone())
        .collect::<Vec<_>>();
    let (workspace, seed) =
        crate::scalar_text_projection_fixture::seeded_collection_projection_workspace_with_item_keys(
            rows,
            posture == WorthUiCollectionProjectionSeedPosture::Partial,
            posture != WorthUiCollectionProjectionSeedPosture::ResetOnly,
            false,
        );
    let entities = identities
        .iter()
        .map(|identity| {
            seed.entity(identity)
                .expect("every authored projection seed has a Query entity")
                .clone()
        })
        .collect();
    (workspace, entities)
}

pub fn seeded_mixed_projection_workspace(
    rows: Vec<(String, String)>,
) -> (
    worth_query::facade::runtime::WorthQueryWorkspace,
    Vec<worth_query::facade::foundation::WorthQueryEntityIdentity>,
) {
    let (workspace, seed) =
        crate::scalar_text_projection_fixture::seeded_collection_projection_workspace(
            rows.clone(),
            false,
            true,
            true,
        );
    let entities = rows
        .iter()
        .map(|(identity, _)| {
            seed.entity(identity)
                .expect("every mixed projection seed has a Query entity")
                .clone()
        })
        .collect();
    (workspace, entities)
}

pub fn insert_projection_status(
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
    identity: &str,
    status: &str,
) -> worth_query::facade::foundation::WorthQueryEntityIdentity {
    crate::scalar_text_projection_fixture::insert_collection_status(workspace, identity, status)
}

pub fn remove_projection_entity(
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
    entity: worth_query::facade::foundation::WorthQueryEntityIdentity,
) {
    workspace
        .delete(entity)
        .expect("certification projection entity deletion");
}

pub fn update_projection_status(
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
    entity: worth_query::facade::foundation::WorthQueryEntityIdentity,
    status: &str,
) {
    crate::scalar_text_projection_fixture::update_status(workspace, entity, status);
}

pub fn update_projection_status_batch(
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
    updates: Vec<(
        worth_query::facade::foundation::WorthQueryEntityIdentity,
        String,
    )>,
) {
    let commands = updates
        .into_iter()
        .fold(
            worth_query::facade::runtime::WorthQueryMutationBatchBuilder::new(),
            |batch, (entity, status)| {
                batch.update(entity, |row| {
                    row
                        .set_aspect(
                            worth_query::facade::runtime::WorthQueryAspectTouch::from_authoring_ingress_text(
                                "query_text.status",
                            )
                            .expect("projection status touch"),
                            worth_query::facade::runtime::WorthQueryAuthoredAspectValue::string(
                                status.clone(),
                            ),
                        )
                        .set_aspect(
                            worth_query::facade::runtime::WorthQueryAspectTouch::from_authoring_ingress_text(
                                "collection_item.status",
                            )
                            .expect("collection status touch"),
                            worth_query::facade::runtime::WorthQueryAuthoredAspectValue::string(status),
                        )
                })
            },
        )
        .build()
        .expect("QP04 update batch declaration");
    workspace
        .write_batch_intent(commands)
        .execute()
        .expect("QP04 atomic Query update batch");
}

pub fn update_projection_identity(
    workspace: &mut worth_query::facade::runtime::WorthQueryWorkspace,
    entity: worth_query::facade::foundation::WorthQueryEntityIdentity,
    identity: &str,
) {
    crate::scalar_text_projection_fixture::update_identity(workspace, entity, identity);
}
