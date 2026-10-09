//! Incoming-owner absence and presence invalidate only the genuine account entry.

use super::*;
use crate::domain_computation::primary_graph::tests::fixture::{
    AccountOwner, AuthorizationWorld, OptionalAccountOwnerQuery, OptionalAccountOwnerResult,
    PublicAccountMembershipQuery,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationOneShotResult, WorthQueryApplicationQueryBatchAdmission as Batch,
    WorthQueryApplicationQueryBatchLimits as Limits,
    WorthQueryManagedDerivedCollectionBatchRefreshDenial as BatchDenial,
    WorthQueryManagedDerivedView, WorthQueryManagedDerivedViewKey,
};
use std::num::NonZeroUsize;
use worth_relational::facade::identity::{EntityId, PartitionId};
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    CreateIntent, DeleteRelationIntent, EntityReference, RelationMutationIntent, RelationSpec,
};

mod membership;

struct Owner(Option<EntityId>);
impl WorthQueryManagedDerivedValue for Owner {
    fn retained_bytes(&self) -> usize {
        0
    }
}

#[test]
fn optional_owner_removal_and_insertion_refresh_at_the_real_current_product() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let execution = serial_request(&request);
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let selected = world.selected_product();
    let principal = selected
        .resolve_authenticated_principal(
            &world.binding,
            &external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let scope = selected
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_owned(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
    let query = world
        .application
        .installed_schema()
        .certification_query(OptionalAccountOwnerQuery::reference())
        .unwrap();
    let census = world
        .application
        .installed_schema()
        .certification_query(PublicAccountMembershipQuery::reference())
        .unwrap();
    let view = world
        .application
        .open_managed_derived_collection_pair(
            &ApplicationDerivedViewDefinition::new(
                "optional-owner",
                PublicAccountMembershipQuery::reference(),
                ApplicationDerivedViewLimits::bounded(8, 65_536),
            ),
            &census,
            &query,
            &query,
            selected.product(),
        )
        .unwrap();
    let membership = membership::read(&world);
    let account = membership.rows()[0].members[0];
    let owner = read(&world, None).ordinary().rows()[0]
        .owner
        .expect("fixture has one native ownership edge");
    let keys = world
        .application
        .reconstruct_managed_derived_collection_pair(
            worth_execution::ExecutionRequest::serial(&execution),
            &view,
            selected.product(),
            &membership,
            |row| {
                row.members
                    .iter()
                    .map(|root| (*root, row.tag.clone()))
                    .collect()
            },
            |_| {
                let first = selected
                    .retain_selection()
                    .unwrap()
                    .admit_application_query(
                        &query,
                        &access,
                        ApplicationQueryParameterSet::new(),
                        current_controls(&request),
                    )
                    .map_err(|_| WorthQueryManagedDerivedViewDenial::QueryExecutionDenied)?;
                let second = selected
                    .retain_selection()
                    .unwrap()
                    .admit_application_query(
                        &query,
                        &access,
                        ApplicationQueryParameterSet::new(),
                        current_controls(&request),
                    )
                    .map_err(|_| WorthQueryManagedDerivedViewDenial::QueryExecutionDenied)?;
                Ok(WorthQueryDerivedPairReadPlans::new(first, second))
            },
            |row| row.account,
            |row| row.account,
            |_, row| Owner(row.owner),
        )
        .unwrap();
    let key = &keys[0];
    assert_eq!(
        world
            .application
            .observe_managed_derived_view(&view)
            .unwrap()
            .get(key)
            .unwrap()
            .unwrap()
            .0,
        Some(owner)
    );
    let graph = world.application.runtime.primary_graph().unwrap();
    let kind = graph
        .layout
        .relation(AccountOwner::reference().name())
        .unwrap()
        .kind;
    let edge = graph.integration_handle().with_runtime_mut(|runtime| {
        runtime
            .read_truth()
            .visible_relations_of_kind(
                kind,
                selected.application_basis().snapshot_handle().version_id(),
            )
            .into_iter()
            .find(|edge| edge.target == account)
            .unwrap()
            .relation_id
    });
    let removed = graph.managed_derived_views().prepare_publication(
        publication_basis(selected.product()),
        &[ViewChange::Adjacency(account, kind, 1)],
        100_000,
    );
    drop(selected);
    super::super::super::fixture::publish_relational_mutation(
        &world,
        WorkerIntentBatch::new("remove-optional-owner").push(MutationIntent::Relation(
            RelationMutationIntent::Delete(DeleteRelationIntent { relation_id: edge }),
        )),
    );
    let absent = world.selected_product();
    removed.apply(absent.product().selected_commit());
    assert_eq!(
        world
            .application
            .observe_managed_derived_view(&view)
            .unwrap()
            .get(key)
            .err(),
        Some(WorthQueryManagedDerivedViewDenial::EntryRefreshRequired)
    );
    assert_eq!(refresh(&world, &view, key).0, None);
    let inserted = graph.managed_derived_views().prepare_publication(
        publication_basis(absent.product()),
        &[ViewChange::Adjacency(account, kind, 1)],
        100_000,
    );
    drop(absent);
    super::super::super::fixture::publish_relational_mutation(
        &world,
        WorkerIntentBatch::new("insert-optional-owner").push(MutationIntent::Create(
            CreateIntent::Relation(RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: kind,
                client_key: ClientKey::raw("restored-owner"),
                source: EntityReference::Existing(owner),
                target: EntityReference::Existing(account),
                fields: AspectFieldPatch::default(),
            }),
        )),
    );
    let present = world.selected_product();
    inserted.apply(present.product().selected_commit());
    assert_eq!(
        world
            .application
            .observe_managed_derived_view(&view)
            .unwrap()
            .get(key)
            .err(),
        Some(WorthQueryManagedDerivedViewDenial::EntryRefreshRequired)
    );
    assert_eq!(refresh(&world, &view, key).0, Some(owner));
}

