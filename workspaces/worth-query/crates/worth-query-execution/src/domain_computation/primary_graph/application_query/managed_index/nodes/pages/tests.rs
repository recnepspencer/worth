//! Physical node backing only: no source, graph identity or read authority is fabricated.
use super::super::{
    WorthQueryManagedIndexNodeId as NodeId, WorthQueryManagedIndexNodeWriter as Writer,
    WorthQueryManagedIndexNodes as Nodes,
};
use super::*;
use crate::domain_computation::primary_graph::application_query::managed_index::{
    invocation::controlled, WorthQueryManagedIndexWorkDenial,
};
use crate::domain_computation::provider_session::WorthQueryPreparedProductReadSessionIdentity;
use std::num::NonZeroUsize;
use worth_execution::{CancellationSource, MapKernelStop, SerialRequest, WorkCeilingDenial};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

mod chain {
    //! Count actual immutable page identities across retained versions and key shares.
    use super::*;
    use std::collections::BTreeSet;

    mod cost {
        use super::*;
        #[test]
        fn retained_chain_pages_grow_with_touched_pages_instead_of_roster_copies() {
            for count in [16, 256] {
                let cancellation = CancellationSource::new();
                let backing = request(&cancellation);
                let execution = ExecutionRequest::serial(&backing);
                controlled(&execution, |control| {
                    let scratch = (0..32 * PAGE_SIZE)
                        .map(|i| RefCell::new(Some(i as u64)))
                        .collect::<Vec<_>>();
                    let mut versions =
                        vec![NodePages::from_scratch(&execution, &scratch, control)?];
                    for step in 0..count {
                        let mut edit = PageEdit::new(versions.last().unwrap().clone(), execution);
                        edit.replace(step % PAGE_SIZE, Some(10000 + step as u64), control)?;
                        versions.push(edit.seal(control)?);
                    }
                    let mut pages = BTreeSet::new();
                    for version in &versions {
                        collect(version.root.as_ref().unwrap(), &mut pages);
                    }
                    assert!(pages.len() <= 32 + count * 2);
                    assert!(
                        pages.len() < versions.len() * 32 / 2,
                        "retaining versions must share untouched pages"
                    );
                    let shares = versions.clone();
                    for version in &shares {
                        collect(version.root.as_ref().unwrap(), &mut pages);
                    }
                    assert!(
                        pages.len() <= 32 + count * 2,
                        "sharing across keys/forks must not copy a page"
                    );
                    assert_eq!(versions[0].get(0, control)?, Some(&0));
                    Ok(())
                })
                .unwrap();
            }
        }
    }
    fn collect(node: &Arc<Directory<u64>>, pages: &mut BTreeSet<usize>) {
        match &node.entry {
            Entry::Page(_) => {
                pages.insert(Arc::as_ptr(node) as usize);
            }
            Entry::Branch { left, right } => {
                collect(left, pages);
                collect(right, pages);
            }
        }
    }
}
mod churn;

fn request(cancel: &CancellationSource) -> SerialRequest {
    SerialRequest::from_policy(
        &ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, None, Some(1_000_000)),
        ),
        cancel.token(),
        None,
    )
}

fn leaf_at(pages: &NodePages<u64>, page: usize) -> &Arc<Directory<u64>> {
    let mut current = pages.root.as_ref().unwrap();
    while let Entry::Branch { left, right } = &current.entry {
        current = if page & (1 << (current.height - 1)) == 0 {
            left
        } else {
            right
        };
    }
    current
}

