//! Keep original evidence indexes while their live publication custody retires.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryHomeRetirementDenial as Denial, WorthQueryHomeRetirementOwner as Owner,
    WorthQueryOperationHomeRetirementProgress as Progress,
};
use worth_execution::ExecutionRequest;
use worth_relational::facade::runtime::RelationalRuntime;
impl WorthQueryCompletedCommitEvidenceStore {
    pub(in crate::domain_computation::primary_graph::provider) fn preflight_home_retirement(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        if !self.fresh_history.is_empty() {
            return Err(Denial::Busy(Owner::PublicationReceipts));
        }
        for evidence in self.by_commit.values_mut() {
            request.consult().map_err(Denial::Execution)?;
            evidence
                .publication_for_home_retirement()
                .preflight_home_retirement(request)?;
        }
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph::provider) fn advance_home_retirement(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<Progress, Denial> {
        use std::ops::Bound::{Excluded, Unbounded};
        request.consult().map_err(Denial::Execution)?;
        let next = match self.home_retired_through {
            Some(after) => self
                .by_commit
                .range_mut((Excluded(after), Unbounded))
                .next(),
            None => self.by_commit.iter_mut().next(),
        };
        let Some((commit, evidence)) = next else {
            return Ok(Progress::Complete);
        };
        if evidence
            .publication_for_home_retirement()
            .advance_home_retirement(request, native)?
            == Progress::Complete
        {
            self.home_retired_through = Some(*commit);
        }
        Ok(Progress::Advanced)
    }
}
