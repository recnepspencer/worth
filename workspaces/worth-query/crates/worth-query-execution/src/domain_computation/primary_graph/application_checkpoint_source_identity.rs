use sha2::{Digest, Sha256};

/// Derives the portable identity only when immutable source meaning is first interned.
pub(super) fn checkpoint_source_identity(
    query: &[u8; 32],
    parameters: &[u8; 32],
    footprint: &super::application_query::observed_source::WorthQueryObservedSourceFootprint,
) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth-query:checkpoint-observed-source:v1");
    digest.update(query);
    digest.update(parameters);
    append_footprint(&mut digest, footprint);
    digest.finalize().into()
}

fn append_footprint(
    digest: &mut Sha256,
    footprint: &super::application_query::observed_source::WorthQueryObservedSourceFootprint,
) {
    entity(digest, footprint.root);
    digest.update([u8::from(footprint.complete)]);
    length(digest, footprint.entities.len());
    for observed in &footprint.entities {
        entity(digest, *observed);
    }
    length(digest, footprint.aspects.len());
    for aspect in &footprint.aspects {
        entity(digest, aspect.entity);
        text(digest, &aspect.entity_name);
        text(digest, aspect.aspect.as_str());
        digest.update(aspect.contract_revision.0.to_be_bytes());
        optional_u64(digest, aspect.native_revision);
    }
    length(digest, footprint.adjacencies.len());
    for adjacency in &footprint.adjacencies {
        entity(digest, adjacency.anchor);
        digest.update(adjacency.relation_kind.as_u32().to_be_bytes());
        digest.update([match adjacency.direction {
            worth_relational::facade::runtime::RelationalAdjacencyDirection::Outgoing => 0,
            worth_relational::facade::runtime::RelationalAdjacencyDirection::Incoming => 1,
        }]);
        optional_u64(digest, adjacency.native_revision.map(|revision| revision.0));
        length(digest, adjacency.comparison_work_limit);
        length(digest, adjacency.endpoints.len());
        for endpoint in &adjacency.endpoints {
            entity(digest, *endpoint);
        }
    }
    digest.update([u8::from(footprint.root_selection.is_some())]);
    if let Some(selection) = &footprint.root_selection {
        digest.update(selection.identity());
    }
}

fn entity(digest: &mut Sha256, entity: worth_relational::facade::identity::EntityId) {
    digest.update(entity.partition_value().to_be_bytes());
    digest.update(entity.local_slot_value().to_be_bytes());
    digest.update(entity.generation_value().to_be_bytes());
}

fn text(digest: &mut Sha256, value: &str) {
    length(digest, value.len());
    digest.update(value.as_bytes());
}

fn length(digest: &mut Sha256, value: usize) {
    digest.update(
        u64::try_from(value)
            .expect("source footprint lengths fit the canonical u64 carrier")
            .to_be_bytes(),
    );
}

fn optional_u64(digest: &mut Sha256, value: Option<u64>) {
    digest.update([u8::from(value.is_some())]);
    digest.update(value.unwrap_or_default().to_be_bytes());
}

#[cfg(test)]
mod tests {
    use worth_relational::facade::identity::{EntityId, PartitionId};

    use crate::domain_computation::primary_graph::application_query::observed_source::{
        source_identity::{WorthQueryObservedSourceEpoch, WorthQueryObservedSourceMeaningRegistry},
        WorthQueryObservedSourceFootprint,
    };
    use crate::domain_computation::primary_graph::application_query::resource_lifecycle::WorthQueryApplicationBasisSelectionIdentity;

    #[test]
    fn portable_identity_ignores_runtime_occurrence_but_changes_with_source_meaning() {
        let root = EntityId::new(PartitionId::main(), 1, 1);
        let first_selection = product_selection();
        let second_selection = product_selection();
        let registry = WorthQueryObservedSourceMeaningRegistry::new(21);
        let first = epoch(&registry, root, footprint(root, true), &first_selection);
        let second = epoch(&registry, root, footprint(root, true), &second_selection);

        assert_ne!(
            first.checkpoint_occurrence(),
            second.checkpoint_occurrence()
        );
        assert_eq!(first.checkpoint_identity(), second.checkpoint_identity(),);

        let changed = epoch(&registry, root, footprint(root, false), &first_selection);
        assert_ne!(first.checkpoint_identity(), changed.checkpoint_identity(),);

        let mut changed_selection = footprint(root, true);
        changed_selection.root_selection = Some(std::sync::Arc::new(
            crate::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedRootSelection::new(
                vec![root], Vec::new(), Vec::new(),
            ),
        ));
        let changed = epoch(&registry, root, changed_selection, &first_selection);
        assert_ne!(first.checkpoint_identity(), changed.checkpoint_identity());
    }

    fn epoch(
        registry: &WorthQueryObservedSourceMeaningRegistry,
        root: EntityId,
        footprint: WorthQueryObservedSourceFootprint,
        selection: &WorthQueryApplicationBasisSelectionIdentity,
    ) -> WorthQueryObservedSourceEpoch {
        let meaning = registry
            .intern(&[7; 32], &[8; 32], footprint, selection)
            .unwrap();
        WorthQueryObservedSourceEpoch::from_observation(
            &[7; 32], &[8; 32], root, selection, meaning,
        )
        .unwrap()
    }

    fn footprint(root: EntityId, complete: bool) -> WorthQueryObservedSourceFootprint {
        WorthQueryObservedSourceFootprint {
            root,
            complete,
            entities: vec![root],
            aspects: Vec::new(),
            adjacencies: Vec::new(),
            root_selection: None,
        }
    }

    fn product_selection() -> WorthQueryApplicationBasisSelectionIdentity {
        let world =
            crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
                true,
            );
        let product = world
            .application
            .product_runtime()
            .admit_product_branch(world.application.product_runtime().default_branch())
            .expect("the fixture product occurrence is live");
        WorthQueryApplicationBasisSelectionIdentity::Product(
            crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
                product.observation(),
            ),
        )
    }
}
