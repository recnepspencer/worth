use worth_store_physical_format::{
    DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration,
    ReleaseCustodyHeadBlockReferenceV1, ReleaseCustodyHeadBlockViewV1,
    ReleaseCustodyHeadRosterDigestV1,
};

use super::seen::{self, SeenHeadBlocks};
use super::walk_budget::ReleaseCustodyHeadWalkAllowance;
use super::{
    ReleaseCustodyHeadWalkDenial as Denial, ReleaseCustodyHeadWalkLimitsV1,
    ReleaseCustodyHeadWalkPort, ReleaseCustodyHeadWalkV1,
};

pub(super) fn walk<P: ReleaseCustodyHeadWalkPort>(
    root: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    limits: ReleaseCustodyHeadWalkLimitsV1,
    port: &mut P,
) -> Result<ReleaseCustodyHeadWalkV1, Denial<P::Error, P::Error>> {
    let mut roster =
        ReleaseCustodyHeadRosterDigestV1::new(root.release_custody_head_root(), limits.max_entries);
    let Some(reference) = root.release_custody_head_root() else {
        let (entry_count, roster_digest) = roster.finish();
        return Ok(ReleaseCustodyHeadWalkV1 {
            entry_count,
            roster_digest,
            node_count: 0,
            frame_bytes_read: 0,
            peak_resident_bytes: 0,
        });
    };
    require_resident(
        ReleaseCustodyHeadWalkLimitsV1::root_resident_preflight_bytes(format, limits.max_nodes)
            .ok_or(Denial::BoundExceeded)?,
        limits.max_resident_bytes,
    )?;
    let slots = port
        .reserve_vec::<Option<(u64, u64)>>(
            seen::slot_count(limits.max_nodes).ok_or(Denial::BoundExceeded)?,
        )
        .map_err(Denial::Storage)?;
    let mut seen =
        SeenHeadBlocks::from_reserved(slots, limits.max_nodes).ok_or(Denial::BoundExceeded)?;
    let page = u64::from(format.page_size().bytes());
    require_resident(
        scratch_bytes(
            seen.capacity_bytes(),
            slot_bytes::<(ReleaseCustodyHeadBlockReferenceV1, u16)>(1),
            Some(page),
        )?,
        limits.max_resident_bytes,
    )?;
    let mut stack = port
        .reserve_vec::<(ReleaseCustodyHeadBlockReferenceV1, u16)>(1)
        .map_err(Denial::Storage)?;
    stack.push((reference, 1));
    // The page allowance above is a before-read reservation check, not yet
    // resident heap. Report only actual vector capacities in the peak.
    let mut peak = scratch_bytes(seen.capacity_bytes(), vector_bytes(&stack), Some(0))?;
    let mut nodes = 0_u64;
    let mut frame_bytes = 0_u64;
    while let Some((reference, depth)) = stack.pop() {
        if depth > limits.max_depth
            || reference.generation() > root.generation()
            || reference.block() >= root.next_release_custody_head_block()
        {
            return Err(Denial::BoundExceeded);
        }
        match seen.insert((reference.generation(), reference.block())) {
            Some(true) => {}
            Some(false) => return Err(Denial::DuplicateNode),
            None => return Err(Denial::BoundExceeded),
        }
        let remaining = limits
            .max_total_frame_bytes
            .checked_sub(frame_bytes)
            .ok_or(Denial::BoundExceeded)?
            .min(page);
        require_resident(
            scratch_bytes(seen.capacity_bytes(), vector_bytes(&stack), Some(remaining))?,
            limits.max_resident_bytes,
        )?;
        let frame = port.read_node(reference, remaining).map_err(Denial::Read)?;
        if frame.len() as u64 > remaining {
            return Err(Denial::BoundExceeded);
        }
        let current = scratch_bytes(
            seen.capacity_bytes(),
            vector_bytes(&stack),
            vector_bytes(&frame),
        )?;
        require_resident(current, limits.max_resident_bytes)?;
        peak = peak.max(current);
        frame_bytes = frame_bytes
            .checked_add(frame.len() as u64)
            .ok_or(Denial::BoundExceeded)?;
        let (view, decoded_format) =
            ReleaseCustodyHeadBlockViewV1::decode(&frame, reference, root.tree_identity())
                .map_err(Denial::Format)?;
        if decoded_format != format {
            return Err(Denial::Root);
        }
        port.visit_node(reference, &frame).map_err(Denial::Visit)?;
        nodes += 1;
        if let Some(entries) = view.entries() {
            for entry in entries {
                roster.push(entry).map_err(Denial::Format)?;
                port.visit_entry(entry).map_err(Denial::Visit)?;
            }
        } else if let Some(children) = view.children() {
            let child_count = view.count();
            // Every block the walk reads is admitted here, with its siblings,
            // before it is stacked: the root is one block of a bound of one.
            // A count past every count is no limit.
            let observed = nodes
                .checked_add(stack.len() as u64)
                .and_then(|held| held.checked_add(child_count as u64))
                .ok_or(Denial::BoundExceeded)?;
            ReleaseCustodyHeadWalkAllowance::nodes(limits.max_nodes)
                .admit(observed)
                .map_err(Denial::Limit)?;
            let required = stack
                .len()
                .checked_add(child_count)
                .ok_or(Denial::BoundExceeded)?;
            if required > stack.capacity() {
                let prospective = scratch_bytes(
                    seen.capacity_bytes(),
                    vector_bytes(&stack).and_then(|old| {
                        old.checked_add(slot_bytes::<(ReleaseCustodyHeadBlockReferenceV1, u16)>(
                            required,
                        )?)
                    }),
                    vector_bytes(&frame),
                )?;
                require_resident(prospective, limits.max_resident_bytes)?;
                let previous = vector_bytes(&stack).ok_or(Denial::BoundExceeded)?;
                port.grow_vec(&mut stack, child_count)
                    .map_err(Denial::Storage)?;
                let overlap = scratch_bytes(
                    seen.capacity_bytes(),
                    vector_bytes(&stack).and_then(|new| new.checked_add(previous)),
                    vector_bytes(&frame),
                )?;
                require_resident(overlap, limits.max_resident_bytes)?;
                peak = peak.max(overlap);
            }
            let next_depth = depth.checked_add(1).ok_or(Denial::BoundExceeded)?;
            for child in children.rev() {
                stack.push((child, next_depth));
            }
        }
        // The view borrows the frame only within this iteration. Drop the
        // charged frame backing before the next addressed read.
        port.discard_vec(frame);
    }
    let (entry_count, roster_digest) = roster.finish();
    port.discard_vec(stack);
    port.discard_vec(seen.into_slots());
    Ok(ReleaseCustodyHeadWalkV1 {
        entry_count,
        roster_digest,
        node_count: nodes,
        frame_bytes_read: frame_bytes,
        peak_resident_bytes: peak,
    })
}

fn require_resident<ReadError, VisitError>(
    required: u64,
    admitted: u64,
) -> Result<(), Denial<ReadError, VisitError>> {
    ReleaseCustodyHeadWalkAllowance::resident_bytes(admitted)
        .admit(required)
        .map_err(Denial::Limit)
}

fn slot_bytes<T>(count: usize) -> Option<u64> {
    u64::try_from(count)
        .ok()?
        .checked_mul(std::mem::size_of::<T>() as u64)
}

fn vector_bytes<T>(values: &Vec<T>) -> Option<u64> {
    slot_bytes::<T>(values.capacity())
}

fn scratch_bytes<E>(
    seen: Option<u64>,
    stacked: Option<u64>,
    frame: Option<u64>,
) -> Result<u64, Denial<E, E>> {
    seen.and_then(|bytes| bytes.checked_add(stacked?))
        .and_then(|bytes| bytes.checked_add(frame?))
        .ok_or(Denial::BoundExceeded)
}
