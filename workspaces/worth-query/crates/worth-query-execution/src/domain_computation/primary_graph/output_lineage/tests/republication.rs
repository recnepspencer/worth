//! A restored generated output continues the performed record it republishes.
//! A restored row that retained no performed proof stays Fresh until verified.

use std::{
    any::TypeId,
    collections::BTreeMap,
    sync::{Arc, Mutex, OnceLock},
};

use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::{
    identity::{EntityId, PartitionId},
    mvcc::CompanionPreflightBudget,
    runtime::PositionedRelationalSnapshot,
    transactions::{
        AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
        WorkerIntentBatch,
    },
};
use worth_runtime_world::facade::ProductBranchObservation;

use super::{checkpoint_identity, RestoredOutputBinding};
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryCheckpointOutputRole,
    invariant_projection::ConsumedOutputEvidence,
    output_lineage::{
        invalidation::InvalidationEditAdmission, ProductCoordinate, RecordedOutput,
        RecordedOutputMutable, RecordedSettlementIdentity, SealedNativeOutputWitness,
        SemanticSource, SourceInvalidationOwner, WorthQueryApplicationOutputCorrespondence,
        WorthQueryApplicationOutputLineage,
    },
    tests::fixture::{
        installed_authorization_world, live_scope, publish_relational_mutation, AccountLabel,
        AccountStatus,
    },
    WorthQueryApplicationObservedFact as Fact, WorthQueryApplicationOutputPosture,
    WorthQueryPrincipalResolutionMode,
};

mod records;
mod rules;

const PARTITION: [u8; 32] = [0x11; 32];
const IDEMPOTENCY_KEY: [u8; 32] = [0x41; 32];

type Witness = Arc<OnceLock<SealedNativeOutputWitness>>;

struct Stage<'world> {
    lineage: WorthQueryApplicationOutputLineage,
    source: SemanticSource,
    observation: &'world ProductBranchObservation,
    correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
    /// The witness the suspended performed record sealed.
    performed_witness: Witness,
    /// The witness sealed over the re-created output at the restoration.
    restored_witness: Witness,
    owner: &'world SourceInvalidationOwner,
    /// The restoration's committed read basis, taken by its registration.
    read_basis: Option<PositionedRelationalSnapshot>,
}

/// What a recorded republication hands to its mark registration.
struct Continued {
    identity: Arc<RecordedSettlementIdentity>,
    predecessor: Arc<RecordedSettlementIdentity>,
    facts: Arc<[Fact]>,
    consumed_outputs: Arc<[ConsumedOutputEvidence]>,
    witness: Witness,
}

