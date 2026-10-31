//! Physical edit custody only; no Source or audience authority is constructed.
use super::*;
use crate::domain_computation::primary_graph::application_query::managed_index::{
    invocation::controlled, WorthQueryManagedIndexWorkDenial as Denial,
};
use worth_execution::{
    CancellationSource, ExecutionWorkCeiling, SerialMemoryBudget, SerialRequest,
};

#[test]
fn working_page_reuses_initialized_allocation_and_preserves_tombstones() {
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&request);
    controlled(&execution, |control| {
        let cells = (0..2 * PAGE_SIZE)
            .map(|i| Cell::new(Some(i as u64)))
            .collect::<Vec<_>>();
        let base = NodePages::from_scratch(&execution, &cells, control)?;
        let mut edit = PageEdit::new(base.clone(), execution);
        edit.replace(0, Some(90), control)?;
        let page = Rc::clone(edit.touched.get(&0).unwrap());
        let pointer = page
            .open
            .borrow()
            .builder
            .as_ref()
            .unwrap()
            .elements()
            .as_ptr();
        for value in 0..128 {
            edit.replace(1, Some(value), control)?;
            assert_eq!(edit.get(1, control)?, Some(value));
        }
        edit.replace(0, None, control)?;
        assert_eq!(
            edit.get(0, control)?,
            None,
            "tombstone overrides actual base"
        );
        assert_eq!(base.get(0, control)?, Some(&0));
        assert_eq!(edit.touched.len(), 1);
        assert_eq!(
            page.open
                .borrow()
                .builder
                .as_ref()
                .unwrap()
                .elements()
                .as_ptr(),
            pointer
        );
        let frozen = edit.seal(control)?;
        assert!(
            page.open.borrow().builder.is_none(),
            "seal moves the sole builder"
        );
        assert_eq!(
            frozen.page_values(0, control)?.unwrap().elements().as_ptr(),
            pointer
        );
        assert_eq!(frozen.get(0, control)?, None);
        assert_eq!(frozen.get(1, control)?, Some(&127));
        assert_eq!(base.get(1, control)?, Some(&1));
        assert!(std::ptr::eq(
            base.page_values(1, control)?.unwrap(),
            frozen.page_values(1, control)?.unwrap()
        ));
        Ok(())
    })
    .unwrap();
    assert_eq!(request.memory().unwrap().charged_memory_bytes(), 0);
}

fn edit_invoice(writes: usize, distinct: bool) -> u64 {
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&request);
    let base = controlled(&execution, |control| {
        NodePages::from_scratch(&execution, &[Cell::new(Some(1_u64))], control).map_err(Into::into)
    })
    .unwrap();
    let ((), report) = ExecutionWorkCeiling::new(None)
        .run_with_checkpoint(None, Some(&request), |_| {
            controlled(&execution, |control| {
                let mut edit = PageEdit::new(base.clone(), execution);
                for i in 0..writes {
                    edit.replace(
                        if distinct {
                            i * PAGE_SIZE
                        } else {
                            i % PAGE_SIZE
                        },
                        Some(i as u64),
                        control,
                    )?;
                }
                assert_eq!(edit.touched.len(), if distinct { writes } else { 1 });
                let frozen = edit.seal(control)?;
                assert_eq!(
                    frozen.get(
                        if distinct {
                            (writes - 1) * PAGE_SIZE
                        } else {
                            (writes - 1) % PAGE_SIZE
                        },
                        control
                    )?,
                    Some(&((writes - 1) as u64))
                );
                Ok(())
            })
            .unwrap();
        })
        .unwrap();
    report.charged_work()
}

#[test]
fn working_page_original_invoice_tracks_touched_pages_instead_of_copying_every_write() {
    let repeated = edit_invoice(64, false);
    let doubled = edit_invoice(128, false);
    let distinct = edit_invoice(64, true);
    eprintln!("page editor original invoices repeated64={repeated}, repeated128={doubled}, distinct64={distinct}");
    assert!(
        doubled <= repeated * 2,
        "repeated writes remain bounded by actual writes"
    );
    assert!(
        distinct > doubled,
        "distinct physical pages pay their actual copies and installation"
    );
}

