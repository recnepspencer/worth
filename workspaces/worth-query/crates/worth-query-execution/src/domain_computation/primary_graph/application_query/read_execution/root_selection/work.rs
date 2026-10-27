use super::super::{OneShotReadWorkObservation, WorthQueryApplicationReadExecutionDenial};
use crate::domain_computation::primary_graph::read_observation::WorthQueryInvariantProjectionWork;

pub(super) struct RootSelectionWork<'a> {
    pub(super) interruption: super::super::ReadInterruption<'a>,

    pub(super) work_units: usize,
    pub(super) reads: WorthQueryInvariantProjectionWork,
    pub(super) predicate_records_examined: usize,
    pub(super) predicate_work_units: usize,
    pub(super) spent: Option<&'a OneShotReadWorkObservation<'a>>,
    pub(super) steps:
        Vec<super::super::super::access_receipt::WorthQueryApplicationQueryRootPathStepRead>,
    pub(super) step_limit: usize,
    pub(super) step_memory: Option<worth_execution::ExecutionMemoryReservation>,
    pub(super) source_allocation:
        Option<std::sync::Arc<super::super::super::observed_source::RootPathAllocation>>,
}

impl<'a> RootSelectionWork<'a> {
    pub(super) fn new(
        interruption: super::super::ReadInterruption<'a>,
        spent: Option<&'a OneShotReadWorkObservation<'a>>,
    ) -> Self {
        Self {
            interruption,

            work_units: 0,
            reads: WorthQueryInvariantProjectionWork::default(),
            predicate_records_examined: 0,
            predicate_work_units: 0,
            spent,
            steps: Vec::new(),
            step_memory: None,
            step_limit: 0,
            source_allocation: None,
        }
    }

    pub(super) fn checkpoint(
        &self,
        subject: &str,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        self.interruption.checkpoint(subject)
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

        if let Some(spent) = self.spent {
            spent.charge(charged, subject)?;
        }
        self.work_units = self.work_units.saturating_add(charged);
        self.reads
            .record_adjacencies(adjacency_lists_read, relation_records_examined);
        Ok(())
    }

    pub(super) fn charge_predicate(
        &mut self,
        records_examined: usize,
        matches_reserved: usize,
        field_reads: usize,
        subject: &str,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        let charged = records_examined.saturating_add(matches_reserved);
        self.checkpoint(subject)?;

        if let Some(spent) = self.spent {
            spent.charge(charged, subject)?;
        }
        self.work_units = self.work_units.saturating_add(charged);
        self.reads.record_fields(field_reads);
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

        if let Some(spent) = self.spent {
            spent.charge(1, subject)?;
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

        if let Some(spent) = self.spent {
            spent.charge(units, subject)?;
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
