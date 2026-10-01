use sha2::Digest;
use worth_proof::NonEmpty;
use worth_store_physical_backend::{ArtifactAppendRange, ArtifactTreeDirectory, ArtifactTreeFile};
use worth_store_physical_format::{
    DerivedFamilyRootDirectoryBinding, IndexedThroughBlobPublication,
    PersistedPhysicalRecoveryOperation,
};
use worth_store_wal::{
    plan_wal_frame_append, LogSequenceNumber, WalAppendFrontier, WalLsnRange,
    WalSegmentArtifactIdentity,
};

use crate::physical_runtime::durability::{
    AdmittedPhysicalMutation, AllocatedPhysicalMutationAttemptBinding,
};
use crate::physical_runtime::{
    AdmittedPhysicalDurabilityGroupMember, CanonicalRedoRecords,
    PhysicalDurabilityGroupMemberBinding, PhysicalWalAppendDeclaration,
    PhysicalWalFrameWriteDisposition, PhysicalWalMemberBasis, PlannedPhysicalMutationParts,
    PreparedPhysicalMutation, PreparedPhysicalRootProjection, RecordAppendBatch, WalBarrierMember,
    WalRangeReservedPhysicalMutation,
};

use super::{nonempty, restore_admitted_vec, ReservedPhysicalWalGroupMembers};
use crate::physical_runtime::durability::wal::PhysicalWalReservationDenial;
mod blob_semantic;
mod recovery_projection;
use recovery_projection::recovery_projection;

pub(super) fn plan_group(
    admitted: Vec<(
        crate::physical_runtime::durability::wal::preparation_admission::AdmittedWalPreparedMutation,
        PhysicalDurabilityGroupMemberBinding,
    )>,
    mut frontier: WalAppendFrontier,
    mut next_lsn: LogSequenceNumber,
    artifact: ArtifactTreeFile,
    first_disposition: PhysicalWalFrameWriteDisposition,
) -> Result<
    ReservedPhysicalWalGroupMembers,
    (
        NonEmpty<AdmittedPhysicalDurabilityGroupMember>,
        PhysicalWalReservationDenial,
    ),
> {
    let mut pending = admitted.into_iter();
    let mut planned = Vec::new();
    let mut first = true;
    while let Some((prepared, group)) = pending.next() {
        let disposition = if first {
            first_disposition
        } else {
            PhysicalWalFrameWriteDisposition::AppendExistingSegment
        };
        match plan_member(
            prepared,
            group,
            frontier,
            next_lsn,
            artifact.clone(),
            disposition,
        ) {
            Ok(member) => {
                frontier = member.mutation().resulting_frontier();
                next_lsn = frontier
                    .last_lsn_end()
                    .expect("a planned nonempty WAL frame advances LSN");
                planned.push(member);
                first = false;
            }
            Err((member, cause)) => {
                let mut restored = planned
                    .into_iter()
                    .map(release_reserved_member)
                    .collect::<Vec<_>>();
                restored.push(member);
                restored.extend(restore_admitted_vec(pending.collect()));
                return Err((nonempty(restored), cause));
            }
        }
    }
    Ok(ReservedPhysicalWalGroupMembers(nonempty(planned)))
}

pub(super) fn release_reserved_members(
    members: NonEmpty<WalBarrierMember<WalRangeReservedPhysicalMutation>>,
) -> NonEmpty<AdmittedPhysicalDurabilityGroupMember> {
    nonempty(
        members
            .into_vec()
            .into_iter()
            .map(release_reserved_member)
            .collect(),
    )
}

pub(super) fn wal_artifact(identity: WalSegmentArtifactIdentity) -> ArtifactTreeFile {
    ArtifactTreeDirectory::families()
        .child("wal")
        .expect("the Store-owned WAL directory is portable")
        .file(&identity.file_name())
        .expect("canonical WAL artifact names are portable")
}