fn staged(test: impl FnOnce(Stage<'_>)) {
    let world = installed_authorization_world(true);
    let output = world
        .selected_product()
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id();
    let graph = world.application.runtime.primary_graph().unwrap();
    let layout = graph.layout();
    // Two World publications put the restoration two product generations
    // after the occurrence's first, so earlier records have an address.
    let label = AccountLabel::reference();
    let label = layout
        .field_locator(label.entity(), label.aspect(), label.field())
        .unwrap();
    for value in ["performed", "suspended"] {
        publish_relational_mutation(
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
        .admit_product_branch(world.application.product_runtime().default_branch())
        .expect("the fixture's default product occurrence is live");
    let observation = product.observation();
    assert!(observation.reference_generation().get() >= 2);
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let correspondence = Arc::new(
        WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles(
            TypeId::of::<()>(),
            TypeId::of::<()>(),
            std::collections::BTreeSet::new(),
            vec![WorthQueryCheckpointOutputRole {
                role: "account".to_owned(),
                posture: WorthQueryApplicationOutputPosture::Preserve,
                entity_name: "Account".to_owned(),
                entity: output,
            }],
            |_| Some(TypeId::of::<()>()),
        )
        .unwrap(),
    );
    let (performed_witness, restored_witness, read_basis) = handle.with_runtime_mut(|runtime| {
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
                .exact_snapshot_live_entity_kind(&snapshot, output)
                .unwrap(),
        }];
        for aspect in layout.native_output_aspects("Account") {
            native.push(Fact::SourceAspectRevision {
                entity_id: output,
                aspect: aspect.clone(),
                native_revision: truth
                    .exact_snapshot_entity_aspect_version(&snapshot, output, aspect)
                    .unwrap(),
            });
        }
        let seal = || {
            SealedNativeOutputWitness::from_checkpoint_facts(
                &correspondence,
                layout,
                &native,
                owner,
                &mut owner.edit_admission(),
            )
            .unwrap()
            .expect("the installed output's native facts seal a witness")
        };
        let read_basis = truth.positioned_snapshot(&snapshot).unwrap();
        let sealed = (seal(), seal(), read_basis);
        runtime.snapshots().release_snapshot(&snapshot).unwrap();
        sealed
    });
    let source = SemanticSource {
        runtime_authority: world.application.runtime.authority_identity().as_u64(),
        schema: world.application.installed_schema.binding_identity(),
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(entity(1)),
        output_binding: TypeId::of::<RestoredOutputBinding>(),
    };
    test(Stage {
        lineage: WorthQueryApplicationOutputLineage::default(),
        source,
        observation,
        correspondence,
        performed_witness,
        restored_witness,
        owner,
        read_basis: Some(read_basis),
    });
}

impl Stage<'_> {
    fn restored_generation(&self) -> u64 {
        self.observation.reference_generation().get()
    }

    /// Retain one record in this stage's source partition before the
    /// restoration's generation.
    fn retain(
        &mut self,
        generation: u64,
        record: impl FnOnce(RecordedOutput) -> RecordedOutput,
        facts: &Arc<[Fact]>,
    ) -> Arc<OnceLock<RecordedOutput>> {
        assert!(generation < self.restored_generation());
        let occurrence = self.observation.lifecycle_incarnation();
        let records = self
            .lineage
            .by_source
            .entry(self.source.clone())
            .or_default()
            .entry(occurrence)
            .or_default()
            .entry(generation)
            .or_default();
        let slot = records.len();
        let cell = Arc::new(OnceLock::new());
        let recorded = record(RecordedOutput {
            native_prior_checkpoint: None,
            performed_origin: None,
            _retained_capacity: None,
            consumed_outputs: Arc::from([]),
            completed_handler_facts: None,
            completed_decision_reuse: None,
            prepared_input_reuse_key: None,
            native_output_witness: OnceLock::new(),
            mutable: Mutex::new(RecordedOutputMutable {
                verification_requirement: None,
                observed_source_facts: Some(Arc::clone(facts)),
                resources: None,
            }),
            settlement_identity: RecordedSettlementIdentity::retain(
                &self.source,
                ProductCoordinate {
                    occurrence,
                    generation,
                },
                slot,
            ),
            correspondence: Arc::clone(&self.correspondence),
            source_identity: Some(checkpoint_identity([0x31; 32])),
            source_partition_identity: Some(PARTITION),
            producer_dependency_identity: None,
            idempotency_key_identity: IDEMPOTENCY_KEY,
        });
        assert!(cell.set(recorded).is_ok());
        records.push(Arc::clone(&cell));
        self.lineage.partition_index.insert(
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
    fn republish(
        &mut self,
        suspended_generation: u64,
        facts: &Arc<[Fact]>,
        witness: &Witness,
        admission: &mut InvalidationEditAdmission,
    ) -> Option<Option<Continued>> {
        let republished = self.lineage.prepare_republication(
            self.source.output_binding,
            self.source.runtime_authority,
            &self.source.schema,
            self.source.scope,
            self.observation.lifecycle_incarnation(),
            suspended_generation,
            PARTITION,
            &self.correspondence,
            facts,
            Arc::clone(witness),
            admission,
        )?;
        let record = self.lineage.record_republished_restoration(
            self.source.output_binding,
            self.source.runtime_authority,
            self.source.schema.clone(),
            self.source.scope,
            self.observation,
            Arc::clone(&self.correspondence),
            checkpoint_identity([0x31; 32]),
            PARTITION,
            None,
            IDEMPOTENCY_KEY,
            Arc::clone(facts),
            None,
            republished,
        );
        Some(record.map(|record| Continued {
            identity: record.identity,
            predecessor: record.predecessor,
            facts: record.facts,
            consumed_outputs: record.consumed_outputs,
            witness: record.witness,
        }))
    }

    /// The only row at the restoration's own address.
    fn restored(&self) -> &RecordedOutput {
        let rows = &self.lineage.by_source[&self.source][&self.observation.lifecycle_incarnation()]
            [&self.restored_generation()];
        assert_eq!(rows.len(), 1, "one restoration address holds one row");
        rows[0].get().unwrap()
    }
}

fn entity(slot: u64) -> EntityId {
    EntityId::new(PartitionId::main(), slot, 1)
}

fn source_entity(slot: u64) -> Fact {
    Fact::SourceEntity {
        entity_id: entity(slot),
    }
}

fn selects(fact: &Fact, slot: u64) -> bool {
    matches!(fact, Fact::SourceEntity { entity_id } if *entity_id == entity(slot))
}

fn admission(work: u64) -> InvalidationEditAdmission {
    InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: work,
        maximum_preparation_bytes: 1024 * 1024,
    })
}
