use sha2::{Digest, Sha256};

use super::super::resource_lifecycle::WorthQueryApplicationBasisSelectionIdentity;
use super::WorthQueryObservedSourceFootprint;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryObservedSourceEpoch {
    query: [u8; 32],
    parameters: [u8; 32],
    root: worth_relational::facade::identity::EntityId,
    occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    observation_generation: u64,
    identity: [u8; 32],
}

impl WorthQueryObservedSourceEpoch {
    pub(in crate::domain_computation::primary_graph) const fn new(
        query: [u8; 32],
        parameters: [u8; 32],
        root: worth_relational::facade::identity::EntityId,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        observation_generation: u64,
        identity: [u8; 32],
    ) -> Self {
        Self {
            query,
            parameters,
            root,
            occurrence,
            observation_generation,
            identity,
        }
    }

    pub(super) fn from_observation(
        query: &[u8; 32],
        parameters: &[u8; 32],
        footprint: &WorthQueryObservedSourceFootprint,
        selection: &WorthQueryApplicationBasisSelectionIdentity,
        identity: [u8; 32],
    ) -> Option<Self> {
        let WorthQueryApplicationBasisSelectionIdentity::Product(product) = selection else {
            return None;
        };
        Some(Self::new(
            *query,
            *parameters,
            footprint.root,
            product.lifecycle_incarnation(),
            product.reference_generation().get(),
            identity,
        ))
    }

    pub(in crate::domain_computation::primary_graph) fn same_occurrence(
        &self,
        other: &Self,
    ) -> bool {
        self.query == other.query
            && self.parameters == other.parameters
            && self.root == other.root
            && self.occurrence == other.occurrence
    }

    pub(in crate::domain_computation::primary_graph) fn same_semantic_source(
        &self,
        other: &Self,
    ) -> bool {
        self.same_occurrence(other) && self.identity == other.identity
    }

    pub(in crate::domain_computation::primary_graph) fn replacement_order(
        &self,
        other: &Self,
    ) -> Option<std::cmp::Ordering> {
        self.same_occurrence(other).then(|| {
            if self.same_semantic_source(other) {
                std::cmp::Ordering::Equal
            } else {
                self.observation_generation
                    .cmp(&other.observation_generation)
            }
        })
    }

    pub(in crate::domain_computation::primary_graph) const fn observation_generation(&self) -> u64 {
        self.observation_generation
    }
}

pub(in crate::domain_computation::primary_graph) fn derive_source_identity(
    query: &[u8; 32],
    parameters: &[u8; 32],
    footprint: &WorthQueryObservedSourceFootprint,
    selection: &WorthQueryApplicationBasisSelectionIdentity,
) -> [u8; 32] {
    derive_observation_identity(
        b"worth-query:observed-source:v4",
        query,
        parameters,
        footprint,
        selection,
    )
}

pub(in crate::domain_computation) fn derive_partition_identity(
    query: &[u8; 32],
    parameters: &[u8; 32],
) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth-query:observed-source-partition:v1");
    digest.update(query);
    digest.update(parameters);
    digest.finalize().into()
}

pub(in crate::domain_computation::primary_graph) fn derive_result_set_identity(
    query: &[u8; 32],
    parameters: &[u8; 32],
    footprint: &WorthQueryObservedSourceFootprint,
    selection: &WorthQueryApplicationBasisSelectionIdentity,
) -> [u8; 32] {
    derive_observation_identity(
        b"worth-query:observed-result-set:v1",
        query,
        parameters,
        footprint,
        selection,
    )
}