fn plan_member(
    prepared: crate::physical_runtime::durability::wal::preparation_admission::AdmittedWalPreparedMutation,
    group: PhysicalDurabilityGroupMemberBinding,
    frontier: WalAppendFrontier,
    start: LogSequenceNumber,
    artifact: ArtifactTreeFile,
    disposition: PhysicalWalFrameWriteDisposition,
) -> Result<
    WalBarrierMember<WalRangeReservedPhysicalMutation>,
    (
        AdmittedPhysicalDurabilityGroupMember,
        PhysicalWalReservationDenial,
    ),
> {
    let prepared = prepared.into_prepared();
    if !matches!(
        prepared.disposition(),
        crate::physical_runtime::PhysicalMutationAdmissionDisposition::Fresh
    ) {
        return Err((
            AdmittedPhysicalDurabilityGroupMember::from_parts(prepared, group),
            PhysicalWalReservationDenial::DuplicateUnresolved,
        ));
    }
    let Some(end) = start
        .get()
        .checked_add(u64::from(prepared.resources().record_count()))
        .map(LogSequenceNumber::new)
    else {
        return Err((
            AdmittedPhysicalDurabilityGroupMember::from_parts(prepared, group),
            PhysicalWalReservationDenial::LsnExhausted,
        ));
    };
    let lsn_range = WalLsnRange::new(start, end)
        .expect("canonical redo is nonempty and therefore has a nonempty LSN range");
    let PlannedPhysicalMutationParts {
        admission,
        batch,
        data,
        mut root,
        context,
    } = prepared.into_parts();
    let data = match data.bind(lsn_range) {
        Ok(data) => data,
        Err((data, denial)) => {
            let prepared =
                PreparedPhysicalMutation::from_planned_parts(PlannedPhysicalMutationParts {
                    admission,
                    batch,
                    data,
                    root,
                    context,
                });
            return Err((
                AdmittedPhysicalDurabilityGroupMember::from_parts(prepared, group),
                PhysicalWalReservationDenial::DataPlanBinding(denial),
            ));
        }
    };
    let binding = admission
        .into_fresh_binding()
        .expect("fresh disposition carries one unallocated WAL binding");
    let prepared_bytes = batch.into_prepared_record_bytes();
    if let Some(basis) = context.derived_directory_basis.as_ref() {
        let records = root.recovery_record_identities().collect::<Vec<_>>();
        let [directory_record] = records.as_slice() else {
            unreachable!("classified directory append owns exactly one assigned record")
        };
        root.set_derived_directory(
            DerivedFamilyRootDirectoryBinding::new(*directory_record, basis.indexed_through),
            basis.expected_previous,
            basis.indexed_through_quarantine,
            basis.replaced_nodes(),
        );
    }
    let projection = recovery_projection(
        &data,
        &root,
        context.blob_record_kind,
        context.derived_directory_basis.as_ref(),
        &prepared_bytes,
    );
    if let PersistedPhysicalRecoveryOperation::GenerationPublished(binding) = projection.operation()
    {
        root.set_latest_blob_publication(
            IndexedThroughBlobPublication::new(
                binding.candidate_root_generation(),
                binding.record(),
                binding.record_payload_sha256(),
            )
            .expect("validated WAL blob binding has a nonzero candidate generation"),
        );
    }
    if let PersistedPhysicalRecoveryOperation::DedupeQuarantined(binding) = projection.operation() {
        root.set_latest_blob_quarantine(binding.record());
    }
    let redo = match data.redo_targets() {
        Some(targets) => CanonicalRedoRecords::from_prepared_records(
            prepared_bytes,
            lsn_range,
            targets,
            &projection,
        ),
        None => CanonicalRedoRecords::from_source_copy(lsn_range, &projection),
    };
    let redo = match data.rewrite() {
        Some(rewrite) => redo.with_encoded_payload(
            rewrite
                .with_group(group.group_identity().bytes())
                .with_page_lsn(lsn_range.start().get())
                .encode(),
        ),
        None => redo,
    };
    let member = PhysicalWalMemberBasis::new(
        group.member_identity(),
        binding.mutation_identity(),
        lsn_range,
    );
    let binding = binding.allocate_wal(group, member, &redo);
    let payload = encode_member_payload(&binding, &redo);
    let declared_identity = declared_member_identity(&binding);
    let frame = match plan_wal_frame_append(frontier, lsn_range, &declared_identity, &payload) {
        Ok(frame) => frame,
        Err(denial) => {
            let prepared = release_planning(binding, redo, data, root, context);
            return Err((
                AdmittedPhysicalDurabilityGroupMember::from_parts(prepared, group),
                PhysicalWalReservationDenial::FramePlanning(denial),
            ));
        }
    };
    let artifact_range = ArtifactAppendRange::new(
        frame.frame().valid_prefix_bytes(),
        frame.frame().encoded_frame().len() as u64,
    )
    .expect("WAL framing returns one nonempty nonoverflowing frame");
    let declaration = PhysicalWalAppendDeclaration::new(
        frontier.segment(),
        frontier.generation(),
        lsn_range,
        artifact_range,
        sha2::Sha256::digest(frame.frame().encoded_frame()).into(),
        disposition,
    );
    Ok(WalBarrierMember::new(
        group,
        WalRangeReservedPhysicalMutation::new(
            binding,
            member,
            redo,
            data,
            root,
            frame,
            artifact,
            declaration,
            context.placement,
            context.deadline,
            context.group_queue_admission,
            context.signal_profile,
            context.durability_policy_basis,
            context.resources,
            context.start,
            context.blob_record_kind,
            context.selected_content_class,
            context.inline_only,
            context.derived_directory_basis,
        ),
    ))
}

