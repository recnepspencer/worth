//! Paid posting traversal retains the original generation and never scans a prefix again.
use super::*;
use crate::indexes::data::{
    BoundedEntityFieldBoundary, BoundedEntityFieldPage, BoundedEntityFieldPageStop as PageStop,
};
use worth_execution::{
    ExecutionAllocationPolicy, ExecutionArrayBuilder, ExecutionRequest, ExecutionWorkCeiling,
    MapKernelContext,
};

impl PreparedEntityFieldLookup {
    /// Read a finite posting page from this exact native selection.
    /// A continuation must name the same owner, basis, generation, kind and value.
    /// The caller supplies interruption, current admission and all page backing.
    pub fn execute_prepared_entity_field_page<E>(
        &self,
        value: &AspectValue,
        limit: usize,
        after: Option<&BoundedEntityFieldBoundary>,
        execution: &ExecutionRequest<'_, '_>,
        mut check_live: impl FnMut() -> Result<(), E>,
    ) -> Result<BoundedEntityFieldPage, PageStop<E>> {
        execution
            .in_scope(|lease| {
                ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(
                    lease,
                    None,
                    |meter| {
                        let allocation = lease
                            .map(ExecutionAllocationPolicy::Execution)
                            .unwrap_or(ExecutionAllocationPolicy::SystemAllocation);
                        self.posting_page(
                            value,
                            limit,
                            after,
                            execution,
                            &mut check_live,
                            allocation,
                            meter,
                        )
                    },
                )
            })
            .map_err(PageStop::Work)?
            .map_err(PageStop::Work)?
            .0
    }

    fn posting_page<E>(
        &self,
        value: &AspectValue,
        limit: usize,
        after: Option<&BoundedEntityFieldBoundary>,
        execution: &ExecutionRequest<'_, '_>,
        check_live: &mut impl FnMut() -> Result<(), E>,
        allocation: ExecutionAllocationPolicy<'_, '_>,
        meter: &mut MapKernelContext<'_, '_>,
    ) -> Result<BoundedEntityFieldPage, PageStop<E>> {
        self.page_admission(execution, check_live)?;
        if limit == 0 || limit == usize::MAX {
            return Err(PageStop::Lookup(denied(
                Denial::InvalidCandidateLimit,
                self.index_id(),
            )));
        }
        let bytes = AuthoritativeFieldComparisonKey::required_encoded_capacity_bytes(value)
            .and_then(|bytes| bytes.checked_add(self.basis.branch_id().0.len() as u64))
            .ok_or(PageStop::AccountingOverflow)?;
        charge(
            meter,
            bytes.checked_add(1).ok_or(PageStop::AccountingOverflow)?,
        )?;
        let memory = match allocation {
            ExecutionAllocationPolicy::Execution(lease) => {
                Some(lease.reserve_memory(bytes).map_err(PageStop::Memory)?)
            }
            ExecutionAllocationPolicy::SystemAllocation => None,
        };
        let key = AuthoritativeFieldComparisonKey::from_aspect_value(value);
        let start = match after {
            Some(boundary)
                if boundary.runtime == self.runtime_identity
                    && boundary.basis.basis().key() == self.basis.basis().key()
                    && Arc::ptr_eq(&boundary.generation, &self.generation)
                    && boundary.kind == self.kind
                    && boundary.key == key =>
            {
                boundary.next
            }
            Some(_) => return Err(PageStop::ForeignBoundary),
            None => 0,
        };
        self.performance_access().count_query_index_attempt();
        let DerivedIndexEntries::EntityField(entries) = &self.generation.entries else {
            unreachable!("prepared entity-field generation")
        };
        let rows = entries.get(&key);
        let length = rows.map_or(0, |rows| rows.len());
        if start > length {
            return Err(PageStop::ForeignBoundary);
        }
        let count = (length - start).min(limit);
        charge(meter, count as u64 + 1)?;
        let mut matching = 0usize;
        let mut previous = start
            .checked_sub(1)
            .and_then(|offset| rows.and_then(|rows| rows.get(offset)))
            .copied();
        for offset in start..start + count {
            charge(meter, 1)?;
            self.page_admission(execution, check_live)?;
            let entity = *rows
                .and_then(|rows| rows.get(offset))
                .expect("bounded posting offset");
            let observed = crate::indexes::projected_field_values::exact_entity_field_matches(
                self.basis.root(),
                entity,
                self.field_locator(),
                value,
            );
            let Some((kind, true)) = observed else {
                return Err(self.corrupt_page(offset + 1));
            };
            if previous.is_some_and(|prior| prior >= entity) {
                return Err(self.corrupt_page(offset + 1));
            }
            previous = Some(entity);
            if kind == self.kind {
                matching += 1;
            }
        }
        let mut ids =
            ExecutionArrayBuilder::allocate(matching, allocation).map_err(PageStop::Allocation)?;
        for offset in start..start + count {
            charge(meter, 1)?;
            self.page_admission(execution, check_live)?;
            let entity = *rows
                .and_then(|rows| rows.get(offset))
                .expect("validated posting offset");
            if crate::indexes::projected_field_values::exact_entity_field_matches(
                self.basis.root(),
                entity,
                self.field_locator(),
                value,
            )
            .is_some_and(|(kind, matches)| kind == self.kind && matches)
            {
                ids.push(entity).map_err(PageStop::Allocation)?;
            }
        }
        let ids = ids.seal().map_err(PageStop::Allocation)?;
        self.page_admission(execution, check_live)?;
        let boundary = (start + count < length).then(|| BoundedEntityFieldBoundary {
            runtime: self.runtime_identity,
            basis: self.basis.clone(),
            generation: Arc::clone(&self.generation),
            kind: self.kind,
            key,
            next: start + count,
            _memory: memory,
        });
        self.performance_access().count_query_index_path();
        Ok(BoundedEntityFieldPage {
            ids,
            generation: self.generation.generation_id,
            examined: count,
            boundary,
        })
    }
    fn page_admission<E>(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        check_live: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<(), PageStop<E>> {
        crate::execution::check_request(execution)
            .map_err(|cause| PageStop::Lookup(denied(Denial::Execution(cause), self.index_id())))?;
        check_live().map_err(PageStop::Admission)
    }
    fn corrupt_page<E>(&self, examined: usize) -> PageStop<E> {
        PageStop::Lookup(
            denied(Denial::CorruptIndexEntries, self.index_id())
                .with_examined_entry_count(examined),
        )
    }
}
fn charge<E>(meter: &mut MapKernelContext<'_, '_>, work: u64) -> Result<(), PageStop<E>> {
    meter
        .checkpoint(work)
        .map_err(|stop| PageStop::Work(worth_execution::WorkCeilingDenial::Stopped(stop)))
}
