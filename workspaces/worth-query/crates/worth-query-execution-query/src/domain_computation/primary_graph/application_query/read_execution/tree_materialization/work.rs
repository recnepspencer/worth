use super::super::OneShotReadWorkObservation;
use super::WorthQueryApplicationReadExecutionDenial;
use crate::domain_computation::primary_graph::read_observation::WorthQueryInvariantProjectionWork;

#[derive(Clone, Copy, Debug)]
struct WindowByteBudget {
    remaining_bytes: usize,
    charged_bytes: usize,
    exceeded_requested_bytes: Option<usize>,
}

pub struct ResultTreeWork<'a, 'request, 'authority> {
    material: Option<super::material_storage::MaterialTreeStorage<'request, 'authority>>,
    interruption: super::super::ReadInterruption<'a>,
    window_budget: Option<WindowByteBudget>,

    pub projected_records: usize,
    pub projected_fields: usize,
    pub reads: WorthQueryInvariantProjectionWork,
    pub ordered_index_entries_examined: usize,
    pub relation_predicate_work_units: usize,
    pub ordering_comparisons: usize,
    pub work_units: usize,
    spent: Option<&'a OneShotReadWorkObservation<'a>>,
}

impl<'a, 'request, 'authority> ResultTreeWork<'a, 'request, 'authority> {
    pub fn new(
        interruption: super::super::ReadInterruption<'a>,
        spent: Option<&'a OneShotReadWorkObservation<'a>>,
        execution: worth_execution::ExecutionRequest<'request, 'authority>,
        allocation: Option<
            &std::sync::Arc<
                crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation,
            >,
        >,
    ) -> Self {
        Self {
            material: allocation.map(|allocation| {
                super::material_storage::MaterialTreeStorage::new(
                    execution,
                    std::sync::Arc::clone(allocation),
                )
            }),
            interruption,
            window_budget: None,

            projected_records: 0,
            projected_fields: 0,
            reads: WorthQueryInvariantProjectionWork::default(),
            ordered_index_entries_examined: 0,
            relation_predicate_work_units: 0,
            ordering_comparisons: 0,
            work_units: 0,
            spent,
        }
    }

    pub fn set_window_budget(&mut self, remaining_bytes: usize) {
        self.window_budget = Some(WindowByteBudget {
            remaining_bytes,
            charged_bytes: 0,
            exceeded_requested_bytes: None,
        });
    }

    pub fn clear_window_budget(&mut self) {
        self.window_budget = None;
    }

    pub const fn has_window_budget(&self) -> bool {
        self.window_budget.is_some()
    }

    pub fn window_charged_bytes(&self) -> usize {
        self.window_budget.map_or(0, |budget| budget.charged_bytes)
    }

    pub fn window_budget_exceeded(&self) -> Option<usize> {
        self.window_budget
            .and_then(|budget| budget.exceeded_requested_bytes)
    }

    /// Checks and deducts `bytes` from the active streaming window budget BEFORE
    /// allocating or cloning fields/children into the materialized tree.
    pub fn charge_window_bytes(
        &mut self,
        bytes: usize,
        subject: &str,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        let Some(budget) = self.window_budget.as_mut() else {
            return Ok(());
        };
        let next_charged = budget.charged_bytes.saturating_add(bytes);
        if bytes > budget.remaining_bytes {
            budget.exceeded_requested_bytes = Some(next_charged);
            return Err(super::result_buffer_denial(subject));
        }
        budget.remaining_bytes -= bytes;
        budget.charged_bytes = next_charged;
        Ok(())
    }

    /// Checks temporary iterator/scratch memory against the remaining streaming
    /// window budget without permanently retaining it after the step finishes.
    pub fn check_transient_window_bytes(
        &mut self,
        bytes: u64,
        subject: &str,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        let Some(budget) = self.window_budget.as_mut() else {
            return Ok(());
        };
        if bytes > budget.remaining_bytes as u64 {
            let requested = usize::try_from(bytes).unwrap_or(usize::MAX);
            budget.exceeded_requested_bytes = Some(budget.charged_bytes.saturating_add(requested));
            return Err(super::result_buffer_denial(subject));
        }
        Ok(())
    }

