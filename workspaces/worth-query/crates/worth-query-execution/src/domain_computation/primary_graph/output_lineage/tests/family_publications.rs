//! Index selection only: these descriptive owner fixtures grant no current seal.
use super::super::super::application_attempt::WorthQueryCheckpointOutputRole;
use super::super::super::WorthQueryApplicationOutputPosture;
use super::super::{
    ProductCoordinate, RecordedOutput, SemanticSource, WorthQueryApplicationOutputCorrespondence,
    WorthQueryApplicationOutputLineage, WorthQueryCurrentOutputFamilyResolution,
};
use super::{checkpoint_identity, source_facts};
use std::{
    any::TypeId,
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
use worth_relational::facade::identity::{EntityId, PartitionId};

struct Initial;
struct Preserve;
struct OtherRole;

fn entity(number: u64) -> EntityId {
    EntityId::new(PartitionId::main(), number, 1)
}

struct Court {
    lineage: WorthQueryApplicationOutputLineage,
    source: SemanticSource,
    coordinate: ProductCoordinate,
    child: worth_runtime_world::facade::ProductBranchIncarnation,
}

impl Court {
    fn new() -> Self {
        let world =
            crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
                true,
            );
        let product = world
            .application
            .product_runtime()
            .admit_product_branch(world.application.product_runtime().default_branch())
            .unwrap();
        let observation = product.observation();
        use worth_runtime_world::facade::*;
        let intent = ProductBranchCreationIntent::from_source(
            "family-head-child",
            ProductBranchCreationPlans::new(
                RelationalBranchCreationPlan::ReuseExact,
                SignalBranchCreationPlan::ReuseExact,
            ),
        )
        .unwrap();
        let RuntimeWorldBranchCreationOutcome::Performed(child) = world
            .application
            .product_runtime()
            .create_product_branch(
                &product,
                None,
                intent,
                &RuntimeWorldCancellationSource::new().token(),
            )
            .unwrap()
        else {
            panic!("real child occurrence");
        };
        let mut lineage = WorthQueryApplicationOutputLineage::default();
        lineage.install_output_families(BTreeMap::from([(
            "family".to_owned(),
            vec![
                (TypeId::of::<Initial>(), "output".to_owned()),
                (TypeId::of::<Preserve>(), "output".to_owned()),
            ],
        )]));
        Self {
            lineage,
            source: SemanticSource {
                runtime_authority: world.application.runtime.authority_identity().as_u64(),
                schema: world.application.installed_schema.binding_identity().clone(),
                scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(entity(1)),
                output_binding: TypeId::of::<Initial>(),
            },
            coordinate: ProductCoordinate { occurrence: observation.lifecycle_incarnation(), generation: 10 },
            child: child.lifecycle_incarnation(),
        }
    }

    fn publish<Binding: 'static>(
        &mut self,
        generation: u64,
        partition: Option<u8>,
        output: u64,
        posture: WorthQueryApplicationOutputPosture,
        facts: bool,
    ) -> Arc<WorthQueryApplicationOutputCorrespondence> {
        let source = SemanticSource {
            output_binding: TypeId::of::<Binding>(),
            ..self.source.clone()
        };
        let role = if source.output_binding == TypeId::of::<OtherRole>() {
            "other"
        } else {
            "output"
        };
        let correspondence = Arc::new(
            WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles(
                source.output_binding,
                TypeId::of::<()>(),
                BTreeSet::new(),
                vec![WorthQueryCheckpointOutputRole {
                    role: role.to_owned(),
                    posture,
                    entity_name: "fixture".to_owned(),
                    entity: entity(output),
                }],
                |_| Some(TypeId::of::<()>()),
            )
            .unwrap(),
        );
        let coordinate = ProductCoordinate {
            generation,
            ..self.coordinate
        };
        let records = self
            .lineage
            .by_source
            .entry(source.clone())
            .or_default()
            .entry(coordinate.occurrence)
            .or_default()
            .entry(generation)
            .or_default();
        let slot = records.len();
        let recorded = RecordedOutput {
            correspondence: Arc::clone(&correspondence),
            source_identity: Some(checkpoint_identity([1; 32])),
            source_partition_identity: partition.map(|partition| [partition; 32]),
            producer_dependency_identity: None,
            idempotency_key_identity: [2; 32],
            observed_source_facts: facts.then(source_facts),
            resources: None,
        };
        records.push(recorded);
        self.lineage.partition_index.insert(
            source,
            coordinate.occurrence,
            generation,
            partition.map(|partition| [partition; 32]),
            slot,
        );
        correspondence
    }

    fn resolve(&self) -> WorthQueryCurrentOutputFamilyResolution {
        self.resolve_budgeted(128).unwrap()
    }

    fn resolve_budgeted(
        &self,
        budget: usize,
    ) -> Result<WorthQueryCurrentOutputFamilyResolution, ()> {
        self.lineage.resolve_current_family(
            self.source.runtime_authority,
            &self.source.schema,
            self.source.scope,
            "family",
            self.coordinate.occurrence,
            self.coordinate.generation,
            budget,
        )
    }
}

#[test]
fn newer_preserve_publication_supersedes_initial_even_without_initial_facts() {
    for (partition, initial_facts) in [(Some(1), true), (Some(1), false), (None, true)] {
        let mut court = Court::new();
        court.publish::<Initial>(
            2,
            partition,
            2,
            WorthQueryApplicationOutputPosture::Create,
            initial_facts,
        );
        let preserve = court.publish::<Preserve>(
            3,
            partition,
            2,
            WorthQueryApplicationOutputPosture::Preserve,
            true,
        );
        let selected = court.resolve();
        assert_eq!(
            selected.candidates.len(),
            1,
            "the obsolete binding must not supply a retry packet"
        );
        assert!(Arc::ptr_eq(
            &selected.candidates[0].correspondence,
            &preserve
        ));
        assert!(!selected.ambiguous_publication);
    }
}

