use crate::PhysicalRecordFormatDeclaration;

use super::{
    ReleaseCustodyHeadMutationV1, ReleaseCustodyHeadNodeWriteV1, ReleaseCustodyHeadPathNodeV1,
    ReleaseCustodyHeadTransitionLimitsV1, ReleaseCustodyHeadTransitionV1,
};
use crate::manifest::release_custody_head::block::max_entries;
use crate::manifest::release_custody_head::entry::ENTRY_BYTES;
use crate::manifest::release_custody_head::{
    ReleaseCustodyHeadBlockReferenceV1, ReleaseCustodyHeadBlockV1, ReleaseCustodyHeadDenial,
    ReleaseCustodyHeadEntryV1, ReleaseCustodyHeadKeyV1,
};

struct Writer {
    tree: u64,
    generation: u64,
    next_block: u64,
    format: PhysicalRecordFormatDeclaration,
    limits: ReleaseCustodyHeadTransitionLimitsV1,
    peak_bytes: u64,
    writes: Vec<ReleaseCustodyHeadNodeWriteV1>,
}

impl Writer {
    fn write(
        &mut self,
        node: ReleaseCustodyHeadBlockV1,
    ) -> Result<ReleaseCustodyHeadBlockReferenceV1, ReleaseCustodyHeadDenial> {
        if self.writes.len() >= usize::from(self.limits.max_new_blocks()) {
            return Err(ReleaseCustodyHeadDenial::Budget);
        }
        let max_frame = u64::from(self.format.page_size().bytes());
        if self
            .peak_bytes
            .checked_add(max_frame)
            .is_none_or(|next| next > self.limits.max_total_frame_bytes())
        {
            return Err(ReleaseCustodyHeadDenial::Budget);
        }
        let frame = node.encode(self.format);
        if frame.len() as u64 > max_frame {
            return Err(ReleaseCustodyHeadDenial::Capacity);
        }
        self.peak_bytes = self
            .peak_bytes
            .checked_add(frame.len() as u64)
            .ok_or(ReleaseCustodyHeadDenial::Budget)?;
        let reference = node.reference(self.format);
        self.writes
            .push(ReleaseCustodyHeadNodeWriteV1::new(reference, frame));
        self.next_block = self
            .next_block
            .checked_add(1)
            .ok_or(ReleaseCustodyHeadDenial::Budget)?;
        Ok(reference)
    }

    fn leaf(
        &mut self,
        entries: Vec<ReleaseCustodyHeadEntryV1>,
    ) -> Result<Vec<ReleaseCustodyHeadBlockReferenceV1>, ReleaseCustodyHeadDenial> {
        if entries.is_empty() {
            return Ok(Vec::new());
        }
        let capacity = max_entries(self.format, ENTRY_BYTES);
        if entries.len() <= capacity {
            let block = ReleaseCustodyHeadBlockV1::leaf(
                self.tree,
                self.generation,
                self.next_block,
                entries,
                self.format,
            )?;
            return Ok(vec![self.write(block)?]);
        }
        if entries.len() != capacity + 1 {
            return Err(ReleaseCustodyHeadDenial::Capacity);
        }
        let midpoint = entries.len().div_ceil(2);
        let left = ReleaseCustodyHeadBlockV1::leaf(
            self.tree,
            self.generation,
            self.next_block,
            entries[..midpoint].to_vec(),
            self.format,
        )?;
        let first = self.write(left)?;
        let right = ReleaseCustodyHeadBlockV1::leaf(
            self.tree,
            self.generation,
            self.next_block,
            entries[midpoint..].to_vec(),
            self.format,
        )?;
        Ok(vec![first, self.write(right)?])
    }

