//! Real census growth retains clean entry custody without a caller size gate.
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::{
    AccountOwner, OwnerMembersQuery, OwnerMembersResult, PrincipalIdentityField,
    PublicScopedAccountSummaryQuery,
};
use std::sync::Arc;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{CreateIntent, EntityReference, RelationSpec};

struct ImageLabel {
    label: String,
    charge: usize,
}
impl WorthQueryManagedDerivedValue for ImageLabel {
    fn retained_bytes(&self) -> usize {
        self.charge
    }
}

#[test]
fn owner_sized_image_grows_from_actual_census_and_preserves_clean_arcs() {
    let world = installed_authorization_world(false);
    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let query = world
        .application
        .installed_schema()
        .certification_query(OwnerMembersQuery::reference())
        .unwrap();
    let entry = world
        .application
        .installed_schema()
        .certification_query(PublicScopedAccountSummaryQuery::reference())
        .unwrap();
    let resolve = |label: &str| {
        world
            .selected_product()
            .resolve_entity(
                AccountLabel::reference(),
                label.to_owned(),
                &request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap()
    };
    let primary = resolve("primary");
    let secondary = resolve("secondary");
    let owner = world
        .selected_product()
        .resolve_entity(
            PrincipalIdentityField::reference(),
            1_u64,
            &request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let graph = world.application.runtime.primary_graph().unwrap();
    let kind = graph
        .layout
        .relation(AccountOwner::reference().name())
        .unwrap()
        .kind;
    let bind = |account, key| {
        super::super::super::fixture::publish_relational_mutation(
            &world,
            WorkerIntentBatch::new("actual-census-member").push(MutationIntent::Create(
                CreateIntent::Relation(RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: kind,
                    client_key: ClientKey::raw(key),
                    source: EntityReference::Existing(owner.entity_id()),
                    target: EntityReference::Existing(account),
                    fields: AspectFieldPatch::default(),
                }),
            )),
        );
    };
    bind(primary.entity_id(), "first-owner-member");
    let selected = world.selected_product();
    let view = world
        .application
        .open_owner_sized_managed_derived_collection_pair(
            "owner-sized-census",
            &query,
            &entry,
            &entry,
            selected.product(),
        )
        .unwrap();
    let read_census = || {
        let current = world.selected_product();
        let principal = current
            .resolve_authenticated_principal(
                &world.binding,
                &external,
                &request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let scope = current
            .resolve_entity(
                PrincipalIdentityField::reference(),
                1_u64,
                &request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
        world
            .application
            .execute_application_query_one_shot(
                current
                    .admit_application_query(
                        &query,
                        &access,
                        ApplicationQueryParameterSet::new(),
                        current_controls(&request),
                    )
                    .unwrap(),
            )
            .unwrap()
    };
    let read_entry = |label: &str| {
        let current = world.selected_product();
        let principal = current
            .resolve_authenticated_principal(
                &world.binding,
                &external,
                &request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let scope = resolve(label);
        let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
        world
            .application
            .execute_application_query_one_shot(
                current
                    .admit_application_query(
                        &entry,
                        &access,
                        ApplicationQueryParameterSet::new(),
                        current_controls(&request),
                    )
                    .unwrap(),
            )
            .map_err(|_| WorthQueryManagedDerivedViewDenial::QueryExecutionDenied)
    };
    // Native IDs and labels both come from the actual declared owner adjacency.
    let members = |row: &OwnerMembersResult| row.members.clone();
    let census = read_census();
    assert_eq!(census.rows().len(), 1);
    assert_eq!(census.rows()[0].members.len(), 1);
    let keys = world
        .application
        .reconstruct_managed_derived_collection_pair_lazy(
            &view,
            selected.product(),
            &census,
            members,
            |label: &String| read_entry(label),
            |row| read_entry(row.label()),
            |row| row.label().to_owned(),
            |row| row.label().to_owned(),
            |_, row| ImageLabel {
                label: row.label().to_owned(),
                charge: row.label().len(),
            },
        )
        .unwrap();
    assert_eq!(keys[0].root(), primary.entity_id());
    let before = world
        .application
        .observe_managed_derived_view(&view)
        .unwrap();
    let original = before.get(&keys[0]).unwrap().unwrap();
    let quote = before.storage_quote().unwrap();
    assert_eq!(quote.entries(), 1);
    assert!(quote.retained_bytes() > 0);

    let prepared = graph.managed_derived_views().prepare_publication(
        publication_basis(selected.product()),
        &[ViewChange::Adjacency(owner.entity_id(), kind, 0)],
        100_000,
    );
    bind(secondary.entity_id(), "second-owner-member");
    let after = world.selected_product();
    prepared.apply(after.product().selected_commit());
    let census = read_census();
    assert_eq!(census.rows().len(), 1);
    assert_eq!(census.rows()[0].members.len(), 2);
    // A consumer's oversized logical payload quote must refuse the complete
    // image before installation. It does not require an enormous allocation.
    let refusal = world
        .application
        .reconcile_managed_derived_collection_pair_lazy(
            &view,
            after.product(),
            &census,
            members,
            |label: &String| read_entry(label),
            |row| read_entry(row.label()),
            |row| row.label().to_owned(),
            |row| row.label().to_owned(),
            |_, row| ImageLabel {
                label: row.label().to_owned(),
                charge: usize::MAX,
            },
        )
        .err();
    assert_eq!(
        refusal,
        Some(WorthQueryManagedDerivedViewDenial::RetainedBytesExceeded)
    );
    let refused = world
        .application
        .observe_managed_derived_view(&view)
        .unwrap();
    assert_eq!(refused.storage_quote().unwrap(), quote);
    assert_eq!(
        refused.get(&keys[0]).err(),
        Some(WorthQueryManagedDerivedViewDenial::MembershipReconciliationRequired)
    );
    assert_eq!(original.label, "primary");
    let mut new_reads = 0;
    let reconciled = world
        .application
        .reconcile_managed_derived_collection_pair_lazy(
            &view,
            after.product(),
            &census,
            members,
            |label: &String| {
                new_reads += 1;
                read_entry(label)
            },
            |row| read_entry(row.label()),
            |row| row.label().to_owned(),
            |row| row.label().to_owned(),
            |_, row| ImageLabel {
                label: row.label().to_owned(),
                charge: row.label().len(),
            },
        )
        .unwrap();
    assert_eq!(new_reads, 1);
    assert_eq!(reconciled.retained_entries(), 1);
    assert_eq!(reconciled.refreshed_entries(), 1);
    let current = world
        .application
        .observe_managed_derived_view(&view)
        .unwrap();
    assert!(Arc::ptr_eq(
        &original,
        &current.get(&keys[0]).unwrap().unwrap()
    ));
    let grown = current.storage_quote().unwrap();
    assert_eq!(grown.entries(), 2);
    assert!(grown.retained_bytes() > quote.retained_bytes());
    assert_eq!(
        before.storage_quote().err(),
        Some(WorthQueryManagedDerivedViewDenial::StaleSource)
    );
    assert_eq!(
        reconciled
            .keys()
            .iter()
            .filter(|key| key.root() == secondary.entity_id())
            .count(),
        1
    );
}
