//! Fallible in-place posting order: O(n log n) comparisons, O(1) scratch.
//! Exact keys bound collision work; ordinal ties select the original first
//! occurrence deterministically. Every comparison and move is admitted first.

use std::cmp::Ordering;

use super::{CompanionPreflightStop, IndexAdmission, ProjectedKey};
use crate::domain_computation::primary_graph::output_lineage::invalidation::admission::key_traversal_work;

pub(super) fn sort(
    projected: &mut [ProjectedKey],
    admission: &mut impl IndexAdmission,
) -> Result<(), CompanionPreflightStop> {
    for root in (0..projected.len() / 2).rev() {
        sift_down(projected, root, admission)?;
    }
    for end in (1..projected.len()).rev() {
        admission.work(1)?;
        projected.swap(0, end);
        sift_down(&mut projected[..end], 0, admission)?;
    }
    Ok(())
}

fn sift_down(
    projected: &mut [ProjectedKey],
    mut root: usize,
    admission: &mut impl IndexAdmission,
) -> Result<(), CompanionPreflightStop> {
    // Only internal nodes have children; the condition also bounds 2*root+1.
    while root < projected.len() / 2 {
        let mut child = root * 2 + 1;
        if child + 1 < projected.len()
            && compare(&projected[child], &projected[child + 1], admission)? == Ordering::Less
        {
            child += 1;
        }
        if compare(&projected[root], &projected[child], admission)? != Ordering::Less {
            break;
        }
        admission.work(1)?;
        projected.swap(root, child);
        root = child;
    }
    Ok(())
}

fn compare(
    left: &ProjectedKey,
    right: &ProjectedKey,
    admission: &mut impl IndexAdmission,
) -> Result<Ordering, CompanionPreflightStop> {
    admission.work(1)?;
    let hash_order = left.hash.cmp(&right.hash);
    if hash_order != Ordering::Equal {
        return Ok(hash_order);
    }
    // Bound traversal before inspecting variable path/value payloads. Charge
    // the larger complete key bound before the actual lexicographic compare.
    admission.work(
        key_traversal_work(&left.key)
            .checked_add(key_traversal_work(&right.key))
            .ok_or(CompanionPreflightStop::WorkCounterOverflow)?,
    )?;
    let left_work = left
        .key
        .comparison_work_bound()
        .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
    let right_work = right
        .key
        .comparison_work_bound()
        .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
    admission.work(left_work.max(right_work))?;
    let key_order = left.key.cmp(&right.key);
    if key_order != Ordering::Equal {
        return Ok(key_order);
    }
    admission.work(1)?;
    Ok(left.ordinal.cmp(&right.ordinal))
}
