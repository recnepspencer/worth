use super::*;

pub(super) const DEFAULT_ROCKETSHIP_NODE_COUNT: usize = 100_000;
pub(super) const DEFAULT_ROCKETSHIP_QUERY_TARGET_COUNT: usize = 256;
pub(super) const ROCKETSHIP_PARTITION_WIDTH: usize = 32;
pub(super) const ROCKETSHIP_CHUNK_SIZE: usize = 4_096;
pub(super) const ROCKETSHIP_RELATION_SEED_BATCH_SIZE: usize = 16_000;
pub(super) const ROCKETSHIP_SUBSYSTEM_ENTITY_PARTITION_FANOUT: usize = 8;

#[derive(Debug)]
pub(super) struct RocketshipSeedOutcome {
    pub(super) entities: Vec<crate::facade::identity::EntityId>,
    pub(super) relation_count: usize,
    pub(super) entity_commit_micros: u128,
    pub(super) relation_commit_micros: u128,
    pub(super) relation_commit_phase_timing: crate::transactions::data::CommitPhaseTiming,
}

#[derive(Debug)]
pub(super) struct RocketshipPseudoRealisticSeedOutcome {
    pub(super) entities: Vec<crate::facade::identity::EntityId>,
    pub(super) mixed_query_targets: Vec<RecordRef>,
    pub(super) traversal_seeds: Vec<crate::facade::identity::EntityId>,
    pub(super) hot_update_target: crate::facade::identity::EntityId,
    pub(super) relation_count: usize,
    pub(super) subsystem_count: usize,
    pub(super) entity_commit_micros: u128,
    pub(super) relation_commit_micros: u128,
    pub(super) relation_commit_phase_timing: crate::transactions::data::CommitPhaseTiming,
}

#[derive(Clone, Copy)]
pub(super) struct RocketshipSubsystemLayout {
    pub(super) section: &'static str,
    pub(super) subsystem: &'static str,
    pub(super) weight: usize,
    pub(super) partition_base: u32,
}

pub(super) const ROCKETSHIP_SUBSYSTEM_LAYOUTS: [RocketshipSubsystemLayout; 12] = [
    RocketshipSubsystemLayout {
        section: "nose",
        subsystem: "guidance",
        weight: 4,
        partition_base: 11,
    },
    RocketshipSubsystemLayout {
        section: "nose",
        subsystem: "avionics",
        weight: 5,
        partition_base: 21,
    },
    RocketshipSubsystemLayout {
        section: "upper_stage",
        subsystem: "cargo_fairing",
        weight: 6,
        partition_base: 31,
    },
    RocketshipSubsystemLayout {
        section: "upper_stage",
        subsystem: "lox_tank",
        weight: 9,
        partition_base: 41,
    },
    RocketshipSubsystemLayout {
        section: "upper_stage",
        subsystem: "methane_tank",
        weight: 9,
        partition_base: 51,
    },
    RocketshipSubsystemLayout {
        section: "interstage",
        subsystem: "separation_ring",
        weight: 5,
        partition_base: 61,
    },
    RocketshipSubsystemLayout {
        section: "booster",
        subsystem: "forward_tank",
        weight: 12,
        partition_base: 71,
    },
    RocketshipSubsystemLayout {
        section: "booster",
        subsystem: "aft_tank",
        weight: 12,
        partition_base: 81,
    },
    RocketshipSubsystemLayout {
        section: "booster",
        subsystem: "thrust_frame",
        weight: 8,
        partition_base: 91,
    },
    RocketshipSubsystemLayout {
        section: "booster",
        subsystem: "engine_cluster",
        weight: 14,
        partition_base: 101,
    },
    RocketshipSubsystemLayout {
        section: "booster",
        subsystem: "plumbing_and_feed",
        weight: 8,
        partition_base: 111,
    },
    RocketshipSubsystemLayout {
        section: "aero",
        subsystem: "fins_and_actuation",
        weight: 8,
        partition_base: 121,
    },
];

pub(super) fn rocketship_node_count() -> usize {
    std::env::var("WORTH_RELATIONAL_ROCKETSHIP_NODE_COUNT")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value >= 1_024)
        .unwrap_or(DEFAULT_ROCKETSHIP_NODE_COUNT)
}

pub(super) fn rocketship_query_target_count(node_count: usize) -> usize {
    std::env::var("WORTH_RELATIONAL_ROCKETSHIP_QUERY_TARGET_COUNT")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_ROCKETSHIP_QUERY_TARGET_COUNT)
        .min(node_count)
}

