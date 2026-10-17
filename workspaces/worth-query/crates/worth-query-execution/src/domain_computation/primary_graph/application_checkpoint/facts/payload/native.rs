//! Pre-effect admission of native publication chunks and their join slots.
//! Nested Fact heaps retain their original native admission. These arrays own
//! inline payload backing, under Execution's existing metadata exclusions.
use super::{Fact, FactPayload, StorageControl};
use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial;
use crate::domain_computation::primary_graph::fact_verification_observation;
use std::cell::RefCell;
use worth_execution::{ExecutionAllocationDenial, ExecutionArray, ExecutionArrayBuilder};

const CHUNK: usize = 32;
type AuthorSlots<T> = ExecutionArray<RefCell<Option<T>>>;

pub(in crate::domain_computation::primary_graph) struct NativeFactPayloadBuilder {
    chunks: AuthorSlots<ExecutionArrayBuilder<Fact>>,
    joins: AuthorSlots<ExecutionArrayBuilder<FactPayload>>,
    roots: AuthorSlots<FactPayload>,
    count: usize,
    written: usize,
}

impl NativeFactPayloadBuilder {
    pub(in crate::domain_computation::primary_graph) fn allocate(
        count: usize,
        control: StorageControl<'_, '_>,
    ) -> Result<Self, StoreDenial> {
        control.check_live()?;
        let chunk_count = count.div_ceil(CHUNK).max(1);
        let chunks = author_slots(chunk_count, control, |chunk| {
            ExecutionArrayBuilder::allocate(
                count.saturating_sub(chunk * CHUNK).min(CHUNK),
                control.policy(),
            )
            .map(Some)
            .map_err(StoreDenial::from)
        })?;
        let joins = author_slots(chunk_count - 1, control, |_| {
            ExecutionArrayBuilder::allocate(2, control.policy())
                .map(Some)
                .map_err(StoreDenial::from)
        })?;
        let roots = author_slots(chunk_count, control, |_| Ok(None))?;
        control.check_live()?;
        Ok(Self {
            chunks,
            joins,
            roots,
            count,
            written: 0,
        })
    }

    pub(in crate::domain_computation::primary_graph) fn push(
        &mut self,
        fact: Fact,
    ) -> Result<(), ExecutionAllocationDenial> {
        assert!(self.written < self.count, "prepared native fact count");
        self.chunks[self.written / CHUNK]
            .borrow_mut()
            .as_mut()
            .expect("unsealed native chunk")
            .push(fact)?;
        fact_verification_observation::native_payload_fact_move();
        self.written += 1;
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn seal(
        self,
    ) -> Result<FactPayload, ExecutionAllocationDenial> {
        assert_eq!(self.written, self.count, "complete native publication only");
        for (index, chunk) in self.chunks.into_iter().enumerate() {
            let chunk = chunk
                .into_inner()
                .expect("one admitted native chunk")
                .seal()?;
            *self.roots[index].borrow_mut() = Some(FactPayload::native(chunk));
            fact_verification_observation::native_payload_chunk();
        }
        // Pair adjacent subtrees in the pre-admitted scratch slots. Taking each
        // child moves its admission; no fact copy or new policy admission occurs.
        let mut joins = self.joins.into_iter();
        let mut width = self.roots.len();
        while width > 1 {
            let mut write = 0;
            for read in (0..width).step_by(2) {
                let left = self.roots[read]
                    .borrow_mut()
                    .take()
                    .expect("completed subtree");
                let root = if read + 1 < width {
                    let right = self.roots[read + 1]
                        .borrow_mut()
                        .take()
                        .expect("completed subtree");
                    let mut children = joins
                        .next()
                        .expect("one slot per native join")
                        .into_inner()
                        .expect("admitted join");
                    children.push(left)?;
                    children.push(right)?;
                    let root = FactPayload::joined(children.seal()?);
                    fact_verification_observation::native_payload_join();
                    root
                } else {
                    left
                };
                *self.roots[write].borrow_mut() = Some(root);
                write += 1;
            }
            width = write;
        }
        let root = self.roots[0]
            .borrow_mut()
            .take()
            .expect("even an empty payload has a root");
        Ok(root)
    }
}

fn author_slots<T>(
    count: usize,
    control: StorageControl<'_, '_>,
    mut value: impl FnMut(usize) -> Result<Option<T>, StoreDenial>,
) -> Result<AuthorSlots<T>, StoreDenial> {
    let mut slots = ExecutionArrayBuilder::allocate(count, control.policy())?;
    for index in 0..count {
        control.check_live()?;
        slots.push(RefCell::new(value(index)?))?;
    }
    Ok(slots.seal()?)
}
