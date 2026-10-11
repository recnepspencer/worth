//! Index selection only: these descriptive owner fixtures grant no current seal.
use super::super::super::application_attempt::WorthQueryCheckpointOutputRole;
use super::super::super::WorthQueryApplicationOutputPosture;
use super::super::*;
use super::{checkpoint_identity, source_facts};
use crate::domain_computation::primary_graph::application_contribution::PriorAbsence;
use crate::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;
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
        lineage.fixture_binding(std::any::TypeId::of::<Initial>(), "Initial");
        lineage.fixture_binding(std::any::TypeId::of::<OtherRole>(), "OtherRole");
        lineage.fixture_binding(std::any::TypeId::of::<Preserve>(), "Preserve");

        lineage.install_output_families(BTreeMap::from([(
            "family".to_owned().into(),
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
                output_binding: OutputBindingIdentity::declared("Initial"),
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
            output_binding: self
                .lineage
                .binding_identity(TypeId::of::<Binding>())
                .unwrap(),
            ..self.source.clone()
        };
        let role = if TypeId::of::<Binding>() == TypeId::of::<OtherRole>() {
            "other"
        } else {
            "output"
        };
        let correspondence = Arc::new(
            WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles(
                TypeId::of::<Binding>(),
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
        let computation_source = ComputationSourceEvidence::unavailable();
        let recorded = RecordedOutput {
            computation_source,
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
            mutable: std::sync::Mutex::new(RecordedOutputMutable::new(
                None,
                facts
                    .then(source_facts)
                    .map(|facts| computation_source.retain_facts(facts)),
                None,
                retained_computation::RecordedComputation::Absent(PriorAbsence::NotProduced),
            )),
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
        let bindings = &self.lineage.binding_identities;
        let source = |binding| SemanticSource {
            output_binding: bindings.identity(binding).unwrap(),
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
