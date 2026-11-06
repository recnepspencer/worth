//! A restored generated output continues the performed record it republishes.
//! Performed opaque-reader and request-context rows continue without cutoff proofs.
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::output_binding_identity::OutputBindingIdentity;

pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::partitioned_computation::incremental::prior_absence::PriorAbsence;
pub use worth_query_execution::court_doors_for_test::equality_type_id::EqualityTypeId as TypeId;
pub use std::collections::BTreeMap;
pub use std::sync::Arc;
pub use std::sync::Mutex;
pub use std::sync::OnceLock;

pub use worth_foundational::facade::AspectValue;
pub use worth_foundational::facade::InternedString;
pub use worth_relational::facade::identity::EntityId;
pub use worth_relational::facade::identity::PartitionId;
pub use worth_relational::facade::runtime::PositionedRelationalSnapshot;
pub use worth_relational::facade::transactions::AspectFieldPatch;
pub use worth_relational::facade::transactions::EntityMutationIntent;
pub use worth_relational::facade::transactions::MutationIntent;
pub use worth_relational::facade::transactions::UpdateEntityFieldsIntent;
pub use worth_relational::facade::transactions::WorkerIntentBatch;
pub use worth_runtime_world::facade::ProductBranchObservation;

pub use crate::domain_computation::primary_graph::output_lineage::tests::checkpoint_identity;
pub use crate::domain_computation::primary_graph::output_lineage::tests::RestoredOutputBinding;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::application_attempt::output_correspondence::WorthQueryCheckpointOutputRole;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::consumed_output::ConsumedOutputEvidence;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::output_lineage::ProductCoordinate;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::recorded_output::RecordedOutput;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::recorded_output::RecordedOutputMutable;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::output_lineage::settlement_identity::RecordedSettlementIdentity;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::native_output_witness::SealedNativeOutputWitness;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::output_lineage::semantic_source::SemanticSource;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::application_attempt::output_correspondence::WorthQueryApplicationOutputCorrespondence;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::WorthQueryApplicationOutputLineage;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountLabel;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact as Fact;
pub use worth_query_declaration::facade::application_operation::ApplicationMutationOutputPosture as WorthQueryApplicationOutputPosture;
pub use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;

pub const PARTITION: [u8; 32] = [0x11; 32];
pub const IDEMPOTENCY_KEY: [u8; 32] = [0x41; 32];

pub type Witness = Arc<OnceLock<SealedNativeOutputWitness>>;

pub struct Stage<'world> {
    pub lineage: WorthQueryApplicationOutputLineage,
    pub source: SemanticSource,
    pub observation: &'world ProductBranchObservation,
    pub correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
    /// The witness the suspended performed record sealed.
    pub performed_witness: Witness,
    /// The witness sealed over the re-created output at the restoration.
    pub restored_witness: Witness,
    pub owner: &'world SourceInvalidationOwner,
    /// The restoration's committed read basis, taken by its registration.
    pub read_basis: Option<PositionedRelationalSnapshot>,
}

/// What a recorded republication hands to its mark registration.
pub struct Continued {
    pub identity: Arc<RecordedSettlementIdentity>,
    pub predecessor: Arc<RecordedSettlementIdentity>,
    pub facts: worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::source_facts::RetainedSourceFacts,
    pub consumed_outputs: Arc<[ConsumedOutputEvidence]>,
    pub witness: Witness,
}