#[test]
fn isolated_node_pages_share_unchanged_backing_and_release_staged_edits() {
    let cancel = CancellationSource::new();
    let backing = request(&cancel);
    let execution = ExecutionRequest::serial(&backing);
    controlled(&execution, |control| {
        let values = (0..4 * PAGE_SIZE)
            .map(|index| RefCell::new(Some(index as u64)))
            .collect::<Vec<_>>();
        let original = NodePages::from_scratch(&execution, &values, control)?;
        let original_page = Arc::downgrade(leaf_at(&original, 0));
        let mut candidate = PageEdit::new(original.clone(), execution);
        candidate.replace(2, Some(999), control)?;
        candidate.replace(PAGE_SIZE + 7, None, control)?;
        assert_eq!(original.get(2, control)?, Some(&2));
        assert_eq!(candidate.get(2, control)?.map(|node| *node), Some(999));
        assert_eq!(
            original.get(PAGE_SIZE + 7, control)?,
            Some(&((PAGE_SIZE + 7) as u64))
        );
        assert_eq!(
            candidate.get(PAGE_SIZE + 7, control)?.map(|node| *node),
            None
        );
        let candidate = candidate.seal(control)?;
        assert!(!Arc::ptr_eq(leaf_at(&original, 0), leaf_at(&candidate, 0)));
        assert!(Arc::ptr_eq(leaf_at(&original, 3), leaf_at(&candidate, 3)));
        let staged_page = Arc::downgrade(leaf_at(&candidate, 0));
        drop(candidate);
        assert!(staged_page.upgrade().is_none());
        assert!(original_page.upgrade().is_some());
        // Growth crosses the old directory's address range without copying its leaves.
        let mut successor = PageEdit::new(original.clone(), execution);
        successor.replace(8 * PAGE_SIZE + 1, Some(500), control)?;
        let successor = successor.seal(control)?;
        assert!(Arc::ptr_eq(leaf_at(&original, 3), leaf_at(&successor, 3)));
        assert_eq!(successor.get(8 * PAGE_SIZE + 1, control)?, Some(&500));
        empty_inserted_page_is_pruned(&execution, &original, control)?;
        drop(successor);
        drop(original);
        assert!(original_page.upgrade().is_none());
        Ok(())
    })
    .unwrap();
    sibling_writers_seal_distinct_coordinates();
}

#[test]
fn original_request_stop_refuses_node_page_handoff_and_releases_staged_pages() {
    let first_cancel = CancellationSource::new();
    let first_backing = request(&first_cancel);
    let first = ExecutionRequest::serial(&first_backing);
    let original = controlled(&first, |control| {
        NodePages::from_scratch(&first, &[RefCell::new(Some(5_u64))], control).map_err(Into::into)
    })
    .unwrap();
    // Retained backing survives its construction request; the edit borrows a new one.
    drop(first_backing);
    let cancel = CancellationSource::new();
    let backing = request(&cancel);
    let execution = ExecutionRequest::serial(&backing);
    let result = controlled(&execution, |control| {
        let mut candidate = PageEdit::new(original.clone(), execution);
        candidate.replace(0, Some(8), control)?;
        cancel.cancel();
        assert!(candidate.replace(PAGE_SIZE, Some(13), control).is_err());
        Ok(candidate) // Catching the stop still cannot hand off this root.
    });
    assert!(matches!(
        result,
        Err(WorthQueryManagedIndexWorkDenial::Controlled(Stop::Request(
            WorkCeilingDenial::Stopped(MapKernelStop::Cancelled)
        )))
    ));
    assert_eq!(backing.memory().unwrap().charged_memory_bytes(), 0);
    let fresh_cancel = CancellationSource::new();
    let fresh_backing = request(&fresh_cancel);
    let fresh = ExecutionRequest::serial(&fresh_backing);
    controlled(&fresh, |control| {
        assert_eq!(original.get(0, control)?, Some(&5));
        let mut retry = PageEdit::new(original.clone(), fresh);
        retry.replace(0, Some(8), control)?;
        assert_eq!(retry.get(0, control)?.map(|node| *node), Some(8));
        Ok(())
    })
    .unwrap();
}

