//! Mixed retention, slot reuse and cold round trips through the public
//! SDK. Every native checkpoint independently recomputes each region's kind
//! authority from metadata and must equal the incrementally carried commitment.
use super::checkpoint::truth;
use super::fixture::World;
use crate::leased_preparation::ordinary_context::support;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::durability::*;
use worth_relational::facade::publication::PatchStreamPosition;
use worth_relational::facade::storage::RelationalStorageUnavailableCause as Cause;
use worth_relational::facade::{
    identity::{KindId, PartitionId},
    runtime::*,
    schema::*,
    symbols::ClientKey,
    transactions::*,
};

fn registry() -> RelationalSchemaRegistry {
    let mut registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "demo.entity".to_string(),
            schema_id: SchemaId("demo".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                support::entity_string_field_aspect(
                    support::aspect_key("name"),
                    support::field_key("name"),
                ),
            ]),
        })
        .unwrap();
    for (kind, name) in [(2, "demo.relation"), (3, "demo.other")] {
        registry = registry
            .register_relation_kind(RelationKindRegistration {
                kind_id: KindId(kind),
                kind_name: name.to_string(),
                schema_id: SchemaId("demo".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy:
                    worth_relational::facade::config::CrossContextPolicy::AllowExplicit,
                cascade_delete_policy:
                    worth_relational::facade::config::CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                    support::relation_string_field_aspect(
                        support::aspect_key("label"),
                        support::field_key("label"),
                    ),
                ]),
                relation_integrity: RelationIntegrityDeclarations::default(),
            })
            .unwrap();
    }
    registry
}

fn build() -> RelationalRuntime {
    use worth_relational::facade::config::{
        MvccConfig, RelationalRuntimeProfile, RetentionBackend, SnapshotReleasePolicy,
    };
    RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .mvcc(MvccConfig {
            track_visibility_metadata: true,
            snapshot_release_policy: SnapshotReleasePolicy::ExplicitRelease,
            auto_reclaim_deleted_records: true,
            reclaim_batch_size: 8,
            retention_backend: RetentionBackend::EpochChunkRetention,
        })
        .schema_registry(registry())
        .build()
        .unwrap()
}

struct Lcg(u64);
impl Lcg {
    fn next(&mut self, bound: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 33) as usize) % bound.max(1)
    }
}

fn relate(world: &mut World, execution: &ExecutionRequest<'_, '_>, rng: &mut Lcg, label: &str) {
    if world.entities.is_empty() {
        return;
    }
    let source = world.entities[rng.next(world.entities.len())];
    let target = world.entities[rng.next(world.entities.len())];
    let intent = MutationIntent::Create(CreateIntent::Relation(RelationSpec {
        partition_id: PartitionId([7, 11, 31][rng.next(3)]),
        kind_id: KindId([2, 3][rng.next(2)]),
        client_key: ClientKey::raw(label),
        source: EntityReference::Existing(source),
        target: EntityReference::Existing(target),
        fields: support::string_field_patch(
            support::aspect_key("label"),
            support::field_key("label"),
            label,
        ),
    }));
    // A duplicate (source, target, kind) is a legitimate typed refusal.
    match world.commit(execution, intent) {
        Ok(commit) => world.release(&commit),
        Err(TransactionCommitError::Conflict { .. }) => {}
        Err(other) => panic!("{other:?}"),
    }
    world.refresh(execution);
}

#[derive(Debug, Default)]
struct CertSource {
    members: Mutex<BTreeMap<RelationalDurableMemberLocator, Vec<u8>>>,
    commits: Mutex<BTreeMap<PatchStreamPosition, Vec<u8>>>,
}
impl RelationalDurableColdSource for CertSource {
    fn retain_members(
        &self,
        _: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, Cause> {
        Ok(RelationalDurableMemberCustody::new(()))
    }
    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        _: u64,
    ) -> Result<Vec<u8>, Cause> {
        self.members
            .lock()
            .unwrap()
            .get(&locator)
            .cloned()
            .ok_or(Cause::MissingMember)
    }
    fn read_commit(&self, position: PatchStreamPosition, _: u64) -> Result<Vec<u8>, Cause> {
        self.commits
            .lock()
            .unwrap()
            .get(&position)
            .cloned()
            .ok_or(Cause::SourceUnavailable)
    }
}

struct Cert {
    source: Arc<CertSource>,
    reopened: bool,
    membership: Vec<RelationalDurableMemberLocator>,
}
impl Cert {
    fn absorb_commits(&self, runtime: &RelationalRuntime, execution: &ExecutionRequest<'_, '_>) {
        let collected: Vec<_> = runtime
            .durability()
            .canonical_commit_artifacts(execution.allocation_policy())
            .map(|artifact| {
                let artifact = artifact.unwrap();
                (artifact.position(), artifact.as_bytes().to_vec())
            })
            .collect();
        self.source.commits.lock().unwrap().extend(collected);
    }
    /// Native capture recomputes every region's kind authority cold and refuses
    /// unless it equals the carried (incremental) root leaf commitment.
    fn capture(
        &mut self,
        runtime: &RelationalRuntime,
        execution: &ExecutionRequest<'_, '_>,
    ) -> RelationalNativeCheckpoint {
        self.absorb_commits(runtime, execution);
        let source: Arc<dyn RelationalDurableColdSource> = self.source.clone();
        let checkpoint = runtime
            .durability_authority()
            .native_checkpoint_with_policy(
                execution,
                execution.allocation_policy(),
                RelationalCheckpointArtifactPolicy::ExternalMembers {
                    current_members: &self.membership,
                    source: self.reopened.then_some(source),
                },
            )
            .unwrap();
        let mut members = self.source.members.lock().unwrap();
        for member in checkpoint.members() {
            if let RelationalCheckpointMember::New { locator, bytes } = member {
                members.insert(*locator, bytes.to_vec());
            }
        }
        checkpoint
    }
}

