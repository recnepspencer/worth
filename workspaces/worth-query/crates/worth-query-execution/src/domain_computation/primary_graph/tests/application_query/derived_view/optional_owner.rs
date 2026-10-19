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

mod optional_owner_removal_and_insertion_refresh_at_the_real_current;

fn batch() -> Batch {
    let nz = |value| NonZeroUsize::new(value).unwrap();
    Batch::new(Limits::new(
        nz(2),
        Some(nz(100_000)),
        nz(262_144),
        nz(4_096),
    ))
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

fn read(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    batch: Option<&Batch>,
) -> Read {
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let selected = world.selected_product(&(phase_custody).test_host_request());
    let principal = selected
        .resolve_authenticated_principal(
            phase_custody,
            &world.binding,
            &external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let scope = selected
        .resolve_entity(
            phase_custody,
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
        .admit_application_query(execution,
            &query,
            &access,
            ApplicationQueryParameterSet::new(),
            crate::domain_computation::primary_graph::WorthQueryApplicationQueryControls::product_one_shot(
                product.retained_clone(), basis,
                NonZeroUsize::new(10).unwrap(), Some(NonZeroUsize::new(10_000).unwrap()), &request,
            ),
        )
        .unwrap();
    match batch {
        Some(batch) => Read::Batched(
            world
                .application
                .execute_application_query_one_shot_in_batch(phase_custody, plan, batch)
                .unwrap(),
        ),
        None => Read::Ordinary(
            world
                .application
                .execute_application_query_one_shot(phase_custody, plan)
                .unwrap(),
        ),
    }
}

fn refresh(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    view: &WorthQueryManagedDerivedView<PublicAccountMembershipQuery, Owner>,
    key: &WorthQueryManagedDerivedViewKey,
) -> std::sync::Arc<Owner> {
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let selected = world.selected_product(&(phase_custody).test_host_request());
    let principal = selected
        .resolve_authenticated_principal(
            phase_custody,
            &world.binding,
            &external,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let scope = selected
        .resolve_entity(
            phase_custody,
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
        .admit_application_query(execution,
            &query,
            &access,
            ApplicationQueryParameterSet::new(),
            crate::domain_computation::primary_graph::WorthQueryApplicationQueryControls::product_one_shot(
                product.retained_clone(), basis,
                NonZeroUsize::new(10).unwrap(), Some(NonZeroUsize::new(10_000).unwrap()), &request,
            ),
        )
        .unwrap();
    world
        .application
        .refresh_managed_derived_collection_pair_entry_in_batch(
            execution,
            view,
            &product,
            key,
            plan,
            &batch(),
            |_, batch| {
                Ok::<_, BatchDenial>(read(phase_custody, execution, world, Some(batch)).batched())
            },
            |row| row.account,
            |row| row.account,
            |_, row| Owner(row.owner),
        )
        .unwrap()
}
