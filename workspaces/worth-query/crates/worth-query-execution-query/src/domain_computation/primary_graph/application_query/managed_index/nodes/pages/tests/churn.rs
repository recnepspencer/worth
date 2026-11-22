//! Physical gap invariance plus genuinely issued writer IDs; no Source authority.
use super::*;
use std::collections::BTreeSet;
use worth_execution::ExecutionWorkCeiling;

fn cells(root: &Option<Arc<Directory<u64>>>) -> usize {
    root.as_ref().map_or(0, |node| {
        1 + match &node.entry {
            Entry::Page(_) => 0,
            Entry::Branch { left, right } => {
                cells(&Some(left.clone())) + cells(&Some(right.clone()))
            }
        }
    })
}
fn physical(keys: [usize; 3]) -> (u64, u64) {
    let cancel = CancellationSource::new();
    let backing = request(&cancel);
    let execution = ExecutionRequest::serial(&backing);
    let original = controlled(&execution, |control| {
        let mut edit = PageEdit::new(NodePages::default(), execution);
        for (index, key) in keys.into_iter().enumerate() {
            edit.replace(key * PAGE_SIZE, Some(10 + index as u64), control)?;
        }
        edit.seal(control).map_err(Into::into)
    })
    .unwrap();
    assert_eq!(cells(&original.root), 5);
    let memory = backing.memory().unwrap().charged_memory_bytes();
    let ((), report) = ExecutionWorkCeiling::new(None)
        .run_with_checkpoint(None, Some(&backing), |_| {
            controlled(&execution, |control| {
                for (index, key) in keys.into_iter().enumerate() {
                    assert_eq!(
                        original.get(key * PAGE_SIZE, control)?,
                        Some(&(10 + index as u64))
                    );
                }
                let mut edit = PageEdit::new(original.clone(), execution);
                edit.replace(keys[1] * PAGE_SIZE, Some(99), control)?;
                let sibling = edit.seal(control)?;
                assert_eq!(sibling.get(keys[1] * PAGE_SIZE, control)?, Some(&99));
                assert_eq!(original.get(keys[1] * PAGE_SIZE, control)?, Some(&11));
                assert!(Arc::ptr_eq(
                    leaf_at(&original, keys[0]),
                    leaf_at(&sibling, keys[0])
                ));
                let mut edit = PageEdit::new(original.clone(), execution);
                edit.replace(keys[1] * PAGE_SIZE, None, control)?;
                let two = edit.seal(control)?;
                assert_eq!(cells(&two.root), 3);
                assert_eq!(two.get(keys[1] * PAGE_SIZE, control)?, None);
                let mut edit = PageEdit::new(two, execution);
                edit.replace(keys[2] * PAGE_SIZE, None, control)?;
                let one = edit.seal(control)?;
                assert_eq!(cells(&one.root), 1);
                assert!(Arc::ptr_eq(
                    one.root.as_ref().unwrap(),
                    leaf_at(&original, keys[0])
                ));
                assert_eq!(one.get(keys[2] * PAGE_SIZE, control)?, None);
                // Nearby absent pages cannot alias a compressed leaf or skipped interval.
                assert_eq!(original.get((keys[2] - 1) * PAGE_SIZE, control)?, None);
                Ok(())
            })
            .unwrap();
        })
        .unwrap();
    assert_eq!(backing.memory().unwrap().charged_memory_bytes(), memory);
    drop(original);
    assert_eq!(backing.memory().unwrap().charged_memory_bytes(), 0);
    (report.charged_work(), memory)
}
#[test]
fn compressed_live_pages_skip_historical_coordinate_gaps_and_collapse_retirements() {
    let max = usize::MAX >> PAGE_BITS;
    let mut reports = Vec::new();
    for bit in [2, 8, 24, usize::BITS - PAGE_BITS - 1] {
        reports.push(physical([0, 1usize << bit, (1usize << bit) + 2]));
    }
    reports.push(physical([0, max - 2, max]));
    eprintln!("compressed fixed-three-page gap lookup/edit SDK and retained bytes={reports:?}");
    assert!(
        reports.iter().all(|result| *result == reports[0]),
        "equal live shapes pay equal original work/memory across address gaps"
    );
    // Start entirely above zero, then insert a lower page: the old root is on
    // the right of the new discriminating bit, not an implicit left spine.
    let cancel = CancellationSource::new();
    let backing = request(&cancel);
    let execution = ExecutionRequest::serial(&backing);
    controlled(&execution, |control| {
        let mut edit = PageEdit::new(NodePages::default(), execution);
        edit.replace((max - 2) * PAGE_SIZE, Some(5u64), control)?;
        let old = edit.seal(control)?;
        assert_eq!(old.get(0, control)?, None);
        let mut edit = PageEdit::new(old.clone(), execution);
        edit.replace(2 * PAGE_SIZE, Some(7), control)?;
        let new = edit.seal(control)?;
        assert_eq!(new.get((max - 2) * PAGE_SIZE, control)?, Some(&5));
        assert_eq!(new.get(2 * PAGE_SIZE, control)?, Some(&7));
        assert_eq!(old.get(2 * PAGE_SIZE, control)?, None);
        assert!(Arc::ptr_eq(leaf_at(&old, max - 2), leaf_at(&new, max - 2)));
        Ok(())
    })
    .unwrap();
    assert_eq!(backing.memory().unwrap().charged_memory_bytes(), 0);
}