fn batch() -> Batch {
    let nz = |value| NonZeroUsize::new(value).unwrap();
    Batch::new(Limits::new(nz(2), nz(100_000), nz(262_144), nz(4_096)))
}

enum Read {
    Ordinary(
        WorthQueryApplicationOneShotResult<OptionalAccountOwnerQuery, OptionalAccountOwnerResult>,
    ),
    Batched(
        crate::domain_computation::primary_graph::WorthQueryApplicationBatchResult<
            OptionalAccountOwnerQuery,
            OptionalAccountOwnerResult,
        >,
    ),
}

impl Read {
    fn ordinary(
        self,
    ) -> WorthQueryApplicationOneShotResult<OptionalAccountOwnerQuery, OptionalAccountOwnerResult>
    {
        match self {
            Self::Ordinary(result) => result,
            Self::Batched(_) => unreachable!("ordinary caller"),
        }
    }
    fn batched(
        self,
    ) -> crate::domain_computation::primary_graph::WorthQueryApplicationBatchResult<
        OptionalAccountOwnerQuery,
        OptionalAccountOwnerResult,
    > {
        match self {
            Self::Batched(result) => result,
            Self::Ordinary(_) => unreachable!("batch caller"),
        }
    }
}

fn read(world: &AuthorizationWorld, batch: Option<&Batch>) -> Read {
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let selected = world.selected_product();
    let principal = selected
        .resolve_authenticated_principal(
            &world.binding,
            &external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let scope = selected
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_owned(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let query = world
        .application
        .installed_schema()
        .certification_query(OptionalAccountOwnerQuery::reference())
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
    let (application, product, basis) = selected.into_parts();
    let plan = application
        .admit_application_query(
            &query,
            &access,
            ApplicationQueryParameterSet::new(),
            crate::domain_computation::primary_graph::WorthQueryApplicationQueryControls::product_one_shot(
                product.retained_clone(), basis,
                NonZeroUsize::new(10).unwrap(), NonZeroUsize::new(10_000).unwrap(), &request,
            ),
        )
        .unwrap();
    match batch {
        Some(batch) => Read::Batched(
            world
                .application
                .execute_application_query_one_shot_in_batch(plan, batch)
                .unwrap(),
        ),
        None => Read::Ordinary(
            world
                .application
                .execute_application_query_one_shot(plan)
                .unwrap(),
        ),
    }
}

fn refresh(
    world: &AuthorizationWorld,
    view: &WorthQueryManagedDerivedView<PublicAccountMembershipQuery, Owner>,
    key: &WorthQueryManagedDerivedViewKey,
) -> std::sync::Arc<Owner> {
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let selected = world.selected_product();
    let principal = selected
        .resolve_authenticated_principal(
            &world.binding,
            &external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let scope = selected
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_owned(),
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let query = world
        .application
        .installed_schema()
        .certification_query(OptionalAccountOwnerQuery::reference())
        .unwrap();
    let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
    let (application, product, basis) = selected.into_parts();
    let plan = application
        .admit_application_query(
            &query,
            &access,
            ApplicationQueryParameterSet::new(),
            crate::domain_computation::primary_graph::WorthQueryApplicationQueryControls::product_one_shot(
                product.retained_clone(), basis,
                NonZeroUsize::new(10).unwrap(), NonZeroUsize::new(10_000).unwrap(), &request,
            ),
        )
        .unwrap();
    world
        .application
        .refresh_managed_derived_collection_pair_entry_in_batch(
            view,
            &product,
            key,
            plan,
            &batch(),
            |_, batch| Ok::<_, BatchDenial>(read(world, Some(batch)).batched()),
            |row| row.account,
            |row| row.account,
            |_, row| Owner(row.owner),
        )
        .unwrap()
}
