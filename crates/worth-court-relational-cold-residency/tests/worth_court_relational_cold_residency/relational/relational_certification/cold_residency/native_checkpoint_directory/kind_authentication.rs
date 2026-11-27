//! A capture cannot replace canonical adjacency authority by rehashing leaves.
use super::*;
use std::sync::Arc;
use worth_relational::facade::{
    durability::{
        RecoveryVerificationMode, RelationalCheckpointArtifactPolicy, RelationalCheckpointMember,
        RelationalDurableCheckpointSource, RelationalDurableRecoverySource,
    },
    transactions::{EntityReference, RelationSpec},
};
use worth_testkit_relational::durable_medium::immutable_members::ImmutableCheckpointMemberSource;

#[test]
fn checkpoint_readmission_refuses_retired_integer_array_digest_before_restore() {
    let host = serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let runtime = memory_runtime();
    create_entity(&execution, &runtime, "binary-digest-checkpoint");
    let native = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
&execution,
execution.allocation_policy(),
worth_relational::facade::durability::RelationalCheckpointArtifactPolicy::ExternalMembers { current_members: &[], source: None },
)
        .unwrap();
    restore(&execution, &native);
    let leaf = rows(field(manifest(&native), "mirror"))[0];
    let digest: worth_foundational::CanonicalDigestId =
        rmp_serde::from_slice(field(field(leaf, "kind_authority"), "head")).unwrap();
    let mut current = rmp_serde::to_vec("head").unwrap();
    current.extend(rmp_serde::to_vec(&digest).unwrap());
    let mut retired = rmp_serde::to_vec("head").unwrap();
    retired.extend(rmp_serde::to_vec(digest.bytes()).unwrap());
    assert_ne!(current, retired);
    // Rewrite only the representation of this actual captured digest. Every
    // semantic value and every other field keeps the owning producer's bytes.
    let mut bytes = Vec::new();
    let mut remaining = native.bytes();
    let mut replacements = 0;
    while let Some(at) = remaining
        .windows(current.len())
        .position(|part| part == current)
    {
        bytes.extend_from_slice(&remaining[..at]);
        bytes.extend_from_slice(&retired);
        remaining = &remaining[at + current.len()..];
        replacements += 1;
    }
    bytes.extend_from_slice(remaining);
    assert!(replacements > 0);
    let stop = memory_runtime()
        .durability_recovery()
        .restore_native_checkpoint(
            &execution,
            &RelationalNativeCheckpoint::from_untrusted_bytes(bytes),
            worth_testkit_relational::cold_recovery::members_source(&native),
        )
        .unwrap_err();
    assert_eq!(
        expect_durability_failure(stop).class,
        RecoveryFailureClass::CorruptCheckpoint
    );
}

