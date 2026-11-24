//! Home recovery refusal carries the original preparation or the runtime outcome.
use super::{PreparedRelationalHomeRecovery, RelationalHomeRecoveryPreparationDenial};
use crate::durability::authority::recovery::home_program_transition::RelationalHomeProgramTransitionSourceRefusal;
use crate::durability::data::RelationalRecoveryStop;

/// Failures before reconstruction return this exact opaque preparation.
/// Reconstruction failures consume that attempt, just like ordinary recovery;
/// the Store source owner retains its independent original source for retry.
#[derive(Debug)]
pub enum RefusedPreparedRelationalHomeRecovery {
    Preparation {
        prepared: PreparedRelationalHomeRecovery,
        cause: RelationalHomeRecoveryPreparationDenial,
    },
    Recovery(RelationalRecoveryStop),
    Transition(RelationalHomeProgramTransitionSourceRefusal),
}
