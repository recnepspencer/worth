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
    members: BTreeMap<RelationalDurableMemberLocator, worth_execution::ExecutionImmutableBytes>,
    original: Arc<dyn RelationalDurableColdSource>,
}

struct ForgedRead {
    original: RelationalDurableReadProtection,
    members: worth_execution::ExecutionArray<(
        RelationalDurableMemberLocator,
        worth_execution::ExecutionImmutableBytes,
    )>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl ForgedSource {
    fn capture_read(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Box<ForgedRead>, RelationalDurableOwnedReadRefusal> {
        let memory = request.reserve_retained_memory(std::mem::size_of::<ForgedRead>() as u64)?;
        let original = RelationalDurableReadProtection::capture(&self.original, request)?;
        let mut members =
            worth_execution::ExecutionRequestArrayBuilder::allocate(*request, self.members.len())
                .map_err(RelationalDurableOwnedReadRefusal::Buffer)?;
        for (locator, bytes) in &self.members {
            members
                .push((*locator, bytes.clone()))
                .map_err(RelationalDurableOwnedReadRefusal::Buffer)?;
        }
        let members = members
            .seal()
            .map_err(RelationalDurableOwnedReadRefusal::Buffer)?;
        Ok(Box::new(ForgedRead {
            original,
            members,
            _memory: memory,
        }))
    }
}

impl RelationalDurableColdSource for ForgedSource {
    fn checkpoint_read_protection(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        (RelationalDurableReadBounds, RelationalDurableReadPayload),
        RelationalDurableOwnedReadRefusal,
    > {
        let read = self.capture_read(request)?;
        Ok((read.original.bounds(), read))
    }
    fn checkpoint_selected_read_protection(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<RelationalDurableReadPayload, RelationalDurableOwnedReadRefusal> {
        self.capture_read(request)
            .map(|read| read as RelationalDurableReadPayload)
    }
    fn read_commit_owned(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        position: PatchStreamPosition,
        limit: u64,
    ) -> Result<worth_execution::ExecutionImmutableBytes, RelationalDurableOwnedReadRefusal> {
        let read = original
            .downcast_ref::<ForgedRead>()
            .ok_or(RelationalDurableOwnedReadRefusal::ForeignProtection)?;
        read.original
            .read_commit_owned_at(&self.original, request, position, limit)
    }
    fn read_transition_owned(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        sequence: crate::branch::RelationalBranchTransitionSequence,
        limit: u64,
    ) -> Result<worth_execution::ExecutionImmutableBytes, RelationalDurableOwnedReadRefusal> {
        let read = original
            .downcast_ref::<ForgedRead>()
            .ok_or(RelationalDurableOwnedReadRefusal::ForeignProtection)?;
        read.original
            .read_transition_owned_at(&self.original, request, sequence, limit)
    }
    fn read_member_owned(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        member: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<worth_execution::ExecutionImmutableBytes, RelationalDurableOwnedReadRefusal> {
        let read = original
            .downcast_ref::<ForgedRead>()
            .ok_or(RelationalDurableOwnedReadRefusal::ForeignProtection)?;
        request
            .check_live()
            .map_err(|cause| RelationalDurableOwnedReadRefusal::Request(cause.into()))?;
        if member.length > limit {
            return Err(RelationalDurableOwnedReadRefusal::Storage(
                Cause::LimitExceeded {
                    length: member.length,
                    limit,
                },
            ));
        }
        read.members
            .iter()
            .find(|(locator, _)| *locator == member)
            .map(|(_, bytes)| bytes.clone())
            .ok_or(RelationalDurableOwnedReadRefusal::Storage(
                Cause::MissingMember,
            ))
    }

    fn retain_members_for_request(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        members: &[RelationalDurableMemberLocator],
        issuer: std::sync::Weak<dyn RelationalDurableColdSource>,
    ) -> Result<RelationalDurableMemberCustody, RelationalDurableOwnedReadRefusal> {
        for member in members {
            request
                .check_live()
                .map_err(|cause| RelationalDurableOwnedReadRefusal::Request(cause.into()))?;
            if !self.members.contains_key(member) {
                return Err(RelationalDurableOwnedReadRefusal::Storage(
                    Cause::MissingMember,
                ));
            }
        }
        RelationalDurableMemberCustody::try_new_from_source_for_request(issuer, request, || {
            let mut held =
                worth_execution::ExecutionRequestArrayBuilder::allocate(*request, members.len())
                    .map_err(RelationalDurableOwnedReadRefusal::Buffer)?;
            for member in members {
                let bytes = self
                    .members
                    .get(member)
                    .expect("the immutable source roster was checked before custody admission");
                held.push(bytes.clone())
                    .map_err(RelationalDurableOwnedReadRefusal::Buffer)?;
            }
            held.seal()
                .map_err(RelationalDurableOwnedReadRefusal::Buffer)
        })
    }
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
    fn read_commit(&self, position: PatchStreamPosition, limit: u64) -> Result<Vec<u8>, Cause> {
        self.original.read_commit(position, limit)
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
            &execution,
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
        members[&leaf.head].bytes(),
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
    members.insert(
        locator,
        worth_execution::ExecutionImmutableBytes::from_external_bytes(Arc::new(
            bytes.into_boxed_slice(),
        )),
    );
    let commitment = super::super::kind_commitment::digest(&head.kind_projection).unwrap();
    let mut draft = wire.manifest.ordinary_owned_draft();
    let forge = |forged: &mut RelationalCheckpointRegion| {
        if forged.head == leaf.head {
            forged.head = locator;
            forged.kind_authority.head = commitment;
        }
    };
    let roots = draft
        .roots
        .iter()
        .map(|root| {
            let mut regions = root.regions.clone();
            regions.iter_mut().for_each(forge);
            RelationalCheckpointRoot::from_ordinary_parts(
                root.commit_id,
                root.recorded_components.clone(),
                root.schema_carrier_digest,
                regions,
                root.content.clone(),
            )
        })
        .collect();
    draft.roots = CheckpointRoots::from_ordinary(roots);
    draft.mirror.iter_mut().for_each(forge);
    wire.manifest = draft.seal_ordinary();
    let mut bytes = magic.to_vec();
    bytes.extend(rmp_serde::to_vec_named(&wire).unwrap());
    let source = Arc::new(ForgedSource {
        members,
        original: attached_test_medium(&runtime)
            .recovery_source()
            .unwrap()
            .into_parts()
            .cold_source,
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
