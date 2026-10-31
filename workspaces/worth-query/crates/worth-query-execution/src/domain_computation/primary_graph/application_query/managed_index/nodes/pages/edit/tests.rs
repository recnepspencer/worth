//! Physical edit custody only; no Source or audience authority is constructed.
use super::*;
use crate::domain_computation::primary_graph::application_query::managed_index::invocation::controlled;
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
