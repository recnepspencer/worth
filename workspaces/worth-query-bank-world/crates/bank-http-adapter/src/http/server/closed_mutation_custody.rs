//! Actual closed native carriers and collection release remain move-only.
use super::{
    closed_mutation_estate::ClosedMutationEstate, closed_mutation_source::ClosedMutationSource,
    estate_source_custody::EstateSourceRecord, mutation_custody::SourceMutationRecord,
};
use worth_query_host::facade::{
    application_entry::{WorthQueryClosedOperation, WorthQueryOperationCleanupReceipt},
    primary_graph::WorthQueryCleanupDispatchPosture as Posture,
};

fn discharged(
    receipt: Option<
        &worth_query_host::facade::primary_graph::WorthQueryApplicationManagedCleanupReceipt,
    >,
) -> bool {
    receipt.is_none_or(|receipt| match receipt.dispatch_posture() {
        Posture::NotDeclared | Posture::TerminalObserved => true,
        Posture::Pending
        | Posture::AttemptedUnresolved
        | Posture::Observed
        | Posture::DiagnosticCopy => false,
    })
}
pub(super) fn may_release(
    closed: &WorthQueryClosedOperation,
    collection: Option<&WorthQueryOperationCleanupReceipt>,
) -> bool {
    discharged(closed.managed_cleanup_receipt())
        && discharged(
            collection.and_then(WorthQueryOperationCleanupReceipt::managed_cleanup_receipt),
        )
}
enum ClosedMutationCustody {
    Source(ClosedMutationSource),
    Estate(ClosedMutationEstate),
}
/// The entire native completion remains with actual remaining delivery/recovery custody.
#[must_use = "retain original native closed evidence and remaining cleanup custody"]
pub struct BankHttpClosedMutationEvidence {
    custody: ClosedMutationCustody,
    collection: Option<WorthQueryOperationCleanupReceipt>,
}
impl BankHttpClosedMutationEvidence {
    /// Borrows native physical close evidence; it is no logical settlement proof.
    pub fn closed(&self) -> &WorthQueryClosedOperation {
        match &self.custody {
            ClosedMutationCustody::Source(owner) => owner.closed(),
            ClosedMutationCustody::Estate(owner) => owner.closed(),
        }
    }
    /// Lends the actual original cleanup owner for its native dispatch/recovery API.
    pub fn closed_mut(&mut self) -> &mut WorthQueryClosedOperation {
        match &mut self.custody {
            ClosedMutationCustody::Source(owner) => owner.closed_mut(),
            ClosedMutationCustody::Estate(owner) => owner.closed_mut(),
        }
    }
    pub fn released_collection(&self) -> Option<&WorthQueryOperationCleanupReceipt> {
        self.collection.as_ref()
    }
    pub fn released_collection_mut(&mut self) -> Option<&mut WorthQueryOperationCleanupReceipt> {
        self.collection.as_mut()
    }
}
/// Confirmed local transfer of a whole closed owner and its original installed runtime.
#[must_use = "closed physical evidence may still own dispatch or recovery"]
pub struct BankHttpClosedMutation {
    pub(super) evidence: BankHttpClosedMutationEvidence,
    pub(super) runtime: std::sync::Arc<dyn super::inbound_completion::BankRailCompletionRuntime>,
}
impl BankHttpClosedMutation {
    pub fn runtime(&self) -> &bank_server::BankIdentityRuntime {
        self.runtime.runtime()
    }
    pub fn evidence(&self) -> &BankHttpClosedMutationEvidence {
        &self.evidence
    }
    pub fn into_parts(
        self,
    ) -> (
        super::native_cleanup_custody::BankHttpCleanupRuntime,
        BankHttpClosedMutationEvidence,
    ) {
        (
            super::native_cleanup_custody::BankHttpCleanupRuntime::retain(self.runtime),
            self.evidence,
        )
    }
}

pub(super) struct ClosedMutationRecord {
    pub(super) owner: super::authenticated_owner::BankHttpAuthenticatedOwner,
    pub(super) sequence: u64,
    pub(super) evidence: BankHttpClosedMutationEvidence,
    pub(super) _permit: tokio::sync::OwnedSemaphorePermit,
}
impl ClosedMutationRecord {
    pub(super) fn from_source(record: SourceMutationRecord) -> Result<Self, SourceMutationRecord> {
        let SourceMutationRecord {
            owner,
            sequence,
            custody,
            released_cleanup,
            _permit,
        } = record;
        match custody.take_closed() {
            Ok(custody) => Ok(Self {
                owner,
                sequence,
                evidence: BankHttpClosedMutationEvidence {
                    custody: ClosedMutationCustody::Source(custody),
                    collection: released_cleanup,
                },
                _permit,
            }),
            Err(custody) => Err(SourceMutationRecord {
                owner,
                sequence,
                custody,
                released_cleanup,
                _permit,
            }),
        }
    }
    pub(super) fn from_estate(
        record: EstateSourceRecord,
        permit: tokio::sync::OwnedSemaphorePermit,
    ) -> Result<Self, (EstateSourceRecord, tokio::sync::OwnedSemaphorePermit)> {
        let EstateSourceRecord {
            owner,
            sequence,
            custody,
            released_cleanup,
            key,
            action,
            registry,
        } = record;
        match custody.take_closed() {
            Ok(custody) => Ok(Self {
                owner,
                sequence,
                evidence: BankHttpClosedMutationEvidence {
                    custody: ClosedMutationCustody::Estate(custody),
                    collection: released_cleanup,
                },
                _permit: permit,
            }),
            Err(custody) => Err((
                EstateSourceRecord {
                    owner,
                    sequence,
                    custody,
                    released_cleanup,
                    key,
                    action,
                    registry,
                },
                permit,
            )),
        }
    }
    pub(super) fn may_release(&self) -> bool {
        may_release(self.evidence.closed(), self.evidence.released_collection())
    }
}