fn derive_observation_identity(
    domain: &[u8],
    query: &[u8; 32],
    parameters: &[u8; 32],
    footprint: &WorthQueryObservedSourceFootprint,
    selection: &WorthQueryApplicationBasisSelectionIdentity,
) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(domain);
    digest.update(query);
    digest.update(parameters);
    match selection {
        WorthQueryApplicationBasisSelectionIdentity::Relational => digest.update([0]),
        WorthQueryApplicationBasisSelectionIdentity::Product(product) => {
            digest.update([1]);
            digest.update(product.lifecycle_incarnation().ordinal().to_be_bytes());
        }
    }
    entity(&mut digest, footprint.root);
    digest.update([u8::from(footprint.complete)]);
    length(&mut digest, footprint.entities.len());
    for observed in &footprint.entities {
        entity(&mut digest, *observed);
    }
    length(&mut digest, footprint.aspects.len());
    for aspect in &footprint.aspects {
        entity(&mut digest, aspect.entity);
        text(&mut digest, &aspect.entity_name);
        text(&mut digest, aspect.aspect.as_str());
        digest.update(aspect.contract_revision.0.to_be_bytes());
        optional_u64(&mut digest, aspect.native_revision);
    }
    length(&mut digest, footprint.adjacencies.len());
    for adjacency in &footprint.adjacencies {
        entity(&mut digest, adjacency.anchor);
        digest.update(adjacency.relation_kind.as_u32().to_be_bytes());
        digest.update([match adjacency.direction {
            worth_relational::facade::runtime::RelationalAdjacencyDirection::Outgoing => 0,
            worth_relational::facade::runtime::RelationalAdjacencyDirection::Incoming => 1,
        }]);
        optional_u64(
            &mut digest,
            adjacency.native_revision.map(|revision| revision.0),
        );
        length(&mut digest, adjacency.comparison_work_limit);
        length(&mut digest, adjacency.endpoints.len());
        for endpoint in &adjacency.endpoints {
            entity(&mut digest, *endpoint);
        }
    }
    match &footprint.root_selection {
        Some(selection) => {
            digest.update([1]);
            digest.update(selection.identity());
        }
        None => digest.update([0]),
    }
    digest.finalize().into()
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
    use worth_foundational::facade::{AspectContractRevision, AspectKey};
    use worth_relational::facade::identity::{EntityId, PartitionId};

    use super::*;
    use crate::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedAspectRevision;

    #[test]
    fn an_aspect_change_below_an_existing_maximum_changes_the_source_epoch() {
        let root = EntityId::new(PartitionId::main(), 1, 1);
        let dominant = aspect(root, "dominant", 90);
        let prior = aspect(root, "changed", 40);
        let mut next = footprint(root, vec![dominant.clone(), prior]);
        let selection = product_selection();
        let query = [7; 32];
        let parameters = [8; 32];
        let prior_identity = derive_source_identity(&query, &parameters, &next, &selection);
        let prior_maximum = next
            .aspects
            .iter()
            .filter_map(|aspect| aspect.native_revision)
            .max();

        next.aspects = vec![dominant, aspect(root, "changed", 41)];
        next.aspects
            .sort_by(|left, right| (left.entity, &left.aspect).cmp(&(right.entity, &right.aspect)));

        assert_eq!(
            prior_maximum,
            next.aspects
                .iter()
                .filter_map(|aspect| aspect.native_revision)
                .max()
        );
        assert_ne!(
            prior_identity,
            derive_source_identity(&query, &parameters, &next, &selection)
        );
        let mut equivalent = next.clone();
        equivalent.aspects.reverse();
        equivalent
            .aspects
            .sort_by(|left, right| (left.entity, &left.aspect).cmp(&(right.entity, &right.aspect)));
        assert_eq!(
            derive_source_identity(&query, &parameters, &next, &selection),
            derive_source_identity(&query, &parameters, &equivalent, &selection)
        );
    }

    #[test]
    fn parameter_partitions_are_distinct_source_occurrences() {
        let root = EntityId::new(PartitionId::main(), 1, 1);
        let selection = product_selection();
        let footprint = footprint(root, vec![aspect(root, "shared", 1)]);
        let lower = WorthQueryObservedSourceEpoch::from_observation(
            &[7; 32], &[1; 32], &footprint, &selection, [3; 32],
        )
        .unwrap();
        let upper = WorthQueryObservedSourceEpoch::from_observation(
            &[7; 32], &[2; 32], &footprint, &selection, [4; 32],
        )
        .unwrap();
        assert!(!lower.same_occurrence(&upper));
        assert_ne!(
            derive_source_identity(&[7; 32], &[1; 32], &footprint, &selection),
            derive_source_identity(&[7; 32], &[2; 32], &footprint, &selection),
        );
    }

    fn footprint(
        root: EntityId,
        mut aspects: Vec<WorthQueryObservedAspectRevision>,
    ) -> WorthQueryObservedSourceFootprint {
        aspects
            .sort_by(|left, right| (left.entity, &left.aspect).cmp(&(right.entity, &right.aspect)));
        WorthQueryObservedSourceFootprint {
            root,
            complete: true,
            entities: vec![root],
            aspects,
            adjacencies: Vec::new(),
            root_selection: None,
        }
    }

    fn aspect(
        entity: EntityId,
        name: &str,
        native_revision: u64,
    ) -> WorthQueryObservedAspectRevision {
        WorthQueryObservedAspectRevision {
            entity,
            entity_name: "Body".to_owned(),
            aspect: AspectKey::new(name).unwrap(),
            contract_revision: AspectContractRevision(1),
            native_revision: Some(native_revision),
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