fn release_planning(
    binding: AllocatedPhysicalMutationAttemptBinding,
    redo: CanonicalRedoRecords,
    data: crate::physical_runtime::durability::WalBoundPhysicalDataPlan,
    root: PreparedPhysicalRootProjection,
    context: crate::physical_runtime::PreparedPhysicalMutationContext,
) -> PreparedPhysicalMutation {
    let binding = binding.release_wal_allocation();
    let batch = RecordAppendBatch::from_prepared_record_bytes(redo.into_prepared_record_bytes());
    PreparedPhysicalMutation::from_planned_parts(PlannedPhysicalMutationParts {
        admission: AdmittedPhysicalMutation::Fresh(binding),
        batch,
        data: data.into_prepared(),
        root,
        context,
    })
}

fn release_reserved_member(
    member: WalBarrierMember<WalRangeReservedPhysicalMutation>,
) -> AdmittedPhysicalDurabilityGroupMember {
    let (binding, reserved) = member.into_parts();
    AdmittedPhysicalDurabilityGroupMember::from_parts(
        reserved.into_prepared_after_no_effect(),
        binding,
    )
}

fn encode_member_payload(
    binding: &AllocatedPhysicalMutationAttemptBinding,
    redo: &CanonicalRedoRecords,
) -> Vec<u8> {
    let persisted_binding = binding.encode_persisted();
    let mut payload = Vec::with_capacity(
        16_usize
            .saturating_add(persisted_binding.len())
            .saturating_add(redo.encoded().len()),
    );
    write_field(&mut payload, &persisted_binding);
    write_field(&mut payload, redo.encoded());
    payload
}

fn declared_member_identity(binding: &AllocatedPhysicalMutationAttemptBinding) -> String {
    let group = hex(binding.group_binding().group_identity().bytes());
    let member = hex(binding.member().member_identity().bytes());
    let fingerprint = hex(binding.fingerprint().bytes());
    format!("group-{group}-member-{member}-{fingerprint}")
}

fn hex(bytes: [u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn write_field(target: &mut Vec<u8>, field: &[u8]) {
    target.extend_from_slice(&(field.len() as u64).to_le_bytes());
    target.extend_from_slice(field);
}
