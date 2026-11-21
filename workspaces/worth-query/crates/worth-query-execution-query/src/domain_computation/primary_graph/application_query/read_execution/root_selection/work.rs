use super::super::{OneShotReadWorkObservation, WorthQueryApplicationReadExecutionDenial};
use crate::domain_computation::primary_graph::read_observation::WorthQueryInvariantProjectionWork;

pub struct RootSelectionWork<'a> {
    pub interruption: super::super::ReadInterruption<'a>,

    pub work_units: usize,
    pub reads: WorthQueryInvariantProjectionWork,
    pub predicate_records_examined: usize,
    pub predicate_work_units: usize,
    pub spent: Option<&'a OneShotReadWorkObservation<'a>>,
    pub steps:
        Vec<super::super::super::access_receipt::WorthQueryApplicationQueryRootPathStepRead>,
    pub step_limit: usize,
    pub step_memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl<'a> RootSelectionWork<'a> {
    pub fn new(
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
        }
    }

    pub fn checkpoint(
        &self,
        subject: &str,
    ) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
        self.interruption.checkpoint(subject)
    }

    pub fn charge(
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

    pub fn charge_predicate(
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

    pub fn charge_source_observation(
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

    pub fn charge_source_copy(
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