#[test]
fn genuine_node_churn_keeps_live_backing_bounded_without_reusing_issued_ids() {
    let (original, root) = retained_node_snapshot();
    let cancel = CancellationSource::new();
    let backing = request(&cancel);
    let execution = ExecutionRequest::serial(&backing);
    let mut retired = Vec::new();
    let mut issued = BTreeSet::new();
    let (mut current, sibling, sibling_id) = controlled(&execution, |control| {
        let current = original
            .edit(execution, 2, 1, control)?
            .seal(&execution, Some(root))?;
        let mut sibling = original.edit(execution, 2, 1, control)?;
        let sibling_id = sibling.push(999)?;
        issued.insert(sibling_id.ordinal);
        let sibling = sibling.seal(&execution, Some(root))?;
        Ok((current, sibling, sibling_id))
    })
    .unwrap();
    let mut previous = None;
    for value in 0..4096u64 {
        let turn_cancel = CancellationSource::new();
        let turn_backing = request(&turn_cancel);
        let turn_execution = ExecutionRequest::serial(&turn_backing);
        let (next, id) = controlled(&turn_execution, |turn_control| {
            let mut edit = current.edit(turn_execution, 2, 1, turn_control)?;
            if let Some(id) = previous {
                edit.remove(id)?;
            }
            let id = edit.push(10_000 + value)?;
            let next = edit.seal(&turn_execution, Some(root))?;
            assert_eq!(next.len(), 2);
            assert!(
                cells(&next.pages.root) <= 3,
                "only two live pages can remain"
            );
            assert_eq!(next.get(root, turn_control)?, &5);
            assert_eq!(next.get(id, turn_control)?, &(10_000 + value));
            Ok((next, id))
        })
        .unwrap();
        if let Some(id) = previous {
            retired.push(id);
        }
        assert!(
            issued.insert(id.ordinal),
            "the actual issuer never recycles old/sibling IDs"
        );
        current = next;
        previous = Some(id);
    }
    let last = previous.unwrap();
    assert!(
        last.ordinal > 4096,
        "the real shared issuer advanced through all retirements"
    );
    assert_eq!(cells(&current.pages.root), 3);
    for id in retired.into_iter().step_by(31).chain([sibling_id]) {
        assert_unwritten(&current, id);
    }
    assert_unwritten(&sibling, last);
    assert_unwritten(&original, last);
    controlled(&execution, |control| {
        assert_eq!(original.get(root, control)?, &5);
        assert_eq!(sibling.get(sibling_id, control)?, &999);
        assert_eq!(current.get(last, control)?, &14_095);
        Ok(())
    })
    .unwrap();
    let ((), report) = ExecutionWorkCeiling::new(None)
        .run_with_checkpoint(None, Some(&backing), |_| {
            controlled(&execution, |control| {
                assert_eq!(current.get(root, control)?, &5);
                assert_eq!(current.get(last, control)?, &14_095);
                let mut edit = current.edit(execution, 2, 1, control)?;
                edit.replace(last, 17)?;
                let successor = edit.seal(&execution, Some(root))?;
                assert_eq!(successor.get(last, control)?, &17);
                assert_eq!(current.get(last, control)?, &14_095);
                Ok(())
            })
            .unwrap();
        })
        .unwrap();
    eprintln!(
        "genuine4096-issue two-live-node final lookup/edit whole original SDK={}",
        report.charged_work()
    );
    drop((current, sibling));
    assert_eq!(backing.memory().unwrap().charged_memory_bytes(), 0);
}
