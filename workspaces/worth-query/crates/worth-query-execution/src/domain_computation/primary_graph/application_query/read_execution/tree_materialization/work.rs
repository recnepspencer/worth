use super::{
    read_execution_denial, WorthQueryApplicationReadExecutionDenial,
    WorthQueryApplicationReadExecutionDenialKind,
};

pub(super) struct ResultTreeWork {
    maximum_work: usize,
    pub(super) projected_records: usize,
    pub(super) projected_fields: usize,
    pub(super) adjacency_lists_read: usize,
    pub(super) relation_records_examined: usize,
    pub(super) ordered_index_entries_examined: usize,
    pub(super) relation_predicate_work_units: usize,
    pub(super) ordering_comparisons: usize,
    pub(super) work_units: usize,
}

fn work_limit_denial(subject: impl Into<String>) -> WorthQueryApplicationReadExecutionDenial {
    read_execution_denial(
        WorthQueryApplicationReadExecutionDenialKind::WorkLimitExceeded,
        subject,
    )
}

impl ResultTreeWork {
    pub(super) fn new(maximum_work: usize) -> Self {
        Self {
            maximum_work,
            projected_records: 0,
            projected_fields: 0,
            adjacency_lists_read: 0,
            relation_records_examined: 0,
            ordered_index_entries_examined: 0,
            relation_predicate_work_units: 0,
            ordering_comparisons: 0,
            work_units: 0,
        }
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
        self.adjacency_lists_read = self.adjacency_lists_read.saturating_add(lists);
        self.relation_records_examined = self.relation_records_examined.saturating_add(examined);
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
        if self.work_units.saturating_add(units) > self.maximum_work {
            return Err(work_limit_denial(subject));
        }
        self.work_units += units;
        Ok(())
    }

    pub(super) fn remaining_work(&self) -> usize {
        self.maximum_work.saturating_sub(self.work_units)
    }
}
