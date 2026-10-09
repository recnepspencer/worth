//! Index selection only: these descriptive owner fixtures grant no current seal.
use super::super::super::application_attempt::WorthQueryCheckpointOutputRole;
use super::super::super::WorthQueryApplicationOutputPosture;
use super::super::*;
use super::{checkpoint_identity, source_facts};
use worth_relational::facade::identity::{EntityId, PartitionId};

mod checkpoint_locator_copy;
mod checkpoint_selection_denial;
mod inherited_settlement;
mod publication_controls;
mod selection_budget;

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
            native_prior_checkpoint: None,
            _retained_capacity: None,
            performed_origin: None,
            consumed_outputs: Arc::from([]),
            completed_handler_facts: None,
            completed_decision_reuse: None,
            prepared_input_reuse_key: None,
            native_output_witness: OnceLock::new(),
            settlement_identity: RecordedSettlementIdentity::retain(&source, coordinate, slot),
            correspondence: Arc::clone(&correspondence),
            source_identity: Some(checkpoint_identity([1; 32])),
            source_partition_identity: partition.map(|partition| [partition; 32]),
            producer_dependency_identity: None,
            idempotency_key_identity: [2; 32],
            mutable: std::sync::Mutex::new(RecordedOutputMutable {
                verification_requirement: None,
                observed_source_facts: facts.then(source_facts),
                resources: None,
            }),
        };
        let cell = Arc::new(OnceLock::new());
        assert!(cell.set(recorded).is_ok());
        records.push(cell);
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

    fn inherit_settlement<Target: 'static, Origin: 'static>(
        &mut self,
        generation: u64,
        origin_generation: u64,
    ) {
        let base_source = self.source.clone();
        let source = |binding| SemanticSource {
            output_binding: binding,
            ..base_source.clone()
        };
        let origin = self.lineage.by_source[&source(TypeId::of::<Origin>())]
            [&self.coordinate.occurrence][&origin_generation][0]
            .get()
            .unwrap()
            .settlement_identity
            .clone();
        let cell = &mut self
            .lineage
            .by_source
            .get_mut(&source(TypeId::of::<Target>()))
            .unwrap()
            .get_mut(&self.coordinate.occurrence)
            .unwrap()
            .get_mut(&generation)
            .unwrap()[0];
        Arc::get_mut(cell)
            .unwrap()
            .get_mut()
            .unwrap()
            .settlement_identity = origin;
    }
}