#[test]
fn retirement_and_descriptive_heads_cannot_resurrect_the_initial_binding() {
    for (posture, facts) in [
        (WorthQueryApplicationOutputPosture::Retire, false),
        (WorthQueryApplicationOutputPosture::Retire, true),
        (WorthQueryApplicationOutputPosture::Preserve, false),
    ] {
        let mut court = Court::new();
        court.publish::<Initial>(
            2,
            Some(1),
            2,
            WorthQueryApplicationOutputPosture::Create,
            true,
        );
        let head = court.publish::<Preserve>(3, Some(1), 2, posture, facts);
        let selected = court.resolve();
        assert!(!selected.ambiguous_publication);
        assert_eq!(selected.candidates.len(), usize::from(facts));
        if facts {
            assert!(Arc::ptr_eq(&selected.candidates[0].correspondence, &head));
            assert_eq!(
                selected.candidates[0]
                    .correspondence
                    .active_entity_for_role("output"),
                None
            );
        }
    }
}

#[test]
fn distinct_partitions_entities_and_roles_remain_distinct_family_members() {
    for (partition, output, other_role) in [
        (Some(2), 2, false),
        (None, 2, false),
        (Some(1), 3, false),
        (Some(1), 2, true),
    ] {
        let mut court = Court::new();
        let initial = court.publish::<Initial>(
            2,
            Some(1),
            2,
            WorthQueryApplicationOutputPosture::Create,
            true,
        );
        let sibling = if other_role {
            court
                .lineage
                .output_families
                .get_mut("family")
                .unwrap()
                .push((TypeId::of::<OtherRole>(), "other".to_owned()));
            court.publish::<OtherRole>(
                3,
                partition,
                output,
                WorthQueryApplicationOutputPosture::Preserve,
                true,
            )
        } else {
            court.publish::<Preserve>(
                3,
                partition,
                output,
                WorthQueryApplicationOutputPosture::Preserve,
                true,
            )
        };
        let selected = court.resolve();
        assert_eq!(selected.candidates.len(), 2);
        for expected in [&initial, &sibling] {
            assert!(selected
                .candidates
                .iter()
                .any(|candidate| Arc::ptr_eq(&candidate.correspondence, expected)));
        }
        assert!(!selected.ambiguous_publication);
    }
}

#[test]
fn equal_publication_positions_refuse_selection_in_both_binding_orders() {
    let mut court = Court::new();
    court.publish::<Initial>(
        2,
        Some(1),
        2,
        WorthQueryApplicationOutputPosture::Create,
        true,
    );
    court.publish::<Preserve>(
        2,
        Some(1),
        2,
        WorthQueryApplicationOutputPosture::Preserve,
        true,
    );
    for _ in 0..2 {
        let selected = court.resolve();
        assert!(selected.ambiguous_publication);
        assert!(
            selected.candidates.is_empty(),
            "no iteration-order retry packet"
        );
        court
            .lineage
            .output_families
            .get_mut("family")
            .unwrap()
            .reverse();
    }
}

#[test]
fn selected_descendant_supersedes_ancestor_even_with_lower_generation() {
    let mut court = Court::new();
    court.publish::<Initial>(
        9,
        Some(1),
        2,
        WorthQueryApplicationOutputPosture::Create,
        true,
    );
    let ancestor = court.coordinate;
    // Coordinates stage descriptive selection evidence; real native branch
    // incarnations supply ancestry identity without granting currentness.
    court.lineage.origins.insert(court.child, ancestor);
    court.coordinate = ProductCoordinate {
        occurrence: court.child,
        generation: 2,
    };
    let descendant = court.publish::<Preserve>(
        1,
        Some(1),
        2,
        WorthQueryApplicationOutputPosture::Preserve,
        true,
    );
    let selected = court.resolve();
    assert_eq!(selected.candidates.len(), 1);
    assert!(Arc::ptr_eq(
        &selected.candidates[0].correspondence,
        &descendant
    ));
    assert!(!selected.ambiguous_publication);
}

#[test]
fn family_publication_selection_charges_bounded_work_for_offered_heads() {
    for (population, expected_work) in [(1_u8, 15), (16, 584), (128, 8_728)] {
        let mut court = Court::new();
        let mut expected = Vec::new();
        for partition in 0..population {
            court.publish::<Initial>(
                1,
                Some(partition),
                u64::from(partition) + 2,
                WorthQueryApplicationOutputPosture::Create,
                true,
            );
            expected.push(court.publish::<Initial>(
                2,
                Some(partition),
                u64::from(partition) + 2,
                WorthQueryApplicationOutputPosture::Preserve,
                true,
            ));
        }
        assert!(
            court.resolve_budgeted(expected_work - 1).is_err(),
            "selection comparisons must consume the work budget"
        );
        let selected = court
            .resolve_budgeted(expected_work)
            .expect("the declared head-selection budget is sufficient");
        assert_eq!(selected.selection_work, expected_work);
        assert_eq!(selected.candidates.len(), usize::from(population));
        for (candidate, expected) in selected.candidates.iter().zip(&expected) {
            assert!(Arc::ptr_eq(&candidate.correspondence, expected));
        }
        assert!(!selected.ambiguous_publication);
    }
}