fn run(seed: u64, steps: usize) {
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(2 * 1024 * 1024 * 1024)),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&request);
    let mut world = World {
        runtime: build(),
        entities: vec![],
        relations: vec![],
    };
    let mut cert = Cert {
        source: Arc::new(CertSource::default()),
        reopened: false,
        membership: vec![],
    };
    let mut rng = Lcg(seed.wrapping_mul(0x9e3779b97f4a7c15) | 1);
    for index in 0..4 {
        world.create(&execution, [7, 11, 29][index % 3], &format!("seed-{index}"));
    }
    let mut ops = [0usize; 8];
    let mut reopens = 0;
    let mut checked = 0;

    for step in 0..steps {
        let label = format!("s{seed}-{step}");
        let op = rng.next(8);
        ops[op] += 1;

        match op {
            0 => world.create(&execution, [7, 11, 29][rng.next(3)], &label),
            1 | 2 => relate(&mut world, &execution, &mut rng, &label),
            3 if !world.relations.is_empty() => {
                let index = rng.next(world.relations.len());
                world.delete_relation(&execution, index)
            }
            4 if world.entities.len() > 2 => {
                let index = rng.next(world.entities.len());
                world.delete_entity(&execution, index)
            }
            5 if !world.entities.is_empty() => {
                let index = rng.next(world.entities.len());
                world.replace(&execution, index, [7, 11, 29, 31][rng.next(4)], &label)
            }
            6 => {
                world
                    .runtime
                    .retention()
                    .run_pass(&execution)
                    .unwrap_or_else(|error| panic!("seed={seed} step={step}: {error:?}"));
                world.refresh(&execution);
            }
            7 if step > 0 => {
                // Trust boundary: checkpoint, then cold restore and continue.
                let checkpoint = cert.capture(&world.runtime, &execution);
                let before = truth(&world.runtime, &execution);
                let reopened = reopen_with(&execution, &checkpoint, cert.source.clone());
                assert_eq!(
                    truth(&reopened, &execution),
                    before,
                    "seed {seed} step {step}"
                );
                cert.membership = checkpoint
                    .members()
                    .iter()
                    .map(RelationalCheckpointMember::locator)
                    .collect();
                cert.reopened = true;
                reopens += 1;
                world.runtime = reopened;
                world.refresh(&execution);
            }
            _ => world.create(&execution, [7, 11, 29][rng.next(3)], &label),
        }
        // Independent cold recomputation versus the carried commitment.
        let _ = cert.capture(&world.runtime, &execution);
        checked += 1;
    }
    eprintln!(
        "RETENTION_ROUNDTRIP seed={seed} steps={steps} checked_steps={checked} ops={ops:?} reopens={reopens} entities={} relations={}",
        world.entities.len(),
        world.relations.len()
    );
}

#[test]
fn mixed_retirement_replacement_reuse_and_cold_roundtrips_keep_region_integrity() {
    for seed in 0..12 {
        run(seed, 80);
    }
}

fn reopen_with(
    execution: &ExecutionRequest<'_, '_>,
    checkpoint: &RelationalNativeCheckpoint,
    source: Arc<CertSource>,
) -> RelationalRuntime {
    let recovery = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            checkpoint.covered_position().unwrap(),
            checkpoint.covered_branch_transitions().unwrap(),
            RelationalNativeCheckpoint::from_untrusted_bytes(checkpoint.bytes().to_vec()),
        )),
        vec![],
        vec![],
        source,
    );
    let mut reopened = build();
    let plan = reopened
        .durability()
        .recovery_plan_from_source(
            execution,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    reopened
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    reopened
}

#[test]
fn a_reused_relation_slot_keeps_retention_readmission_valid() {
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(1 << 30)),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&request);
    let mut world = World {
        runtime: build(),
        entities: vec![],
        relations: vec![],
    };
    let mut cert = Cert {
        source: Arc::default(),
        reopened: false,
        membership: vec![],
    };
    world.create(&execution, 7, "left");
    world.create(&execution, 7, "right");
    world.create(&execution, 7, "different source");
    world.relate(&execution, 0, 1, 7, "old generation");
    let old = world.relations[0];
    world.delete_relation(&execution, 0);
    world.runtime.retention().run_pass(&execution).unwrap();
    let checkpoint = cert.capture(&world.runtime, &execution);
    world.runtime = reopen_with(&execution, &checkpoint, cert.source.clone());
    world.refresh(&execution);
    world.relate(&execution, 2, 1, 7, "new generation");
    let new = world.relations[0];
    assert_eq!(new.slot_index(), old.slot_index());
    assert!(new.generation_value() > old.generation_value());
    world.runtime.retention().run_pass(&execution).unwrap();
    world.refresh(&execution);
    world.create(&execution, 7, "after retention");
}
