//! Index selection only: these descriptive owner fixtures grant no current seal.
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::application_attempt::output_correspondence::WorthQueryCheckpointOutputRole;
pub use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture as WorthQueryApplicationOutputPosture;
pub use std::sync::Arc;
pub use std::collections::BTreeMap;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::computation_source::ComputationSourceEvidence;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::family_selection::checkpoint_priors::NativePriorCheckpointOutput;
pub use std::sync::OnceLock;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::output_lineage::ProductCoordinate;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::recorded_output::RecordedOutput;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::recorded_output::RecordedOutputMutable;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::output_lineage::settlement_identity::RecordedSettlementIdentity;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::output_lineage::semantic_source::SemanticSource;
pub use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId as TypeId;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::WorthQueryApplicationOutputCorrespondence;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::WorthQueryApplicationOutputLineage;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::WorthQueryCurrentOutputFamilyResolution;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::WorthQueryPriorOutputBindingResolution;
pub use crate::domain_computation::primary_graph::output_lineage::tests::checkpoint_identity;
pub use crate::domain_computation::primary_graph::output_lineage::tests::source_facts;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::computation_retention::prior_absence::PriorAbsence;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::output_binding_identity::OutputBindingIdentity;
pub use worth_relational::facade::identity::EntityId;
pub use worth_relational::facade::identity::PartitionId;

pub struct Initial;
pub struct Preserve;
pub struct OtherRole;

pub fn entity(number: u64) -> EntityId {
    EntityId::new(PartitionId::main(), number, 1)
}

pub struct Court {
    pub request: worth_execution::SerialRequest,
    pub lineage: WorthQueryApplicationOutputLineage,
    pub source: SemanticSource,
    pub coordinate: ProductCoordinate,
    pub child: worth_runtime_world::facade::ProductBranchIncarnation,
}

impl Court {
    pub fn new() -> Self {
        let world =
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
        let product = world
            .application
            .product_runtime()
            .admit_product_branch_for_test(
                world
                    .application
                    .product_runtime()
                    .default_branch_for_test(),
            )
            .unwrap();
        let observation = product.observation_for_test();
        use worth_runtime_world::facade::*;
        let intent = ProductBranchCreationIntent::from_source(
            "family-head-child",
            ProductBranchCreationPlans::new(
                RelationalBranchCreationPlan::ReuseExact,
                SignalBranchCreationPlan::ReuseExact,
            ),
        )
        .unwrap();
        let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host_request);
        let RuntimeWorldBranchCreationOutcome::Performed(child) = world
            .application
            .product_runtime()
            .create_product_branch_for_test(
                execution,
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
        lineage.fixture_binding(TypeId::of_for_test::<Initial>(), "Initial");
        lineage.fixture_binding(TypeId::of_for_test::<OtherRole>(), "OtherRole");
        lineage.fixture_binding(TypeId::of_for_test::<Preserve>(), "Preserve");

        lineage.install_output_families_for_test(BTreeMap::from([(
            "family".to_owned().into(),
            vec![
                (TypeId::of_for_test::<Initial>(), "output".to_owned()),
                (TypeId::of_for_test::<Preserve>(), "output".to_owned()),
            ],
        )]));
        Self {
            request: worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request(),
            lineage,
            source: SemanticSource::new_for_test((*world.application.runtime_for_test()).authority_identity_for_test().as_u64(), (*world.application.installed_schema_for_test()).binding_identity().clone(), worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(entity(1)), OutputBindingIdentity::declared_for_test("Initial")),
            coordinate: ProductCoordinate::new_for_test(observation.lifecycle_incarnation(), 10),
            child: child.lifecycle_incarnation(),
        }
    }