    pub fn materialized(&self) -> bool {
        self.material.is_some()
    }
    pub fn collect_material<T>(
        &self,
        capacity: usize,
        values: impl Iterator<Item = T>,
        subject: &str,
    ) -> Result<Vec<T>, WorthQueryApplicationReadExecutionDenial> {
        if !self.materialized() {
            return Ok(values.collect());
        }
        self.material_slots::<T>(capacity)?;
        let mut result = super::allocate_result_vector(capacity, subject)?;
        for value in values {
            if result.len() == capacity {
                return Err(super::result_buffer_denial(subject));
            }
            result.push(value);
        }
        Ok(result)
    }
    pub fn material_slots<T>(
        &self,
        count: usize,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        self.material
            .as_ref()
            .map_or(Ok(()), |storage| storage.slots::<T>(count))
    }
    pub fn material_next<T>(
        &self,
        values: &mut Vec<T>,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        if self.material.is_some() && values.len() == values.capacity() {
            self.material_slots::<T>(values.len().checked_add(1).ok_or_else(|| super::super::denial::relational_read_denial(worth_relational::facade::transactions::RelationalExecutionDenialCause::ChargedBytesOverflow, "managed source replacement"))?)?;
            values.try_reserve_exact(1).map_err(|_| super::super::denial::relational_read_denial(worth_relational::facade::transactions::RelationalExecutionDenialCause::AllocationFailed, "managed source tree"))?;
        }
        Ok(())
    }
    pub fn material_tree<T>(
        &self,
        len: usize,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        self.material
            .as_ref()
            .map_or(Ok(()), |storage| storage.tree::<T>(len))
    }
    pub fn material_field(
        &self,
        slot: &str,
        value: &worth_foundational::facade::AspectValue,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        self.material
            .as_ref()
            .map_or(Ok(()), |storage| storage.field(slot, value))
    }
    pub fn material_text(
        &self,
        text: &str,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        self.material
            .as_ref()
            .map_or(Ok(()), |storage| storage.text(text))
    }

    pub fn charge_projection(
        &mut self,
        record_count: usize,
        fields_per_record: usize,
        subject: &str,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        let field_count = record_count.saturating_mul(fields_per_record);
        self.charge_work(record_count.saturating_add(field_count), subject)?;
        self.projected_records = self.projected_records.saturating_add(record_count);
        self.projected_fields = self.projected_fields.saturating_add(field_count);
        Ok(())
    }

    pub fn charge_adjacency(
        &mut self,
        lists: usize,
        examined: usize,
        reserved: usize,
        subject: &str,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        self.charge_work(
            lists.saturating_add(examined).saturating_add(reserved),
            subject,
        )?;
        self.reads.record_adjacencies(lists, examined);
        Ok(())
    }

    pub fn charge_ordering_comparison(
        &mut self,
        subject: &str,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        self.charge_work(1, subject)?;
        self.ordering_comparisons = self.ordering_comparisons.saturating_add(1);
        Ok(())
    }

    pub fn charge_ordered_index_entries(
        &mut self,
        examined: usize,
        subject: &str,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        self.charge_work(examined, subject)?;
        self.reads.record_index_candidates(examined);
        self.ordered_index_entries_examined =
            self.ordered_index_entries_examined.saturating_add(examined);
        Ok(())
    }

    pub fn charge_source_observation(
        &mut self,
        units: usize,
        subject: &str,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        self.charge_work(units, subject)
    }

    pub fn charge_relation_predicate(
        &mut self,
        subject: &str,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        self.charge_work(2, subject)?;
        self.relation_predicate_work_units = self.relation_predicate_work_units.saturating_add(2);
        Ok(())
    }

    pub fn charge_traversal_work(
        &mut self,
        units: u64,
        subject: &str,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        if units == 0 {
            return self.checkpoint(subject);
        }
        let units = usize::try_from(units).map_err(|_| {
            super::super::denial::relational_read_denial(
                worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkCounterOverflow,
                subject,
            )
        })?;
        self.charge_work(units, subject)
    }

    fn charge_work(
        &mut self,
        units: usize,
        subject: &str,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        self.checkpoint(subject)?;

        if let Some(spent) = self.spent {
            spent.charge(units, subject)?;
        }
        self.work_units += units;
        Ok(())
    }

    pub fn checkpoint(
        &self,
        subject: &str,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        self.interruption.checkpoint(subject)
    }
}

impl Drop for ResultTreeWork<'_, '_, '_> {
    fn drop(&mut self) {
        if let Some(spent) = self.spent {
            spent.tree(self.work_units);
        }
    }
}