fn empty_inserted_page_is_pruned(
    execution: &ExecutionRequest<'_, '_>,
    original: &NodePages<u64>,
    control: &WorthQueryManagedIndexControl<'_>,
) -> Result<(), Stop> {
    let ordinal = 4 * PAGE_SIZE;
    let mut edited = PageEdit::new(original.clone(), *execution);
    edited.replace(ordinal, Some(1), control)?;
    edited.replace(ordinal, None, control)?;
    assert_eq!(edited.get(ordinal, control)?.copied(), None);
    let edited = edited.seal(control)?;
    assert_eq!(edited.get(ordinal, control)?.copied(), None);
    assert!(
        Arc::ptr_eq(leaf_at(original, 3), leaf_at(&edited, 3)),
        "untouched page stays shared"
    );
    Ok(())
}

fn retained_node_snapshot() -> (Nodes<u64>, NodeId) {
    // Reserve a genuine descriptive namespace through its existing owner.
    // This reservation is not an admitted Source or a graph-read authority.
    let reserved = WorthQueryPreparedProductReadSessionIdentity::reserve().unwrap();
    let cancel = CancellationSource::new();
    let backing = request(&cancel);
    let execution = ExecutionRequest::serial(&backing);
    controlled(&execution, |control| {
        let mut writer = Writer::allocate(&execution, reserved.identity(), 1, 1, control)?;
        let root = writer.push(5_u64)?;
        Ok((writer.seal(&execution, Some(root))?, root))
    })
    .unwrap()
}

fn sibling_writers_seal_distinct_coordinates() {
    let (original, root) = retained_node_snapshot();
    let cancel = CancellationSource::new();
    let backing = request(&cancel);
    let execution = ExecutionRequest::serial(&backing);
    let (left, right, a, b) = controlled(&execution, |control| {
        let mut left = original.edit(execution, 2, 2, control)?;
        let mut right = original.edit(execution, 2, 2, control)?;
        let a = left.push(11)?;
        let b = right.push(13)?;
        assert_ne!(a, b, "sibling writers share the real coordinate issuer");
        left.replace(root, 6)?;
        right.replace(root, 7)?;
        let left = left.seal(&execution, Some(root))?;
        let right = right.seal(&execution, Some(root))?;
        assert_eq!(left.root(), Some(root));
        assert_eq!(right.root(), Some(root));
        assert_eq!((left.len(), right.len(), original.len()), (2, 2, 1));
        assert_eq!(left.get(a, control)?, &11);
        assert_eq!(right.get(b, control)?, &13);
        assert_eq!(left.get(root, control)?, &6);
        assert_eq!(right.get(root, control)?, &7);
        assert_eq!(original.get(root, control)?, &5);
        Ok((left, right, a, b))
    })
    .unwrap();
    assert_unwritten(&left, b);
    assert_unwritten(&right, a);
    assert_unwritten(&original, a);
    assert_unwritten(&original, b);
    let successor = controlled(&execution, |control| {
        let mut writer = left.edit(execution, 2, 2, control)?;
        writer.remove(a)?;
        let replacement = writer.push(17)?;
        assert_ne!(replacement, a, "retired writer IDs are never recycled");
        assert_ne!(replacement, b, "a sibling ID is never recycled");
        let successor = writer.seal(&execution, Some(root))?;
        assert_eq!(successor.get(replacement, control)?, &17);
        assert_eq!(left.get(a, control)?, &11);
        assert_eq!(original.get(root, control)?, &5);
        Ok(successor)
    })
    .unwrap();
    assert_unwritten(&successor, a);
}

fn assert_unwritten(nodes: &Nodes<u64>, id: NodeId) {
    // A denied read latches the first stop, so each denial gets a fresh call.
    let cancel = CancellationSource::new();
    let backing = request(&cancel);
    let execution = ExecutionRequest::serial(&backing);
    let result = controlled(&execution, |control| {
        nodes.get(id, control)?;
        Ok(())
    });
    assert!(matches!(
        result,
        Err(WorthQueryManagedIndexWorkDenial::Controlled(
            Stop::UnwrittenNode
        ))
    ));
}
