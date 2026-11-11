//! One invoking request funds the actual index decode and its retained owners.
use super::funding::{IndexArray, IndexArrayBuilder, IndexCheckpointWork};
use crate::durability::data::RelationalRecoveryStop as Stop;
use worth_execution::ExecutionMemoryReservation;

#[derive(Default)]
pub(super) struct IndexDecodeBudget<'w> {
    work: Option<&'w IndexCheckpointWork<'w>>,
    funds: Option<IndexArrayBuilder<'w, Option<ExecutionMemoryReservation>>>,
}
impl<'w> IndexDecodeBudget<'w> {
    pub(super) fn new(work: &'w IndexCheckpointWork<'w>, count: usize) -> Result<Self, Stop> {
        let slots = count
            .checked_mul(2)
            .and_then(|n| n.checked_add(3))
            .ok_or_else(overflow)?;
        Ok(Self {
            work: Some(work),
            funds: Some(work.array(slots)?),
        })
    }
    pub(super) fn transient(
        &self,
        bytes: usize,
    ) -> Result<Option<ExecutionMemoryReservation>, Stop> {
        self.work
            .map(|work| work.reserve(bytes as u64))
            .transpose()
            .map(Option::flatten)
    }
    pub(super) fn consult(&self) -> Result<(), Stop> {
        self.work.map_or(Ok(()), IndexCheckpointWork::consult)
    }
    pub(super) fn work(&self) -> Option<&IndexCheckpointWork<'_>> {
        self.work
    }
    pub(super) fn reserve(&mut self, bytes: u64) -> Result<(), Stop> {
        if let Some(work) = self.work {
            work.consult()?;
            let memory = work.reserve(bytes)?;
            self.funds
                .as_mut()
                .expect("funded decode retains its fixed funding roster")
                .push(memory)?;
        }
        Ok(())
    }
    pub(super) fn finish(self) -> Result<IndexArray<Option<ExecutionMemoryReservation>>, Stop> {
        let mut funds = self.funds.expect("only funded decode exports funded DATA");
        while funds.len() < funds.element_count() {
            funds.push(None)?;
        }
        funds.seal()
    }
}
pub(super) fn overflow() -> Stop {
    Stop::Request(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
}
pub(super) fn wire_stop(
    cause: crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal,
) -> Stop {
    use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactRefusal as Refusal;
    match cause {
        Refusal::Request(cause) => Stop::Request(cause),
        cause => {
            super::corrupt(&format!("index checkpoint decode quote refused: {cause:?}")).into()
        }
    }
}