pub fn staged(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    test: impl FnOnce(Stage<'_>),
) {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    world.application.with_host_advancement(|phase| {
    let phase_custody = &phase;
    let output = world
        .selected_product(&(phase_custody).test_host_request_for_test(), )
        .resolve_entity(phase_custody,
            AccountStatus::reference(),
            "open".to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id_for_test();
    let graph = (*world.application.runtime_for_test()).primary_graph_for_test().unwrap();
    let layout = graph.layout_for_test();
    // Two World publications put the restoration two product generations
    // after the occurrence's first, so earlier records have an address.
    let label = AccountLabel::reference();
    let label = layout
        .field_locator_for_test(label.entity(), label.aspect(), label.field())
        .unwrap();
    for value in ["performed", "suspended"] {
        publish_relational_mutation(&phase_custody.borrowed_request_for_test(),
            &world,
            WorkerIntentBatch::new("republication-generation").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: output,
                    fields: AspectFieldPatch::from(BTreeMap::from([(
                        label.clone(),
                        AspectValue::String(InternedString::Raw(value.to_owned())),
                    )])),
                }),
            )),
        );
    }
    let product = world
        .application
        .product_runtime()
        .admit_product_branch_for_test(world.application.product_runtime().default_branch_for_test())
        .expect("the fixture's default product occurrence is live");
    let observation = product.observation_for_test();
    assert!(observation.reference_generation().get() >= 2);
    let handle = graph.integration_handle_for_test();
    let owner = &(*(*handle.source_owner_for_test()).invalidation_owner_for_test());
    let correspondence = Arc::new(
        WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles_for_test(
TypeId::of_for_test::<()>(),
TypeId::of_for_test::<()>(),
std::iter::empty(),
&[WorthQueryCheckpointOutputRole::new_for_test("account".to_owned(), WorthQueryApplicationOutputPosture::Preserve, "Account".to_owned(), output)],
|_| Some(TypeId::of_for_test::<()>()),
&phase_custody.borrowed_request_for_test(),
)
        .unwrap(),
    );
    let (performed_witness, restored_witness, read_basis) = handle.with_runtime_mut_for_test(request_execution, |runtime| {
        let basis = runtime
            .admit_branch_basis(&runtime.main_branch_identity())
            .unwrap();
        let snapshot = runtime
            .snapshots()
            .snapshot_for_observation(&basis.observation())
            .unwrap();
        let truth = runtime.read_truth();
        let mut native = vec![Fact::Entity {
            entity_id: output,
            kind: truth
                 .read_exact_snapshot_live_entity_kind(request_execution, &snapshot, output).unwrap()
                .unwrap(),
        }];
        for aspect in layout.native_output_aspects_for_test("Account") {
            native.push(Fact::SourceAspectRevision {
                entity_id: output,
                aspect: aspect.clone(),
                native_revision: truth
                     .read_exact_snapshot_entity_aspect_version(request_execution, &snapshot, output, aspect).unwrap()
                    .unwrap(),
            });
        }
let seal = || {
            SealedNativeOutputWitness::from_checkpoint_facts_for_test(&correspondence, layout, &native)
            .unwrap()
            .expect("the installed output's native facts seal a witness")
        };


        let read_basis = truth.positioned_snapshot(&snapshot).unwrap();
        let sealed = (seal(), seal(), read_basis);
        runtime.snapshots().release_snapshot(&snapshot).unwrap();
        sealed
    });
    let source = SemanticSource::new_for_test((*world.application.runtime_for_test()).authority_identity_for_test().as_u64(), (*world.application.installed_schema_for_test()).binding_identity(), worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(entity(1)), OutputBindingIdentity::declared_for_test("RestoredOutputBinding"));
    let mut lineage = WorthQueryApplicationOutputLineage::default();
    lineage.fixture_binding(
        TypeId::of_for_test::<RestoredOutputBinding>(),
        "RestoredOutputBinding",
    );
    test(Stage {
        lineage,
        source,
        observation,
        correspondence,
        performed_witness,
        restored_witness,
        owner,
        read_basis: Some(read_basis),
    });
    }).expect("the republication fixture opens its installed host call");
}

