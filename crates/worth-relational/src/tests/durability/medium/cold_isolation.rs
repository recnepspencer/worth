//! Relation-bearing cold roots must preserve the demanded region's isolation.
use super::*;
use crate::identity::data::{EntityId, PartitionId};

struct ColdRoot {
    runtime: crate::runtime::RelationalRuntime,
    source: Arc<MemberSource>,
    entities: Vec<EntityId>,
    manifest: crate::durability::data::RelationalCheckpointManifest,
    remote_symbol: Option<crate::symbols::data::Symbol>,
}

fn recover_shape(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    missing_history: Option<PartitionId>,
    count: u32,
    ring: bool,
) -> ColdRoot {
    recover_shape_inner(execution, missing_history, count, ring, false)
}
fn recover_shape_inner(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    missing_history: Option<PartitionId>,
    count: u32,
    ring: bool,
    paged: bool,
) -> ColdRoot {
    let runtime = if paged {
        durable_runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations)
    } else {
        durable_runtime_with_test_schema()
    };
    let mut remote_symbol = None;
    let mut entities = Vec::new();
    let mut targets = Vec::new();
    for id in 1..=count {
        let partition = PartitionId(id);
        let a = create_entity_in_partition(execution, &runtime, "source", partition);
        let b = create_entity_in_partition(execution, &runtime, "target", partition);
        entities.push(a);
        targets.push(b);
    }
    for (index, &a) in entities.iter().enumerate() {
        let b = if ring {
            entities[(index + 1) % entities.len()]
        } else {
            targets[index]
        };
        if paged && index == 1 {
            ring_dictionary::populate(execution, &runtime);
        }
        let label = if paged && index + 1 == entities.len() {
            "remote-paged-edge"
        } else {
            "edge"
        };
        if paged && index + 1 == entities.len() {
            remote_symbol = Some(ring_dictionary::publish_edge(
                execution, &runtime, a, b, label,
            ));
        } else {
            create_relation_in_partition(execution, &runtime, a, b, label, a.partition_id);
        }
    }
    if paged {
        // A branch-head envelope is intentionally resident. Keep the remote
        // field out of that envelope's readmitted meaning closure.
        create_entity_in_partition(execution, &runtime, "source", PartitionId::main());
    }
    let native = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            execution,
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    #[derive(serde::Deserialize)]
    struct Input {
        manifest: crate::durability::data::RelationalCheckpointManifest,
    }
    let input: Input = rmp_serde::from_slice(
        native
            .bytes()
            .strip_prefix(crate::durability::log::native_checkpoint_codec::CHECKPOINT_MAGIC)
            .unwrap(),
    )
    .unwrap();
    let missing = missing_history.map(|id| {
        input.manifest.roots[0]
            .regions
            .iter()
            .find(|leaf| leaf.partition_id == id)
            .unwrap()
            .history
    });
    let artifacts = attached_test_medium(&runtime).offered();
    let source = Arc::new(MemberSource {
        members: native
            .members()
            .iter()
            .filter_map(|member| match member {
                RelationalCheckpointMember::New { locator, bytes } if Some(*locator) != missing => {
                    Some((*locator, bytes.clone()))
                }
                RelationalCheckpointMember::New { .. } => None,
                _ => panic!("first capture owns all members"),
            })
            .collect(),
        commits: artifacts
            .iter()
            .map(|entry| (entry.position(), entry.as_bytes().to_vec()))
            .collect(),
        canonical_members: artifacts
            .iter()
            .map(|artifact| {
                (
                    crate::durability::log::native_checkpoint_codec::member_frame::locator(
                        artifact.as_bytes(),
                    ),
                    artifact.position(),
                )
            })
            .collect(),
        canonical_member_reads: Default::default(),
        journal_reads: Default::default(),
        damaged: AtomicBool::new(false),
        reads: AtomicUsize::new(0),
        member_reads: std::sync::Mutex::new(Vec::new()),
        commit_reads: std::sync::Mutex::new(Vec::new()),
        commits_damaged: AtomicBool::new(false),
        failed_member: std::sync::Mutex::new(None),
        failures: std::sync::Mutex::new(Default::default()),
    });
    let recovery = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            native.covered_position().unwrap(),
            native.covered_branch_transitions().unwrap(),
            native,
        )),
        Vec::new(),
        Vec::new(),
        source.clone(),
    );
    let mut receiving = if paged {
        durable_runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations)
    } else {
        durable_runtime_with_test_schema()
    };
    receiving
        .unshared_state_mut()
        .expect("fresh receiver is unshared")
        .durability
        .attach_recovered_medium(runtime.durability.detach_medium().unwrap());
    let plan = receiving
        .durability()
        .recovery_plan_from_source(
            execution,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    receiving
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert_eq!(source.reads.load(Ordering::Relaxed), 0);
    ColdRoot {
        runtime: receiving,
        source,
        entities,
        manifest: input.manifest,
        remote_symbol,
    }
}

#[path = "cold_projection_adjacency_matrix.rs"]
mod projection_adjacency_matrix;

#[path = "cold_ring_dictionary.rs"]
mod ring_dictionary;
