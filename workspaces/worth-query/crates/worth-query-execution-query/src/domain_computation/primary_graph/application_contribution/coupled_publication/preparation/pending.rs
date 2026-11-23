//! Demands the prefix or a stage owes, prepared once their barrier allows.
use super::*;

pub(super) enum PendingOutputDemand<'a> {
    Prefix(&'a DeclaredCoupledOutputDemand),
    Stage(DeclaredCoupledOutputDemand, usize),
}
impl PendingOutputDemand<'_> {
    pub(super) fn demand(&self) -> &DeclaredCoupledOutputDemand {
        match self {
            Self::Prefix(demand) => demand,
            Self::Stage(demand, _) => demand,
        }
    }
    /// The applied position of the stage whose effect owes this demand.
    pub(super) fn owed(&self) -> usize {
        match self {
            Self::Prefix(_) => 0,
            Self::Stage(_, owed) => *owed,
        }
    }
}

impl<S: ApplicationSchema> CoupledApplicationAssembly<'_, '_, '_, S> {
    pub(super) fn prepare_pending(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        before: &worth_relational::facade::runtime::RelationalPinnedRead,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
        pending: &mut Vec<PendingOutputDemand<'_>>,
        allowed: Option<&[String]>,
    ) -> Result<(), CoupledSourceReadStop> {
        let mut position = 0;
        while position < pending.len() {
            if allowed.is_none_or(|families| {
                families
                    .iter()
                    .any(|family| family == pending[position].demand().family)
            }) {
                let demand = pending.remove(position);
                self.owe_at(demand.owed());
                self.prepare_one(execution, before, demand.demand(), candidate)?;
            } else {
                position += 1;
            }
        }
        Ok(())
    }
}
