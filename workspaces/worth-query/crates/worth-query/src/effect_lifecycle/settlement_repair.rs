use worth_relational::facade::history::RelationalCommitReceipt;
use worth_relational::facade::publication::PublicationSettlementRepairError;

use super::EffectExecutionAuthority;

#[derive(Debug)]
pub enum EffectSettlementRepairError {
    MissingRelationalAuthority,
    Settlement(PublicationSettlementRepairError),
}

impl std::fmt::Display for EffectSettlementRepairError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingRelationalAuthority => formatter.write_str(
                "effect settlement repair requires the owning Relational runtime authority",
            ),
            Self::Settlement(error) => {
                write!(formatter, "effect settlement repair failed: {error:?}")
            }
        }
    }
}

impl std::error::Error for EffectSettlementRepairError {}

pub(crate) fn repair_pending_effect_settlement(
    mut authority: EffectExecutionAuthority<'_>,
    commit_id: worth_relational::facade::history::CommitId,
) -> Result<RelationalCommitReceipt, EffectSettlementRepairError> {
    let runtime = authority
        .relational_runtime()
        .ok_or(EffectSettlementRepairError::MissingRelationalAuthority)?;
    runtime
        .repair_pending_publication_settlement(commit_id)
        .map_err(EffectSettlementRepairError::Settlement)
}
