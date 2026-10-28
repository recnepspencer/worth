use std::sync::Arc;

use crate::branch::{RelationalBranchBasisDescriptor, RelationalBranchIdentity};
use crate::mvcc::RelationalBranchObservation;

use super::{RelationalBasisRetentionReason, RelationalExternalRetentionTerminalAccounting};

/// Explicit external owner obligation for one admitted component basis.
///
/// This is separate from cloning an admitted basis. It is owner-issued and
/// must be consumed by release or dropped; it exposes no read or mutation
/// capability itself.
#[derive(Debug)]
pub struct RelationalBranchRetentionLease {
    observation: Option<RelationalBranchObservation>,
    obligation: Option<super::RelationalExternalBasisRetentionObligation>,
    terminal_accounting: Arc<RelationalExternalRetentionTerminalAccounting>,
}

impl RelationalBranchRetentionLease {
    pub(crate) fn new(
        observation: RelationalBranchObservation,
        binding: &super::RelationalBranchRetentionBinding,
        terminal_accounting: Arc<RelationalExternalRetentionTerminalAccounting>,
    ) -> Result<Self, super::RelationalRetentionAcquisitionDenial> {
        let obligation = super::RelationalExternalBasisRetentionObligation::acquire(
            binding,
            Arc::clone(observation.selected_root()),
        )?;
        Ok(Self {
            observation: Some(observation),
            obligation: Some(obligation),
            terminal_accounting,
        })
    }

    pub fn identity(&self) -> &RelationalBranchIdentity {
        self.observation
            .as_ref()
            .expect(/* Proof: The private lease constructor installs observation and obligation together; consuming release takes each once, and Drop checks the disarmed state. */ "live retention lease carries its observation")
            .identity()
    }

    pub fn descriptor(&self) -> &RelationalBranchBasisDescriptor {
        self.observation
            .as_ref()
            .expect(/* Proof: The private lease constructor installs observation and obligation together; consuming release takes each once, and Drop checks the disarmed state. */ "live retention lease carries its observation")
            .descriptor()
    }

    pub const fn retention_reason(&self) -> RelationalBasisRetentionReason {
        RelationalBasisRetentionReason::ExternalComponentBasis
    }

    pub fn release(mut self) -> RelationalBranchRetentionReleaseReceipt {
        let observation = self
            .observation
            .take()
            .expect(/* Proof: The private lease constructor installs observation and obligation together; consuming release takes each once, and Drop checks the disarmed state. */ "retention lease can be released only once");
        let outcome = self
            .obligation
            .take()
            .expect(/* Proof: The private lease constructor installs observation and obligation together; consuming release takes each once, and Drop checks the disarmed state. */ "live retention lease carries its owner obligation")
            .release();
        self.terminal_accounting.record_explicit_release();
        RelationalBranchRetentionReleaseReceipt {
            descriptor: observation.descriptor().clone(),
            outcome,
        }
    }

    pub(crate) fn owner_relationship(
        &self,
        binding: &super::RelationalBranchRetentionBinding,
    ) -> super::RelationalRetentionOwnerRelationship {
        self.obligation
            .as_ref()
            .map(|obligation| obligation.owner_relationship(binding))
            .unwrap_or(super::RelationalRetentionOwnerRelationship::OwnerUnavailable)
    }

    pub(crate) fn admitted_basis(&self) -> crate::branch::AdmittedRelationalBranchBasis {
        self.observation
            .as_ref()
            .expect(/* Proof: The private lease constructor installs observation and obligation together; consuming release takes each once, and Drop checks the disarmed state. */ "live retention lease carries its observation")
            .admitted_basis()
    }
}

impl Drop for RelationalBranchRetentionLease {
    fn drop(&mut self) {
        if self.observation.take().is_some() {
            self.obligation.take();
            self.terminal_accounting.record_dropped_release();
        }
    }
}

/// A denied release that preserves the still-live external obligation.
#[derive(Debug)]
pub struct RelationalBranchRetentionReleaseDenial {
    denial: crate::branch::RelationalBranchBasisDenial,
    lease: RelationalBranchRetentionLease,
}

impl RelationalBranchRetentionReleaseDenial {
    pub(crate) fn new(
        denial: crate::branch::RelationalBranchBasisDenial,
        lease: RelationalBranchRetentionLease,
    ) -> Self {
        Self { denial, lease }
    }

    pub fn denial(&self) -> &crate::branch::RelationalBranchBasisDenial {
        &self.denial
    }

    pub fn lease(&self) -> &RelationalBranchRetentionLease {
        &self.lease
    }

    pub fn into_lease(self) -> RelationalBranchRetentionLease {
        self.lease
    }
}

/// Evidence that one explicit external retention obligation was consumed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalBranchRetentionTerminalOutcome {
    Released,
    OwnerUnavailable,
}

/// Evidence that one explicit external retention obligation reached one
/// terminal path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalBranchRetentionReleaseReceipt {
    descriptor: RelationalBranchBasisDescriptor,
    outcome: RelationalBranchRetentionTerminalOutcome,
}

impl RelationalBranchRetentionReleaseReceipt {
    pub fn descriptor(&self) -> &RelationalBranchBasisDescriptor {
        &self.descriptor
    }

    pub const fn outcome(&self) -> RelationalBranchRetentionTerminalOutcome {
        self.outcome
    }
}