#[test]
fn forged_cross_region_kind_commitment_is_refused_before_region_payload_reads() {
    for dimension in 0..4 {
        let host = serial_request();
        let execution = worth_execution::ExecutionRequest::serial(&host);
        let runtime = memory_runtime();
        let source = super::super::region_publications::create_entity(
            &execution,
            &runtime,
            "source",
            PartitionId(1),
        );
        let target = super::super::region_publications::create_entity(
            &execution,
            &runtime,
            "target",
            PartitionId(2),
        );
        let mut transaction = begin_main_transaction(&runtime);
        transaction
            .push_batch(
                WorkerIntentBatch::new("actual cross-region relation").push(
                    MutationIntent::Create(CreateIntent::Relation(RelationSpec {
                        partition_id: PartitionId(3),
                        kind_id: KindId(2),
                        client_key: "cross-region".into(),
                        source: EntityReference::Existing(source),
                        target: EntityReference::Existing(target),
                        fields: field_patch("label", "label", "cross-region"),
                    })),
                ),
                execution.allocation_policy(),
            )
            .unwrap();
        let relation = transaction.commit(&execution, &runtime).unwrap();
        release_test_commit_snapshot(&runtime, &relation);
        let native = runtime
            .durability_authority()
            .native_checkpoint_with_policy(
                &execution,
                execution.allocation_policy(),
                RelationalCheckpointArtifactPolicy::ExternalMembers {
                    current_members: &[],
                    source: None,
                },
            )
            .unwrap();
        let payload = native.bytes().strip_prefix(CHECKPOINT_MAGIC).unwrap();
        let manifest = manifest(&native);
        let original_roots = rows(field(manifest, "roots"));
        let roots = original_roots
            .iter()
            .map(|root| {
                let regions = forge_regions(field(root, "regions"), dimension);
                replace_field(root, "regions", &regions)
            })
            .collect::<Vec<_>>();
        let roots = array(&roots.iter().map(Vec::as_slice).collect::<Vec<_>>());
        let mirror = forge_regions(field(manifest, "mirror"), dimension);
        let amended = replace_field(manifest, "roots", &roots);
        let amended = replace_field(&amended, "mirror", &mirror);
        let bytes = [
            CHECKPOINT_MAGIC,
            &replace_field(payload, "manifest", &amended),
        ]
        .concat();
        let canonical_members = rows(field(manifest, "canonical_members"))
            .iter()
            .map(|entry| {
                rmp_serde::from_slice::<RelationalDurableMemberLocator>(field(entry, "member"))
                    .unwrap()
            })
            .collect::<Vec<_>>();
        let commits = runtime
            .durability()
            .canonical_commit_artifacts(&execution)
            .map(|artifact| {
                let artifact = artifact.unwrap();
                (artifact.position(), artifact.as_bytes().to_vec())
            })
            .collect();
        let source = Arc::new(ImmutableCheckpointMemberSource {
            members: native
                .members()
                .iter()
                .map(|member| {
                    let RelationalCheckpointMember::New { locator, bytes } = member else {
                        panic!("first capture owns its members");
                    };
                    (*locator, bytes.clone())
                })
                .collect(),
            commits,
            canonical_members: rows(field(manifest, "canonical_members"))
                .iter()
                .map(|entry| {
                    (
                        rmp_serde::from_slice(field(entry, "member")).unwrap(),
                        rmp_serde::from_slice(field(entry, "position")).unwrap(),
                    )
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
            commit_reads: Default::default(),
            commits_damaged: Default::default(),
            failed_member: Default::default(),
            failures: Default::default(),
        });
        let recovery = RelationalDurableRecoverySource::new(
            Some(RelationalDurableCheckpointSource::new(
                native.covered_position().unwrap(),
                native.covered_branch_transitions().unwrap(),
                RelationalNativeCheckpoint::from_untrusted_bytes(bytes),
            )),
            Vec::new(),
            Vec::new(),
            source.clone(),
        );
        let mut receiving = memory_runtime();
        let plan = receiving
            .durability()
            .recovery_plan_from_source(
                &execution,
                recovery,
                RecoveryVerificationMode::NormalRecoveryVerification,
            )
            .unwrap();
        let failure = receiving
            .durability_recovery()
            .recover(&execution, plan)
            .unwrap_err();
        let reads = source
            .canonical_member_reads
            .lock()
            .unwrap()
            .iter()
            .chain(source.member_reads.lock().unwrap().iter())
            .copied()
            .collect::<Vec<_>>();
        assert!(
            !reads.is_empty(),
            "dimension {dimension}: the untrusted checkpoint must obtain canonical authority"
        );
        assert!(
            reads
                .iter()
                .all(|locator| canonical_members.contains(locator)),
            "dimension {dimension}: only canonical authority acquisition may precede kind refusal: {reads:?}"
        );
        assert!(
            matches!(
                failure,
                RelationalRecoveryStop::Durability(ref error)
                if error.class == RecoveryFailureClass::CorruptCheckpoint
                    && error.detail.contains("StorageContentMismatch")
            ),
            "dimension {dimension}: {failure:?}"
        );
    }
}

fn forge_regions(bytes: &[u8], dimension: usize) -> Vec<u8> {
    let mut leaves = rows(bytes);
    let index = leaves
        .iter()
        .position(|leaf| {
            rmp_serde::from_slice::<PartitionId>(field(leaf, "partition_id")).unwrap()
                == PartitionId(1)
        })
        .unwrap();
    let leaf = leaves[index];
    let (part, key, value) = match dimension {
        0 => {
            let part = "kind_authority";
            let mut digest = *rmp_serde::from_slice::<worth_foundational::CanonicalDigestId>(
                field(field(leaf, part), "head"),
            )
            .unwrap()
            .bytes();
            digest[0] ^= 1;
            (
                part,
                "head",
                rmp_serde::to_vec(&worth_foundational::CanonicalDigestId::new(digest)).unwrap(),
            )
        }
        1 | 2 => {
            let part = "live_counts";
            let key = if dimension == 1 {
                "entities"
            } else {
                "relations"
            };
            let count: usize = rmp_serde::from_slice(field(field(leaf, part), key)).unwrap();
            (part, key, rmp_serde::to_vec(&(count + 1)).unwrap())
        }
        3 => ("directory", "entities", array(&[])),
        _ => unreachable!("four explicit corruptions"),
    };
    let changed = replace_field(field(leaf, part), key, &value);
    let changed_leaf = replace_field(leaf, part, &changed);
    leaves[index] = &changed_leaf;
    array(&leaves)
}
