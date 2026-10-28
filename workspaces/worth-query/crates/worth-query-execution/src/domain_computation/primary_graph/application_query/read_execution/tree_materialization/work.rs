use super::super::OneShotReadWorkObservation;
use super::WorthQueryApplicationReadExecutionDenial;
use crate::domain_computation::primary_graph::read_observation::WorthQueryInvariantProjectionWork;

pub(super) struct ResultTreeWork<'a, 'request, 'authority> {
    material: Option<super::material_storage::MaterialTreeStorage<'request, 'authority>>,
    interruption: super::super::ReadInterruption<'a>,

    pub(super) projected_records: usize,
    pub(super) projected_fields: usize,
    pub(super) reads: WorthQueryInvariantProjectionWork,
    pub(super) ordered_index_entries_examined: usize,
    pub(super) relation_predicate_work_units: usize,
    pub(super) ordering_comparisons: usize,
    pub(super) work_units: usize,
    spent: Option<&'a OneShotReadWorkObservation<'a>>,
}

impl<'a, 'request, 'authority> ResultTreeWork<'a, 'request, 'authority> {
    pub(super) fn new(
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

    pub(super) fn materialized(&self) -> bool {
        self.material.is_some()
    }
    pub(super) fn collect_material<T>(
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
    pub(super) fn material_slots<T>(
        &self,
        count: usize,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        self.material
            .as_ref()
            .map_or(Ok(()), |storage| storage.slots::<T>(count))
    }
    pub(super) fn material_next<T>(
        &self,
        values: &mut Vec<T>,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        if self.material.is_some() && values.len() == values.capacity() {
            self.material_slots::<T>(values.len().checked_add(1).ok_or_else(|| super::super::denial::relational_read_denial(worth_relational::facade::transactions::RelationalExecutionDenialCause::ChargedBytesOverflow, "managed source replacement"))?)?;
            values.try_reserve_exact(1).map_err(|_| super::super::denial::relational_read_denial(worth_relational::facade::transactions::RelationalExecutionDenialCause::AllocationFailed, "managed source tree"))?;
        }
        Ok(())
    }
    pub(super) fn material_tree<T>(
        &self,
        len: usize,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        self.material
            .as_ref()
            .map_or(Ok(()), |storage| storage.tree::<T>(len))
    }
    pub(super) fn material_field(
        &self,
        slot: &str,
        value: &worth_foundational::facade::AspectValue,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        self.material
            .as_ref()
            .map_or(Ok(()), |storage| storage.field(slot, value))
    }
    pub(super) fn material_text(
        &self,
        text: &str,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        self.material
            .as_ref()
            .map_or(Ok(()), |storage| storage.text(text))
    }

    pub(super) fn charge_projection(
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

    pub(super) fn charge_adjacency(
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

    pub(super) fn charge_ordering_comparison(
        &mut self,
        subject: &str,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        self.charge_work(1, subject)?;
        self.ordering_comparisons = self.ordering_comparisons.saturating_add(1);
        Ok(())
    }

    pub(super) fn charge_ordered_index_entries(
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

    pub(super) fn charge_source_observation(
        &mut self,
        units: usize,
        subject: &str,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        self.charge_work(units, subject)
    }

    pub(super) fn charge_relation_predicate(
        &mut self,
        subject: &str,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        self.charge_work(2, subject)?;
        self.relation_predicate_work_units = self.relation_predicate_work_units.saturating_add(2);
        Ok(())
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

    pub(super) fn checkpoint(
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