    fn branch(
        &mut self,
        level: u16,
        children: Vec<ReleaseCustodyHeadBlockReferenceV1>,
    ) -> Result<Vec<ReleaseCustodyHeadBlockReferenceV1>, ReleaseCustodyHeadDenial> {
        if children.is_empty() {
            return Ok(Vec::new());
        }
        let capacity = max_entries(
            self.format,
            ReleaseCustodyHeadBlockReferenceV1::ENCODED_BYTES,
        );
        if children.len() <= capacity {
            let block = ReleaseCustodyHeadBlockV1::branch(
                self.tree,
                self.generation,
                self.next_block,
                level,
                children,
                self.format,
            )?;
            return Ok(vec![self.write(block)?]);
        }
        if children.len() != capacity + 1 {
            return Err(ReleaseCustodyHeadDenial::Capacity);
        }
        let midpoint = children.len().div_ceil(2);
        let left = ReleaseCustodyHeadBlockV1::branch(
            self.tree,
            self.generation,
            self.next_block,
            level,
            children[..midpoint].to_vec(),
            self.format,
        )?;
        let first = self.write(left)?;
        let right = ReleaseCustodyHeadBlockV1::branch(
            self.tree,
            self.generation,
            self.next_block,
            level,
            children[midpoint..].to_vec(),
            self.format,
        )?;
        Ok(vec![first, self.write(right)?])
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn plan(
    source_root: Option<ReleaseCustodyHeadBlockReferenceV1>,
    source_next_block: u64,
    source_path: &[ReleaseCustodyHeadPathNodeV1],
    mutation: ReleaseCustodyHeadMutationV1,
    result_generation: u64,
    tree_identity: u64,
    format: PhysicalRecordFormatDeclaration,
    limits: ReleaseCustodyHeadTransitionLimitsV1,
) -> Result<ReleaseCustodyHeadTransitionV1, ReleaseCustodyHeadDenial> {
    if source_next_block == 0
        || result_generation == 0
        || tree_identity == 0
        || source_path.len() > usize::from(limits.max_path_nodes())
        || source_root.is_none() != source_path.is_empty()
        || source_root.is_some_and(|root| {
            root.generation() >= result_generation || root.block() >= source_next_block
        })
    {
        return Err(ReleaseCustodyHeadDenial::Path);
    }
    validate_mutation(mutation)?;
    let mut resident = 0_u64;
    let mut decoded = Vec::with_capacity(source_path.len());
    let mut selected = source_root;
    for (index, item) in source_path.iter().enumerate() {
        let expected = selected.ok_or(ReleaseCustodyHeadDenial::Path)?;
        if item.reference() != expected
            || expected.block() >= source_next_block
            || item.frame().len() > format.page_size().bytes() as usize
        {
            return Err(ReleaseCustodyHeadDenial::Path);
        }
        resident = resident
            .checked_add(item.frame().len() as u64)
            .ok_or(ReleaseCustodyHeadDenial::Budget)?;
        if resident > limits.max_total_frame_bytes() {
            return Err(ReleaseCustodyHeadDenial::Budget);
        }
        let (node, node_format) =
            ReleaseCustodyHeadBlockV1::decode(item.frame(), expected, tree_identity)?;
        if node_format != format {
            return Err(ReleaseCustodyHeadDenial::Path);
        }
        selected = if let Some(children) = node.children() {
            Some(children[child_index(children, mutation.key())])
        } else {
            None
        };
        if index + 1 < source_path.len() && selected != Some(source_path[index + 1].reference()) {
            return Err(ReleaseCustodyHeadDenial::Path);
        }
        decoded.push(node);
    }
    if !source_path.is_empty() && selected.is_some() {
        return Err(ReleaseCustodyHeadDenial::Path);
    }
    let mut writes = Vec::new();
    writes
        .try_reserve_exact(usize::from(limits.max_new_blocks()))
        .map_err(|_| ReleaseCustodyHeadDenial::Budget)?;
    let mut writer = Writer {
        tree: tree_identity,
        generation: result_generation,
        next_block: source_next_block,
        format,
        limits,
        peak_bytes: resident,
        writes,
    };
    let mut replacement = if let Some(leaf) = decoded.last() {
        let entries = leaf.entries().ok_or(ReleaseCustodyHeadDenial::Path)?;
        writer.leaf(changed_leaf(entries, mutation)?)?
    } else {
        writer.leaf(changed_leaf(&[], mutation)?)?
    };
    for node in decoded.iter().rev().skip(1) {
        let children = node.children().ok_or(ReleaseCustodyHeadDenial::Path)?;
        let chosen = child_index(children, mutation.key());
        let mut updated = Vec::with_capacity(children.len() + 1);
        updated.extend_from_slice(&children[..chosen]);
        updated.extend_from_slice(&replacement);
        updated.extend_from_slice(&children[chosen + 1..]);
        replacement = writer.branch(node.level(), updated)?;
    }
    let result_root = match replacement.len() {
        0 => None,
        1 => Some(replacement[0]),
        2 => {
            let level = replacement[0]
                .level()
                .checked_add(1)
                .ok_or(ReleaseCustodyHeadDenial::Capacity)?;
            Some(
                writer
                    .branch(level, replacement)?
                    .into_iter()
                    .next()
                    .ok_or(ReleaseCustodyHeadDenial::Path)?,
            )
        }
        _ => return Err(ReleaseCustodyHeadDenial::Path),
    };
    Ok(ReleaseCustodyHeadTransitionV1 {
        source_root,
        source_next_block,
        mutation,
        result_root,
        result_next_block: writer.next_block,
        writes: writer.writes,
        peak_frame_bytes: writer.peak_bytes,
    })
}

fn child_index(
    children: &[ReleaseCustodyHeadBlockReferenceV1],
    key: ReleaseCustodyHeadKeyV1,
) -> usize {
    children
        .partition_point(|child| child.first() <= key)
        .saturating_sub(1)
}

fn validate_mutation(
    mutation: ReleaseCustodyHeadMutationV1,
) -> Result<(), ReleaseCustodyHeadDenial> {
    match mutation {
        ReleaseCustodyHeadMutationV1::Upsert {
            expected_prior: None,
            next,
        } if next.predecessor().is_none() => Ok(()),
        ReleaseCustodyHeadMutationV1::Upsert {
            expected_prior: Some(prior),
            next,
        } if prior.key() == next.key()
            && !prior.terminal()
            && prior.descriptor_record() != next.descriptor_record()
            && prior.source_basis_digest() == next.source_basis_digest()
            && next.cumulative_dropped() > prior.cumulative_dropped()
            && next.source_root_generation() > prior.source_root_generation()
            && next.predecessor().is_some_and(|predecessor| {
                predecessor.descriptor_record() == prior.descriptor_record()
                    && predecessor.descriptor_frame_sha256() == prior.descriptor_frame_sha256()
            }) =>
        {
            Ok(())
        }
        ReleaseCustodyHeadMutationV1::RetireTerminal { expected_prior }
            if expected_prior.terminal() =>
        {
            Ok(())
        }
        _ => Err(ReleaseCustodyHeadDenial::Mutation),
    }
}

fn changed_leaf(
    entries: &[ReleaseCustodyHeadEntryV1],
    mutation: ReleaseCustodyHeadMutationV1,
) -> Result<Vec<ReleaseCustodyHeadEntryV1>, ReleaseCustodyHeadDenial> {
    let key = mutation.key();
    let index = entries.binary_search_by_key(&key, |entry| entry.key());
    let actual = index.ok().map(|index| entries[index]);
    if actual != mutation.expected_prior() {
        return Err(ReleaseCustodyHeadDenial::Mutation);
    }
    let mut updated = Vec::new();
    updated
        .try_reserve_exact(
            entries
                .len()
                .checked_add(1)
                .ok_or(ReleaseCustodyHeadDenial::Budget)?,
        )
        .map_err(|_| ReleaseCustodyHeadDenial::Budget)?;
    updated.extend_from_slice(entries);
    match mutation {
        ReleaseCustodyHeadMutationV1::Upsert { next, .. } if Some(next) != actual => match index {
            Ok(index) => updated[index] = next,
            Err(index) => updated.insert(index, next),
        },
        ReleaseCustodyHeadMutationV1::RetireTerminal { .. } => {
            updated.remove(index.unwrap());
        }
        _ => return Err(ReleaseCustodyHeadDenial::Mutation),
    }
    Ok(updated)
}
