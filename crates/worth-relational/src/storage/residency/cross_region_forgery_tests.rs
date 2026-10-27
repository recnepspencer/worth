//! Rehashing a peer-kind claim cannot replace canonical root authority.
use super::super::region_image::*;
use super::*;
use crate::durability::data::*;
use crate::durability::medium::*;
use crate::identity::data::{KindId, PartitionId};

#[derive(serde::Serialize, serde::Deserialize)]
struct ExternalWire {
    manifest: RelationalCheckpointManifest,
    members: Vec<()>,
    envelopes: Vec<()>,
}

#[derive(Debug)]
struct ForgedSource {
    members: BTreeMap<RelationalDurableMemberLocator, Arc<[u8]>>,
    commits: BTreeMap<PatchStreamPosition, Vec<u8>>,
}

impl RelationalDurableColdSource for ForgedSource {
    fn retain_members(
        &self,
        _members: &[RelationalDurableMemberLocator],
    ) -> Result<crate::durability::medium::RelationalDurableMemberCustody, Cause> {
        Ok(crate::durability::medium::RelationalDurableMemberCustody::new(()))
    }

    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        _: u64,
    ) -> Result<Vec<u8>, Cause> {
        self.members
            .get(&locator)
            .map(|bytes| bytes.to_vec())
            .ok_or(Cause::MissingMember)
    }
    fn read_commit(&self, position: PatchStreamPosition, _: u64) -> Result<Vec<u8>, Cause> {
        self.commits
            .get(&position)
            .cloned()
            .ok_or(Cause::MissingMember)
    }
}

#[test]
fn rehashed_cross_region_kind_forgery_is_refused_by_the_canonical_root() {
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let runtime = durable_runtime_with_test_schema();
    let a = create_entity_in_partition(&execution, &runtime, "a", PartitionId(1));
    let b = create_entity_in_partition(&execution, &runtime, "b", PartitionId(2));
    create_relation_in_partition(&execution, &runtime, a, b, "peer-kind", PartitionId(3));
    let native = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    let magic = crate::durability::log::native_checkpoint_codec::CHECKPOINT_MAGIC;
    let mut wire: ExternalWire =
        rmp_serde::from_slice(native.bytes().strip_prefix(magic).unwrap()).unwrap();
    let leaf = wire.manifest.roots[0]
        .regions
        .iter()
        .find(|leaf| leaf.partition_id == a.partition_id)
        .unwrap()
        .clone();
    let mut members = native
        .members()
        .iter()
        .map(|member| match member {
            RelationalCheckpointMember::New { locator, bytes } => (*locator, bytes.clone()),
            _ => panic!("first capture owns members"),
        })
        .collect::<BTreeMap<_, _>>();
    let schema_digest = wire.manifest.roots[0].schema_carrier_digest;
    let frame = RelationalRegionImage::decode(
        &members[&leaf.head],
        leaf.head,
        a.partition_id,
        schema_digest,
        RelationalRegionImageKind::Head,
    )
    .unwrap();
    let mut head: RegionHeadImage = rmp_serde::from_slice(frame.payload).unwrap();
    assert_eq!(head.kind_projection.len(), 1);
    assert_eq!(
        head.kind_projection[0].relation.partition_id,
        PartitionId(3)
    );
    head.kind_projection[0].kind = KindId(999);
    for entry in &mut head.adjacency_kinds {
        for (kind, _) in &mut entry.kinds {
            *kind = KindId(999);
        }
    }
    let (locator, bytes) = RelationalRegionImage::encode(
        a.partition_id,
        schema_digest,
        RelationalRegionImageKind::Head,
        &head,
    )
    .unwrap();
    members.insert(locator, bytes.into());
    let commitment = super::super::kind_commitment::digest(&head.kind_projection).unwrap();
    for forged in wire
        .manifest
        .roots
        .iter_mut()
        .flat_map(|root| &mut root.regions)
        .chain(&mut wire.manifest.mirror)
    {
        if forged.head == leaf.head {
            forged.head = locator;
            forged.kind_authority.head = commitment;
        }
    }
    let mut bytes = magic.to_vec();
    bytes.extend(rmp_serde::to_vec_named(&wire).unwrap());
    let source = Arc::new(ForgedSource {
        members,
        commits: attached_test_medium(&runtime)
            .offered()
            .iter()
            .map(|commit| (commit.position(), commit.as_bytes().to_vec()))
            .collect(),
    });
    let recovery = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            native.covered_position().unwrap(),
            native.covered_branch_transitions().unwrap(),
            RelationalNativeCheckpoint::from_untrusted_bytes(bytes),
        )),
        Vec::new(),
        Vec::new(),
        source,
    );
    let mut receiving = durable_runtime_with_test_schema();
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
    assert!(matches!(failure, RelationalRecoveryStop::Durability(error)
        if error.class == RecoveryFailureClass::CorruptCheckpoint && error.detail.contains("StorageContentMismatch")));
}
