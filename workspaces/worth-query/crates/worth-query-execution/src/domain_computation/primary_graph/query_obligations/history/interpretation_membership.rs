//! Local grant conservation while the same original acquire is unresolved.
use super::journal::QueryObligationDenial;
use worth_foundational::facade::ProgramSupportObligationIdentity;

pub(super) enum InterpretationMembership {
    Local,
    Pending,
    AbandonedPending,
    Durable(ProgramSupportObligationIdentity),
    Disposed,
}
impl InterpretationMembership {
    pub(super) fn begin(&mut self) -> Result<(), QueryObligationDenial> {
        match self {
            Self::Local => {
                *self = Self::Pending;
                Ok(())
            }
            Self::Pending | Self::AbandonedPending | Self::Durable(_) | Self::Disposed => {
                Err(QueryObligationDenial::ObligationSubjectMismatch)
            }
        }
    }
    pub(super) fn validate_pending(&self, local: usize) -> Result<(), QueryObligationDenial> {
        match self {
            Self::Pending | Self::AbandonedPending if local > 0 => Ok(()),
            Self::Local
            | Self::Pending
            | Self::AbandonedPending
            | Self::Durable(_)
            | Self::Disposed => Err(QueryObligationDenial::ObligationSubjectMismatch),
        }
    }
    pub(super) fn committed(
        &mut self,
        local: &mut usize,
        identity: ProgramSupportObligationIdentity,
    ) {
        // Called only under the owner lock after validation and authenticated fold.
        *local = local.checked_sub(1).expect("validated local transfer");
        *self = Self::Durable(identity);
    }
    pub(super) fn no_effect(&mut self, local: &mut usize) {
        match self {
            Self::Pending => *self = Self::Local,
            Self::AbandonedPending => {
                *local = local.checked_sub(1).expect("validated abandoned transfer");
                *self = Self::Disposed;
            }
            Self::Local | Self::Durable(_) | Self::Disposed => {
                unreachable!("validated pending transfer")
            }
        }
    }
    pub(super) fn abandon(&mut self, local: &mut usize) {
        match self {
            Self::Local => {
                *local = local.checked_sub(1).expect("issued local interpretation");
                *self = Self::Disposed;
            }
            Self::Pending => *self = Self::AbandonedPending,
            Self::Durable(_) => {} // Closing a cache is not durable logical retirement.
            Self::AbandonedPending | Self::Disposed => unreachable!("linear local token"),
        }
    }
}

#[cfg(test)]
mod tests;