fn prepared<'a>(
    base: &NodePages<u64>,
    count: usize,
    execution: ExecutionRequest<'a, 'a>,
) -> PageEdit<'a, u64> {
    controlled(&execution, |control| {
        let mut edit = PageEdit::new(base.clone(), execution);
        for page in 0..count {
            edit.replace(page * PAGE_SIZE, Some(10_000 + page as u64), control)?;
        }
        Ok(edit)
    })
    .unwrap()
}

fn seal_invoice(
    base: &NodePages<u64>,
    count: usize,
    ceiling: Option<u64>,
) -> (Result<(), Denial>, usize, u64) {
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&request);
    let edit = prepared(base, count, execution);
    let allocations = Rc::clone(&edit.sealed_branches);
    let (result, report) = ExecutionWorkCeiling::new(ceiling)
        .run_with_checkpoint(None, Some(&request), |_| {
            controlled(&execution, |control| edit.seal(control).map_err(Into::into))
        })
        .unwrap();
    let result = result.map(|frozen| {
        controlled(&execution, |control| {
            assert_eq!(frozen.get(0, control)?, Some(&10_000));
            assert_eq!(
                frozen.get((count - 1) * PAGE_SIZE, control)?,
                Some(&(10_000 + count as u64 - 1))
            );
            assert_eq!(base.get(0, control)?, Some(&0));
            assert!(std::ptr::eq(
                base.page_values(255, control)?.unwrap(),
                frozen.page_values(255, control)?.unwrap()
            ));
            Ok(())
        })
        .unwrap();
    });
    assert_eq!(
        request.memory().unwrap().charged_memory_bytes(),
        0,
        "no partial output or prepared scratch grant survives either outcome"
    );
    (result, allocations.get(), report.charged_work())
}

