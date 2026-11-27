//! A named coupled commit over cold residency. Recovered from an
//! external-member checkpoint, a world's regions are cold; the producer reads
//! a record in a partition the authored prefix never touched, so Native
//! demands that region during preparation. The candidate's plan is composed
//! once, from the authored plan every preparation round starts from, and the
//! producer's outputs commit under exactly their planned names.
use super::*;
use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use worth_relational::facade::config::RelationalRuntimeProfile;
use worth_relational::facade::durability::*;
use worth_relational::facade::runtime::ProjectionAspectScope;
use worth_testkit_relational::durable_medium::{
    immutable_members::ImmutableCheckpointMemberSource, TestDurableMedium,
};

const FAR: PartitionId = PartitionId::new(7);

fn durable(medium: &Arc<TestDurableMedium>) -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(named_registry())
        .build_durable(medium.attachment())
        .unwrap()
}

fn member_source(
    checkpoint: &RelationalNativeCheckpoint,
    medium: &TestDurableMedium,
) -> Arc<ImmutableCheckpointMemberSource> {
    let artifacts = medium.offered();
    Arc::new(ImmutableCheckpointMemberSource {
        members: checkpoint
            .members()
            .iter()
            .map(|member| match member {
                RelationalCheckpointMember::New { locator, bytes } => (*locator, bytes.clone()),
                RelationalCheckpointMember::Carried { .. } => panic!("a first capture"),
            })
            .collect(),
        commits: artifacts
            .iter()
            .map(|artifact| (artifact.position(), artifact.as_bytes().to_vec()))
            .collect(),
        canonical_members: artifacts
            .iter()
            .map(|artifact| {
                let locator = RelationalDurableMemberLocator {
                    sha256: Sha256::digest(artifact.as_bytes()).into(),
                    length: artifact.as_bytes().len() as u64,
                };
                (locator, artifact.position())
            })
            .collect(),
        canonical_member_reads: Default::default(),
        journal_reads: Default::default(),
        damaged: Default::default(),
        reads: Default::default(),
        member_reads: Default::default(),
        paged_reads: Default::default(),
        commit_page_reads: Default::default(),
        naming_page_reads: Default::default(),
        commits_damaged: Default::default(),
        commit_reads: Default::default(),
        failed_member: Default::default(),
        failures: Default::default(),
    })
}

/// The named world recovered cold, its far record, and the member source.
fn recovered(
    execution: &ExecutionRequest<'_, '_>,
) -> (World, EntityId, Arc<ImmutableCheckpointMemberSource>) {
    let far = MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: FAR,
        kind_id: KindId(1),
        client_key: ClientKey::raw("far"),
        fields: string_aspect_field_patch([(aspect_key("name"), field_key("name"), "far")]),
    }));
    let original = TestDurableMedium::new();
    let (world, ids) = named_world_on(execution, durable(&original), seeds().push(far));
    let checkpoint = world
        .runtime
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
    let source = member_source(&checkpoint, &original);
    let input = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            checkpoint.covered_position().unwrap(),
            checkpoint.covered_branch_transitions().unwrap(),
            checkpoint,
        )),
        Vec::new(),
        Vec::new(),
        source.clone(),
    );
    let medium = TestDurableMedium::new();
    for artifact in &original.offered() {
        assert!(matches!(
            medium.attachment().publish(artifact),
            DurablePublicationFate::Committed
        ));
    }
    let mut runtime = durable(&medium);
    let receipt = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(RelationalSchemaRegistry::new())
        .unwrap();
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    let receipt = runtime
        .readmit_recovered_initial_schema_installation(receipt)
        .unwrap();
    let (owner, installed) = receipt.issue_application_publication_owner();
    let world = World {
        runtime,
        owner,
        installed: installed.with_source_scope_owned_producers([("owned".into(), KindId(1))]),
        ..world
    };
    (world, ids[2], source)
}

fn far_is_live(
    execution: &ExecutionRequest<'_, '_>,
    candidate: &mut Candidate<'_>,
    far: EntityId,
) -> Result<bool, Failure> {
    let pin = candidate.pin_source(execution)?;
    Ok(pin
        .project_candidate()
        .entity_record_with_projection_scope(
            execution,
            far,
            ProjectionAspectScope::whole_aspects([aspect_key("name")]),
            |_| Some(()),
        )
        .expect("the far record reads")
        .is_some())
}

#[test]
fn a_cold_named_coupled_commit_composes_its_plan_once() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let (world, far, source) = recovered(execution);
    let reads = source.reads.load(Ordering::Relaxed);
    let seen = RefCell::new(Vec::new());
    let assembly = assemble(|execution, candidate| {
        seen.borrow_mut()
            .push(far_is_live(execution, candidate, far)?);
        candidate.append_named_publication(
            execution,
            birth(&world, execution, &KEYS),
            &none(),
            round(PartitionId::main()),
            Some(names(PartitionId::main(), "cold")),
        )
    });
    let proposal = world.prepare(execution, &assembly).unwrap();
    assert!(
        source.reads.load(Ordering::Relaxed) > reads,
        "the far region was cold and demanded"
    );
    assert_eq!(
        seen.borrow().last(),
        Some(&true),
        "the demanded region is read"
    );
    let candidate = world
        .runtime
        .prepare_validated_proposal(execution, proposal)
        .unwrap_or_else(|err| panic!("{:?} / {err:?}", err.lineage_name_denial_ref()));
    let RelationalPublicationOutcome::Performed(performed) = world
        .runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("the named group performs")
    };
    let result = world
        .runtime
        .settle_performed_publication(performed)
        .unwrap();
    assert_eq!(
        committed_names(&world, execution, &result).len(),
        KEYS.len() + 1
    );
    release_test_commit_snapshot(&world.runtime, &result);
}