impl Stage<'_> {
    pub fn restored_generation(&self) -> u64 {
        self.observation.reference_generation().get()
    }

    /// Retain one record in this stage's source partition before the
    /// restoration's generation.
    pub fn retain(
        &mut self,
        generation: u64,
        record: impl FnOnce(RecordedOutput) -> RecordedOutput,
        facts: &Arc<[Fact]>,
    ) -> Arc<OnceLock<RecordedOutput>> {
        assert!(generation < self.restored_generation());
        let occurrence = self.observation.lifecycle_incarnation();
        let records = (*self.lineage.by_source_mut_for_test())
            .entry(self.source.clone())
            .or_default()
            .entry(occurrence)
            .or_default()
            .entry(generation)
            .or_default();
        let slot = records.len();
        let cell = Arc::new(OnceLock::new());
        let recorded = record({
            // Bound in the original literal order, passed in the door's.
            let (producing_implementation, required_publication, native_prior_checkpoint, performed_origin, consumed_outputs, completed_handler_facts, completed_decision_reuse, prepared_input_reuse_key, native_output_witness, computation_source, mutable, settlement_identity, correspondence, source_identity, source_partition_identity, producer_dependency_identity, idempotency_key_identity) = (None, None, None, None, Arc::from([]), None, None, None, OnceLock::new(), worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::computation_source::ComputationSourceEvidence::for_test(false), Mutex::new(RecordedOutputMutable::new_for_test(None, (Some(Arc::clone(facts))).map(|facts| worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::computation_source::ComputationSourceEvidence::for_test(false).retain_facts_for_test(facts)), worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::retained_computation::RecordedComputation::Absent(PriorAbsence::Restored))), RecordedSettlementIdentity::retain_for_test(
                &self.source,
                ProductCoordinate::new_for_test(occurrence, generation),
                slot,
            ), Arc::clone(&self.correspondence), Some(checkpoint_identity([0x31; 32])), Some(PARTITION), None, IDEMPOTENCY_KEY);
            RecordedOutput::new_for_test(
                producing_implementation,
                required_publication,
                computation_source,
                native_prior_checkpoint,
                performed_origin,
                consumed_outputs,
                completed_handler_facts,
                completed_decision_reuse,
                prepared_input_reuse_key,
                native_output_witness,
                settlement_identity,
                correspondence,
                source_identity,
                source_partition_identity,
                producer_dependency_identity,
                idempotency_key_identity,
                mutable,
            )
        });
        assert!(cell.set(recorded).is_ok());
        records.push(Arc::clone(&cell));
        (*self.lineage.partition_index_mut_for_test()).insert_for_test(
            self.source.clone(),
            occurrence,
            generation,
            Some(PARTITION),
            slot,
        );
        cell
    }

    /// Restore the record suspended at `suspended_generation` with `facts`.
    /// `None`: that record has nothing to continue. `Some(None)`: no new row
    /// took the republication.
    pub fn republish(
        &mut self,
        suspended_generation: u64,
        facts: &Arc<[Fact]>,
        witness: &Witness,
    ) -> Option<Option<Continued>> {
        let republished = self.lineage.prepare_republication_for_test(self.lineage.binding_type_for_test(&(*self.source.output_binding_for_test())), (*self.source.runtime_authority_for_test()), &(*self.source.schema_for_test()), self.source.scope, self.observation.lifecycle_incarnation(), suspended_generation, PARTITION, &self.correspondence, &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::source_facts::RetainedSourceFacts::for_test_for_test(false, Arc::clone(facts)), Arc::clone(witness), worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::control::StorageControl::new_for_test(worth_execution::ExecutionAllocationPolicy::SystemAllocation, None))?;
        let record = self.lineage.record_republished_restoration_for_test(
            self.lineage.binding_type_for_test(&(*self.source.output_binding_for_test())),
            (*self.source.runtime_authority_for_test()),
            (*self.source.schema_for_test()).clone(),
            self.source.scope,
            self.observation,
            Arc::clone(&self.correspondence),
            checkpoint_identity([0x31; 32]),
            PARTITION,
            None,
            IDEMPOTENCY_KEY,
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::source_facts::RetainedSourceFacts::for_test_for_test(
                false,
                Arc::clone(facts),
            ),
            republished,
        );
        Some(record.map(|record| {
            let (identity, predecessor, facts, consumed_outputs, witness) =
                record.into_parts_for_test();
            Continued {
                identity,
                predecessor,
                facts,
                consumed_outputs,
                witness,
            }
        }))
    }

    /// The only row at the restoration's own address.
    pub fn restored(&self) -> &RecordedOutput {
        let rows = &(*self.lineage.by_source_for_test())[&self.source]
            [&self.observation.lifecycle_incarnation()][&self.restored_generation()];
        assert_eq!(rows.len(), 1, "one restoration address holds one row");
        rows[0].get().unwrap()
    }
}

pub fn entity(slot: u64) -> EntityId {
    EntityId::new(PartitionId::main(), slot, 1)
}

pub fn source_entity(slot: u64) -> Fact {
    Fact::SourceEntity {
        entity_id: entity(slot),
    }
}

pub fn selects(fact: &Fact, slot: u64) -> bool {
    matches!(fact, Fact::SourceEntity { entity_id } if *entity_id == entity(slot))
}

#[path = "absence/mod.rs"]
pub mod absence;
#[path = "records/mod.rs"]
pub mod records;
#[path = "rules/mod.rs"]
pub mod rules;