#[test]
fn working_page_batch_seals_shared_directory_ancestors_once() {
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&request);
    let base = controlled(&execution, |control| {
        let cells = (0..256 * PAGE_SIZE)
            .map(|i| Cell::new(Some(i as u64)))
            .collect::<Vec<_>>();
        NodePages::from_scratch(&execution, &cells, control).map_err(Into::into)
    })
    .unwrap();
    let (first, first_branches, first_work) = seal_invoice(&base, 64, None);
    first.unwrap();
    let (doubled, doubled_branches, doubled_work) = seal_invoice(&base, 128, None);
    doubled.unwrap();
    assert_eq!(
        (first_branches, doubled_branches),
        (65, 128),
        "complete dyadic prefixes need their interior cells plus only the shared path to the root"
    );
    eprintln!("batch seal actual branch allocations64/128={first_branches}/{doubled_branches}; whole original SDK seal work={first_work}/{doubled_work}");
    assert!(first_work > 0 && doubled_work <= first_work * 21 / 10);
    controlled(&execution, |control| {
        let mut growth = PageEdit::new(base.clone(), execution);
        growth.replace(512 * PAGE_SIZE, Some(89), control)?;
        growth.replace(513 * PAGE_SIZE, Some(97), control)?;
        let allocations = Rc::clone(&growth.sealed_branches);
        let growth = growth.seal(control)?;
        assert_eq!(
            allocations.get(),
            2,
            "one adjacent-page branch and one root; absent bit levels have no cells"
        );
        assert_eq!(growth.root.as_ref().unwrap().height, 10);
        assert!(std::ptr::eq(
            base.page_values(0, control)?.unwrap(),
            growth.page_values(0, control)?.unwrap()
        ));
        drop(growth);
        let mut left = PageEdit::new(base.clone(), execution);
        let mut right = PageEdit::new(base.clone(), execution);
        left.replace(0, Some(71), control)?;
        right.replace(0, Some(73), control)?;
        // Two adjacent new pages share their entire new path.
        left.replace(512 * PAGE_SIZE, Some(89), control)?;
        left.replace(513 * PAGE_SIZE, Some(97), control)?;
        // Delete a complete old page, not just one value.
        for ordinal in PAGE_SIZE..2 * PAGE_SIZE {
            left.replace(ordinal, None, control)?;
        }
        let left = left.seal(control)?;
        let right = right.seal(control)?;
        assert_eq!(left.get(0, control)?, Some(&71));
        assert_eq!(right.get(0, control)?, Some(&73));
        assert_eq!(base.get(0, control)?, Some(&0));
        assert_eq!(left.get(PAGE_SIZE, control)?, None);
        assert_eq!(base.get(PAGE_SIZE, control)?, Some(&(PAGE_SIZE as u64)));
        assert_eq!(left.get(512 * PAGE_SIZE, control)?, Some(&89));
        assert_eq!(right.get(512 * PAGE_SIZE, control)?, None);
        assert_eq!(base.get(513 * PAGE_SIZE, control)?, None);
        assert_eq!(left.get(513 * PAGE_SIZE, control)?, Some(&97));
        assert!(!Arc::ptr_eq(
            base.root.as_ref().unwrap(),
            right.root.as_ref().unwrap()
        ));
        assert!(std::ptr::eq(
            base.page_values(255, control)?.unwrap(),
            left.page_values(255, control)?.unwrap()
        ));
        let mut empty_growth = PageEdit::new(base.clone(), execution);
        empty_growth.replace(4096 * PAGE_SIZE, Some(1), control)?;
        empty_growth.replace(4096 * PAGE_SIZE, None, control)?;
        let allocations = Rc::clone(&empty_growth.sealed_branches);
        let empty_growth = empty_growth.seal(control)?;
        assert_eq!(
            allocations.get(),
            0,
            "empty growth allocates no final or discarded wrappers"
        );
        assert!(Arc::ptr_eq(
            empty_growth.root.as_ref().unwrap(),
            base.root.as_ref().unwrap()
        ));
        Ok(())
    })
    .unwrap();
    drop(base);
    assert_eq!(request.memory().unwrap().charged_memory_bytes(), 0);
}

#[test]
fn working_page_batch_refusal_preserves_predecessor_and_releases_partial_seal() {
    use worth_execution::{MapKernelStop, WorkCeilingDenial};
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&request);
    let base = controlled(&execution, |control| {
        let cells = (0..256 * PAGE_SIZE)
            .map(|i| Cell::new(Some(i as u64)))
            .collect::<Vec<_>>();
        NodePages::from_scratch(&execution, &cells, control).map_err(Into::into)
    })
    .unwrap();
    let sibling = base.clone();
    let baseline = request.memory().unwrap().charged_memory_bytes();
    let (funded, _, work) = seal_invoice(&base, 64, None);
    funded.unwrap();
    let (limited, branches, _) = seal_invoice(&base, 64, Some(work / 2));
    assert!(
        branches > 0,
        "the original work ceiling stops after actual partial seal progress"
    );
    assert!(
        matches!(
            limited,
            Err(Denial::Controlled(Stop::Request(
                WorkCeilingDenial::Stopped(MapKernelStop::WorkCeiling)
            )))
        ),
        "{limited:?}"
    );
    assert_eq!(request.memory().unwrap().charged_memory_bytes(), baseline);
    controlled(&execution, |control| {
        for ordinal in [0, PAGE_SIZE, 63 * PAGE_SIZE, 255 * PAGE_SIZE] {
            assert_eq!(base.get(ordinal, control)?, Some(&(ordinal as u64)));
            assert_eq!(sibling.get(ordinal, control)?, Some(&(ordinal as u64)));
        }
        Ok(())
    })
    .unwrap();
    seal_invoice(&base, 64, None).0.unwrap();
    drop((base, sibling));
    assert_eq!(request.memory().unwrap().charged_memory_bytes(), 0);
}
