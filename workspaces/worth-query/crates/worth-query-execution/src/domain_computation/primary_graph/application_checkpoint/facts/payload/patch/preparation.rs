//! Sparse topology planning and funding, before any original Fact is cloned.
use super::*;

type Nodes = ExecutionArrayBuilder<RefCell<Option<PreparedNode>>>;

fn intersects(changed: &im::OrdSet<usize>, offset: usize, length: usize) -> bool {
    changed.range(offset..offset + length).next().is_some()
}

fn children(payload: &FactPayload) -> Result<(&FactPayload, &FactPayload), StoreDenial> {
    let FactStorage::Joined { children } = &payload.storage else {
        return Err(StoreDenial::ConflictingBody);
    };
    if payload.start != 0 || payload.len() != children[0].len() + children[1].len() {
        return Err(StoreDenial::ConflictingBody);
    }
    Ok((&children[0], &children[1]))
}

pub(super) fn count_nodes(
    payload: &FactPayload,
    changed: &im::OrdSet<usize>,
    offset: usize,
    control: StorageControl<'_, '_>,
) -> Result<usize, StoreDenial> {
    control.check_live()?;
    observation::native_patch_path();
    if !intersects(changed, offset, payload.len()) {
        return Ok(1);
    }
    match &payload.storage {
        FactStorage::Native(chunk) if chunk.facts.len() <= 32 => {
            chunk.clone_bytes.ok_or(StoreDenial::Representability)?;
            Ok(1)
        }
        FactStorage::Joined { .. } => {
            let (left, right) = children(payload)?;
            count_nodes(left, changed, offset, control)?
                .checked_add(count_nodes(right, changed, offset + left.len(), control)?)
                .and_then(|count| count.checked_add(1))
                .ok_or(StoreDenial::Representability)
        }
        // Opaque original/decoded storage keeps its existing cold path. Data
        // equality is never permission to relabel it as native patch storage.
        _ => Err(StoreDenial::ConflictingBody),
    }
}

pub(super) fn build(
    payload: &FactPayload,
    changed: &im::OrdSet<usize>,
    offset: usize,
    nodes: &mut Nodes,
    control: StorageControl<'_, '_>,
) -> Result<usize, StoreDenial> {
    control.check_live()?;
    observation::native_patch_path();
    let node = if !intersects(changed, offset, payload.len()) {
        PreparedNode::Keep(payload.clone())
    } else if let FactStorage::Native(chunk) = &payload.storage {
        let quote = chunk.clone_bytes.ok_or(StoreDenial::Representability)?;
        let copied_heaps = control.policy().reserve_memory(quote);
        control.check_live()?;
        let copied_heaps = copied_heaps?;
        let mut replacements = ExecutionArrayBuilder::allocate(payload.len(), control.policy())?;
        let mut replacement_heaps =
            ExecutionArrayBuilder::allocate(payload.len(), control.policy())?;
        for _ in 0..payload.len() {
            control.check_live()?;
            replacements.push(RefCell::new(None))?;
            replacement_heaps.push(std::sync::OnceLock::new())?;
        }
        let changed = changed
            .range(offset..offset + payload.len())
            .fold(0, |bits, ordinal| bits | (1_u32 << (*ordinal - offset)));
        PreparedNode::Leaf(PreparedLeaf {
            original: payload.clone(),
            changed,
            filled: 0,
            replacements: replacements.seal()?,
            output: ExecutionArrayBuilder::allocate(payload.len(), control.policy())?,
            copied_heaps,
            replacement_heaps: replacement_heaps.seal()?,
        })
    } else {
        let (left, right) = children(payload)?;
        let a = build(left, changed, offset, nodes, control)?;
        let b = build(right, changed, offset + left.len(), nodes, control)?;
        PreparedNode::Branch {
            left: a,
            right: b,
            left_len: left.len(),
            output: ExecutionArrayBuilder::allocate(2, control.policy())?,
        }
    };
    let index = nodes.len();
    nodes.push(RefCell::new(Some(node)))?;
    Ok(index)
}
