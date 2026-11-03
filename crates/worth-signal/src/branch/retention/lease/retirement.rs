//! Nonwaiting retirement of the original external Signal obligation.
use super::SignalBranchRetentionLease;
use crate::branch::SignalBranchBasisDescriptor;
use worth_execution::{ExecutionRequest, WorkCeilingDenial};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalBranchRetentionRetirementDenial {
    Busy,
    Execution(WorkCeilingDenial),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalBranchRetentionRetirementProgress {
    Complete,
}
#[derive(Debug)]
#[must_use]
pub struct SignalBranchRetentionRetirement {
    original: SignalBranchRetentionLease,
    retired_descriptor: Option<SignalBranchBasisDescriptor>,
    retired_ledger: Option<super::super::registry::RetiredSignalExternalObligation>,
}
#[derive(Debug)]
#[must_use]
pub struct RetiredSignalBranchRetention {
    _original: SignalBranchRetentionRetirement,
}
impl SignalBranchRetentionLease {
    pub fn into_retirement(self) -> SignalBranchRetentionRetirement {
        SignalBranchRetentionRetirement {
            original: self,
            retired_descriptor: None,
            retired_ledger: None,
        }
    }
}
impl SignalBranchRetentionRetirement {
    pub fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<SignalBranchRetentionRetirementProgress, SignalBranchRetentionRetirementDenial>
    {
        use SignalBranchRetentionRetirementDenial as Denial;
        if self.retired_descriptor.is_some() {
            return Ok(SignalBranchRetentionRetirementProgress::Complete);
        }
        request.consult().map_err(Denial::Execution)?;
        self.retired_ledger = Some(
            self.original
                .binding
                .try_terminate_external(self.original.lease_id, request)?,
        );
        // Disarm Drop only after the genuine owner transaction. Keep its same
        // descriptor allocation and ledger share in the retired payload.
        self.retired_descriptor = self.original.descriptor.take();
        Ok(SignalBranchRetentionRetirementProgress::Complete)
    }
    pub fn into_retired(self) -> Result<RetiredSignalBranchRetention, Self> {
        if self.retired_descriptor.is_some() {
            Ok(RetiredSignalBranchRetention { _original: self })
        } else {
            Err(self)
        }
    }
}
impl RetiredSignalBranchRetention {
    pub fn reclaim(self) {
        drop(self);
    }
}