pub(super) fn seed_rocketship_world(
    runtime: &RelationalRuntime,
    node_count: usize,
) -> RocketshipSeedOutcome {
    let entity_commit_started_at = Instant::now();
    let entity_outcome = {
        let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
        let mut batch = WorkerIntentBatch::new("rocketship-entities-bulk");
        let mut entity_specs = Vec::with_capacity(node_count);
        for index in 0..node_count {
            let partition_id = PartitionId(1 + (index % ROCKETSHIP_PARTITION_WIDTH) as u32);
            entity_specs.push(crate::transactions::data::EntitySpec {
                partition_id,
                kind_id: KindId(1),
                client_key: crate::symbols::data::ClientKey::raw(format!("rocket-node-{index}")),
                fields: crate::tests::support::aspect_field_patch_from_values([
                    (
                        crate::tests::support::aspect_key("name"),
                        crate::tests::support::field_key("name"),
                        crate::tests::support::string_aspect_value(&format!("rocket-node-{index}")),
                    ),
                    (
                        crate::tests::support::aspect_key("zone"),
                        crate::tests::support::field_key("zone"),
                        crate::tests::support::usize_aspect_value(
                            index % ROCKETSHIP_PARTITION_WIDTH,
                        ),
                    ),
                ]),
            });
        }
        for intent in bulk_entity_create_intents(&entity_specs) {
            batch = batch.push(intent);
        }
        txn.push_batch(
            batch,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
        txn.commit(
            runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("rocketship entity seed commit")
    };
    let entity_commit_micros = entity_commit_started_at.elapsed().as_micros();
    let entities = changed_entities(&entity_outcome);
    assert_eq!(
        entities.len(),
        node_count,
        "rocketship entity seed should create the requested entity count"
    );

    let mut relation_specs = Vec::with_capacity(entities.len() + (entities.len() / 64));
    for index in 0..(entities.len() - 1) {
        relation_specs.push(crate::transactions::data::RelationSpec {
            partition_id: PartitionId(101 + (index % ROCKETSHIP_PARTITION_WIDTH) as u32),
            kind_id: KindId(2),
            client_key: crate::symbols::data::ClientKey::raw(format!("rocket-edge-{index}")),
            source: crate::transactions::data::EntityReference::Existing(entities[index]),
            target: crate::transactions::data::EntityReference::Existing(entities[index + 1]),
            fields: crate::transactions::data::AspectFieldPatch::default(),
        });
        if index + ROCKETSHIP_PARTITION_WIDTH < entities.len() && index % 64 == 0 {
            relation_specs.push(crate::transactions::data::RelationSpec {
                partition_id: PartitionId(201 + ((index / 64) % ROCKETSHIP_PARTITION_WIDTH) as u32),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw(format!("rocket-rib-{index}")),
                source: crate::transactions::data::EntityReference::Existing(entities[index]),
                target: crate::transactions::data::EntityReference::Existing(
                    entities[index + ROCKETSHIP_PARTITION_WIDTH],
                ),
                fields: crate::transactions::data::AspectFieldPatch::default(),
            });
        }
    }
    let relation_count = relation_specs.len();
    let mut relation_commit_micros = 0u128;
    let mut relation_commit_phase_timing = crate::transactions::data::CommitPhaseTiming::default();
    for (chunk_index, relation_chunk) in relation_specs
        .chunks(ROCKETSHIP_RELATION_SEED_BATCH_SIZE)
        .enumerate()
    {
        let relation_commit_started_at = Instant::now();
        let outcome = {
            let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
            let mut batch =
                WorkerIntentBatch::new(format!("rocketship-relations-bulk-{chunk_index}"));
            for intent in bulk_relation_create_intents(relation_chunk) {
                batch = batch.push(intent);
            }
            txn.push_batch(
                batch,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .expect("test staging stays within configured resource budgets");
            txn.commit(
                runtime,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .expect("rocketship relation seed commit chunk")
        };
        relation_commit_micros += relation_commit_started_at.elapsed().as_micros();
        relation_commit_phase_timing.draft_preparation_micros +=
            outcome.execution().phase_timing.draft_preparation_micros;
        relation_commit_phase_timing.draft_bulk_admission_micros +=
            outcome.execution().phase_timing.draft_bulk_admission_micros;
        relation_commit_phase_timing.draft_merge_plan_micros +=
            outcome.execution().phase_timing.draft_merge_plan_micros;
        relation_commit_phase_timing.draft_structural_summary_micros += outcome
            .execution()
            .phase_timing
            .draft_structural_summary_micros;
        relation_commit_phase_timing.draft_working_state_clone_micros += outcome
            .execution()
            .phase_timing
            .draft_working_state_clone_micros;
        relation_commit_phase_timing.working_state_preparation_micros += outcome
            .execution()
            .phase_timing
            .working_state_preparation_micros;
        relation_commit_phase_timing.invariant_pre_check_micros +=
            outcome.execution().phase_timing.invariant_pre_check_micros;
        relation_commit_phase_timing.authoritative_mutation_micros += outcome
            .execution()
            .phase_timing
            .authoritative_mutation_micros;
        relation_commit_phase_timing.history_resolution_micros +=
            outcome.execution().phase_timing.history_resolution_micros;
        relation_commit_phase_timing.invariant_post_check_micros +=
            outcome.execution().phase_timing.invariant_post_check_micros;
        relation_commit_phase_timing.artifact_assembly_micros +=
            outcome.execution().phase_timing.artifact_assembly_micros;
        relation_commit_phase_timing.durable_append_micros +=
            outcome.execution().phase_timing.durable_append_micros;
        relation_commit_phase_timing.publication_micros +=
            outcome.execution().phase_timing.publication_micros;
        relation_commit_phase_timing.publication_storage_commit_micros += outcome
            .execution()
            .phase_timing
            .publication_storage_commit_micros;
        assert_eq!(
            changed_relations(&outcome).len(),
            relation_chunk.len(),
            "rocketship relation seed chunk should create the expected relation count"
        );
    }

    RocketshipSeedOutcome {
        entities,
        relation_count,
        entity_commit_micros,
        relation_commit_micros,
        relation_commit_phase_timing,
    }
}
