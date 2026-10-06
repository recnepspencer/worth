use super::super::{
    read_execution_denial, OneShotReadWorkObservation, WorthQueryApplicationReadExecutionDenial,
    WorthQueryApplicationReadExecutionDenialKind,
};

pub(super) struct RootSelectionWork<'a> {
    pub(super) request:
        worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    pub(super) maximum_work: usize,
    pub(super) work_units: usize,
    pub(super) adjacency_lists_read: usize,
    pub(super) relation_records_examined: usize,
    pub(super) predicate_records_examined: usize,
    pub(super) predicate_work_units: usize,
    pub(super) spent: Option<&'a OneShotReadWorkObservation>,
}

impl<'a> RootSelectionWork<'a> {
    pub(super) fn new(
        maximum_work: usize,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        spent: Option<&'a OneShotReadWorkObservation>,
    ) -> Self {
        Self {
            request: request.clone(),
            maximum_work,
            work_units: 0,
            adjacency_lists_read: 0,
            relation_records_examined: 0,
            predicate_records_examined: 0,
            predicate_work_units: 0,
            spent,
        }
    }

    pub(super) fn remaining(&self) -> usize {
        self.maximum_work.saturating_sub(self.work_units)
    }

    pub(super) fn checkpoint(
        &self,
        subject: &str,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        super::super::interruption::checkpoint(&self.request, subject)
    }

    pub(super) fn charge(
        &mut self,
        adjacency_lists_read: usize,
        relation_records_examined: usize,
        endpoint_records_reserved: usize,
        subject: &str,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        let charged = adjacency_lists_read
            .saturating_add(relation_records_examined)
            .saturating_add(endpoint_records_reserved);
        self.checkpoint(subject)?;
        if self.work_units.saturating_add(charged) > self.maximum_work {
            return Err(read_execution_denial(
                WorthQueryApplicationReadExecutionDenialKind::WorkLimitExceeded,
                subject,
            ));
        }
        self.work_units = self.work_units.saturating_add(charged);
        self.adjacency_lists_read = self
            .adjacency_lists_read
            .saturating_add(adjacency_lists_read);
        self.relation_records_examined = self
            .relation_records_examined
            .saturating_add(relation_records_examined);
        Ok(())
    }

    pub(super) fn charge_predicate(
        &mut self,
        records_examined: usize,
        matches_reserved: usize,
        subject: &str,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        let charged = records_examined.saturating_add(matches_reserved);
        self.checkpoint(subject)?;
        if self.work_units.saturating_add(charged) > self.maximum_work {
            return Err(read_execution_denial(
                WorthQueryApplicationReadExecutionDenialKind::WorkLimitExceeded,
                subject,
            ));
        }
        self.work_units = self.work_units.saturating_add(charged);
        self.predicate_records_examined = self
            .predicate_records_examined
            .saturating_add(records_examined);
        self.predicate_work_units = self.predicate_work_units.saturating_add(charged);
        Ok(())
    }

    pub(super) fn charge_source_observation(
        &mut self,
        subject: &str,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        self.checkpoint(subject)?;
        if self.work_units >= self.maximum_work {
            return Err(read_execution_denial(
                WorthQueryApplicationReadExecutionDenialKind::WorkLimitExceeded,
                subject,
            ));
        }
        self.work_units += 1;
        Ok(())
    }

    pub(super) fn charge_source_copy(
        &mut self,
        units: usize,
        subject: &str,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        self.checkpoint(subject)?;
        if self.work_units.saturating_add(units) > self.maximum_work {
            return Err(read_execution_denial(
                WorthQueryApplicationReadExecutionDenialKind::WorkLimitExceeded,
                subject,
            ));
        }
        self.work_units += units;
        Ok(())
    }
}

impl Drop for RootSelectionWork<'_> {
    fn drop(&mut self) {
        if let Some(spent) = self.spent {
            spent.root(self.work_units);
        }
    }
}