    pub fn publish<Binding: 'static>(
        &mut self,
        generation: u64,
        partition: Option<u8>,
        output: u64,
        posture: WorthQueryApplicationOutputPosture,
        facts: bool,
    ) -> Arc<WorthQueryApplicationOutputCorrespondence> {
        let source = {
            let __output_binding = self
                .lineage
                .binding_identity_for_test(TypeId::of_for_test::<Binding>())
                .unwrap();
            let mut __base = self.source.clone();
            *__base.output_binding_mut_for_test() = __output_binding;
            __base
        };
        let role = if TypeId::of_for_test::<Binding>() == TypeId::of_for_test::<OtherRole>() {
            "other"
        } else {
            "output"
        };
        let correspondence = Arc::new(
            WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles_for_test(
                TypeId::of_for_test::<Binding>(),
                TypeId::of_for_test::<()>(),
                std::iter::empty(),
                &[WorthQueryCheckpointOutputRole::new_for_test(
                    role.to_owned(),
                    posture,
                    "fixture".to_owned(),
                    entity(output),
                )],
                |_| Some(TypeId::of_for_test::<()>()),
                &worth_execution::ExecutionRequest::serial(&self.request),
            )
            .unwrap(),
        );
        let coordinate = {
            let __generation = generation;
            let mut __base = self.coordinate;
            *__base.generation_mut_for_test() = __generation;
            __base
        };
        let records = (*self.lineage.by_source_mut_for_test())
            .entry(source.clone())
            .or_default()
            .entry((*coordinate.occurrence_for_test()))
            .or_default()
            .entry(generation)
            .or_default();
        let slot = records.len();
        let computation_source = ComputationSourceEvidence::unavailable_for_test();
        let recorded = RecordedOutput::new_for_test(None, None, computation_source, None, None, Arc::from([]), None, None, None, OnceLock::new(), RecordedSettlementIdentity::retain_for_test(&source, coordinate, slot), Arc::clone(&correspondence), Some(checkpoint_identity([1; 32])), partition.map(|partition| [partition; 32]), None, [2; 32], std::sync::Mutex::new(RecordedOutputMutable::new_for_test(
                None,
                facts
                    .then(source_facts)
                    .map(|facts| computation_source.retain_facts_for_test(facts)),
                worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::retained_computation::RecordedComputation::Absent(PriorAbsence::NotProduced),
            )));
        let cell = Arc::new(OnceLock::new());
        assert!(cell.set(recorded).is_ok());
        records.push(cell);
        (*self.lineage.partition_index_mut_for_test()).insert_for_test(
            source,
            (*coordinate.occurrence_for_test()),
            generation,
            partition.map(|partition| [partition; 32]),
            slot,
        );
        correspondence
    }

    pub fn resolve(&self) -> WorthQueryCurrentOutputFamilyResolution {
        self.lineage
            .resolve_current_family_for_test(
                &worth_execution::ExecutionRequest::serial(&self.request),
                (*self.source.runtime_authority_for_test()),
                &(*self.source.schema_for_test()),
                self.source.scope,
                "family",
                (*self.coordinate.occurrence_for_test()),
                (*self.coordinate.generation_for_test()),
            )
            .expect("the installed family resolves")
    }

    pub fn inherit_settlement<Target: 'static, Origin: 'static>(
        &mut self,
        generation: u64,
        origin_generation: u64,
    ) {
        let base_source = self.source.clone();
        let bindings = &(*self.lineage.binding_identities_for_test());
        // The update's field value first, then its base, as the literal evaluated them.
        let source = |binding| {
            let output_binding = bindings.identity_for_test(binding).unwrap();
            let mut source = base_source.clone();
            (*source.output_binding_mut_for_test()) = output_binding;
            source
        };
        let origin = (*(*self.lineage.by_source_for_test())
            [&source(TypeId::of_for_test::<Origin>())][&(*self.coordinate.occurrence_for_test())]
            [&origin_generation][0]
            .get()
            .unwrap()
            .settlement_identity_for_test())
        .clone();
        // The key reads the bindings before the lineage is borrowed mutably.
        let target_source = source(TypeId::of_for_test::<Target>());
        let cell = &mut (*self.lineage.by_source_mut_for_test())
            .get_mut(&target_source)
            .unwrap()
            .get_mut(&(*self.coordinate.occurrence_for_test()))
            .unwrap()
            .get_mut(&generation)
            .unwrap()[0];
        (*Arc::get_mut(cell)
            .unwrap()
            .get_mut()
            .unwrap()
            .settlement_identity_mut_for_test()) = origin;
    }
}

#[path = "binding_succession/mod.rs"]
pub mod binding_succession;
#[path = "checkpoint_locator_copy/mod.rs"]
pub mod checkpoint_locator_copy;
#[path = "checkpoint_selection_denial/mod.rs"]
pub mod checkpoint_selection_denial;
#[path = "inherited_settlement/mod.rs"]
pub mod inherited_settlement;
#[path = "publication_controls/mod.rs"]
pub mod publication_controls;
#[path = "selection_budget/mod.rs"]
pub mod selection_budget;
