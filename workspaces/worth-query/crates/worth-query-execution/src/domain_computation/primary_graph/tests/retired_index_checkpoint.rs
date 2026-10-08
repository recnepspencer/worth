//! A store persisted before the inbound-completion correlation index was
//! retired still opens and runs under the current code.
//!
//! The retired definition and its generation stay in the native checkpoint.
//! Readmission binds Query's own indexes by name, kind and scope, so the extra
//! definition binds to nothing; publications refresh only Query's installed
//! index set, so nothing maintains or requires the retired one.

use crate::facade::application_installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;
use std::collections::BTreeMap;
use std::time::Duration;

use worth_query_declaration::facade::application_schema::{
    ApplicationScalarValueBinding, U64ApplicationValueBinding,
};
use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus;
use worth_relational::facade::history::RelationalCommitReceipt;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::indexes::{
    DerivedIndexBuildRequest, DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind,
};
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::transactions::{
    AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};

use super::fixture::{
    installed_world, live_scope, publish_relational_mutation_on_application, restored_world,
    IdentityWorld,
};
use crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionMode;

/// The exact definition the retired code registered.
const RETIRED_CORRELATION_INDEX: &str = "worth-query-provider.inbound-completion-correlation";

#[test]
fn a_store_holding_the_retired_correlation_index_opens_and_runs() {
    let world = installed_world(&[("alice", WorthQueryPrincipalMappingStatus::Enabled)]);
    let principal = resolve_alice(&world, 1);
    set_principal_identity(&world, principal, 2);
    let retired = persist_as_the_retired_code_did(&world);
    let checkpoint = world
        .application
        .capture_application_checkpoint(CapturePolicy::SystemAllocation)
        .expect("the store with the retired index captures");
    drop(world);

    let restored = restored_world(checkpoint)
        .expect("a store holding the retired index opens under the current code");
    assert_retired_index_is_inert(&restored, retired);
    let principal = resolve_alice(&restored, 2);
    // The fixture publication asserts every installed index refreshes.
    set_principal_identity(&restored, principal, 3);
    let head = with_runtime(&restored, main_head);
    with_runtime(&restored, |runtime| {
        assert!(
            runtime
                .index_access()
                .published_generation_for_commit(retired, &head)
                .is_none(),
            "nothing maintains the retired index"
        );
    });
    resolve_alice(&restored, 3);

    // The retired index now lags head; capture and readmission still hold.
    let checkpoint = restored
        .application
        .capture_application_checkpoint(CapturePolicy::SystemAllocation)
        .expect("a store with a lagging retired index captures");
    drop(restored);
    let reopened = restored_world(checkpoint).expect("the lagging retired index still reopens");
    assert_retired_index_is_inert(&reopened, retired);
    resolve_alice(&reopened, 3);
}

/// Registers the retired definition and builds its generation at the
/// committed head, the state every store written by the retired code holds.
pub(super) fn persist_as_the_retired_code_did(world: &IdentityWorld) -> DerivedIndexId {
    let graph = world.application.runtime.primary_graph().unwrap();
    let correlation = graph
        .layout
        .provider_inbound_completion()
        .correlation
        .clone();
    graph.integration_handle().with_runtime_mut(|runtime| {
        let head = main_head(runtime);
        let retired = runtime.index_authority().register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: RETIRED_CORRELATION_INDEX.to_owned(),
            kind: DerivedIndexKind::EntityField {
                field_locator: correlation,
            },
            branch_scoped: false,
        });
        let built = runtime
            .index_authority()
            .build_for_commit(DerivedIndexBuildRequest {
                source_commit_id: head.commit_id,
                branch_id: head.branch_id.clone(),
                index_ids: vec![retired.index_id],
            });
        assert_eq!(built.generations.len(), 1, "{built:?}");
        retired.index_id
    })
}

pub(super) fn assert_retired_index_is_inert(world: &IdentityWorld, retired: DerivedIndexId) {
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    assert!(!handle.primary_index_ids.contains(&retired));
    handle.with_runtime(|runtime| {
        let lookup = runtime.index_access().definition_lookup_snapshot();
        assert_eq!(
            lookup.candidate_count_for_name(RETIRED_CORRELATION_INDEX),
            1,
            "the persisted definition is readmitted, not dropped"
        );
    });
}

pub(super) fn resolve_alice(world: &IdentityWorld, identity: u64) -> EntityId {
    let scope = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &scope);
    let principal = world
        .selected_product()
        .resolve_authenticated_principal(
            &world.binding,
            &external,
            &scope,
            WorthQueryPrincipalResolutionMode::Certification,
        )
        .expect("the installed identity index answers");
    assert_eq!(*principal.principal_identity(), identity);
    principal.principal_entity_id()
}

pub(super) fn set_principal_identity(world: &IdentityWorld, principal: EntityId, identity: u64) {
    let graph = world.application.runtime.primary_graph().unwrap();
    let locator = graph
        .layout
        .principal_binding(world.binding.binding())
        .unwrap()
        .principal_identity_locator
        .clone();
    publish_relational_mutation_on_application(
        &world.application,
        WorkerIntentBatch::new("set-principal-identity").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: principal,
                fields: AspectFieldPatch::from(BTreeMap::from([(
                    locator,
                    U64ApplicationValueBinding::encode(&identity).unwrap(),
                )])),
            }),
        )),
    );
}

pub(super) fn with_runtime<T>(
    world: &IdentityWorld,
    read: impl FnOnce(&RelationalRuntime) -> T,
) -> T {
    let graph = world.application.runtime.primary_graph().unwrap();
    graph.integration_handle().with_runtime(read)
}

pub(super) fn main_head(runtime: &RelationalRuntime) -> RelationalCommitReceipt {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .expect("the main branch is observable");
    basis
        .observation()
        .commit_receipt()
        .expect("the main branch has a committed head")
        .clone()
}
