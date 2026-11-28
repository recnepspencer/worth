use std::sync::Arc;

use crate::adapter::TruthWritebackReceipt;
use crate::facade::{BridgeWritebackLoopDisposition, BridgeWritebackOutcomeClass, RuntimeBridge};
use crate::identity::{
    AsyncWritebackCommittedIdentityTag, AsyncWritebackNoopIdentityTag,
    AsyncWritebackRejectedIdentityTag, BridgeIdentity,
};

use super::receipt::{
    committed_receipt_identity, noop_receipt_identity, rejected_receipt_identity,
    BridgeAsyncWritebackNoopReceipt,
};
use super::{
    BridgeAsyncWritebackCausalityTransferReceipt, BridgeAsyncWritebackCounters,
    BridgeAsyncWritebackRejectedReceipt, StagedBridgeAsyncWritebackEffect,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeAsyncWritebackNoopClass {
    DuplicateCompletion,
    CanonicalNoop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeAsyncWritebackRejectedClass {
    LoopPreventionRejected,
    AuthorityRejected,
    CompletionOwnerMismatch,
    CompletionCommitUnavailable,
}

pub type BridgeAsyncCommittedWritebackIdentity = BridgeIdentity<AsyncWritebackCommittedIdentityTag>;
pub type BridgeAsyncNoopWritebackIdentity = BridgeIdentity<AsyncWritebackNoopIdentityTag>;
pub type BridgeAsyncRejectedWritebackIdentity = BridgeIdentity<AsyncWritebackRejectedIdentityTag>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeAsyncCommittedWriteback {
    committed_identity: BridgeAsyncCommittedWritebackIdentity,
    staged: StagedBridgeAsyncWritebackEffect,
    authority_receipt: TruthWritebackReceipt,
    causality_transfer: BridgeAsyncWritebackCausalityTransferReceipt,
    counters: BridgeAsyncWritebackCounters,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeAsyncNoopWriteback {
    noop_identity: BridgeAsyncNoopWritebackIdentity,
    staged: StagedBridgeAsyncWritebackEffect,
    noop_class: BridgeAsyncWritebackNoopClass,
    receipt_identity: BridgeAsyncWritebackNoopReceipt,
    authoritative_artifact_digest: Arc<str>,
    counters: BridgeAsyncWritebackCounters,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeAsyncWritebackRejectedWriteback {
    rejected_identity: BridgeAsyncRejectedWritebackIdentity,
    staged: StagedBridgeAsyncWritebackEffect,
    rejected_class: BridgeAsyncWritebackRejectedClass,
    receipt_identity: BridgeAsyncWritebackRejectedReceipt,
    detail: Arc<str>,
    counters: BridgeAsyncWritebackCounters,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeAsyncWritebackCommitReport {
    Committed(BridgeAsyncCommittedWriteback),
    Noop(BridgeAsyncNoopWriteback),
    Rejected(BridgeAsyncWritebackRejectedWriteback),
}

impl BridgeAsyncCommittedWriteback {
    fn new(
        staged: &StagedBridgeAsyncWritebackEffect,
        authority_receipt: TruthWritebackReceipt,
    ) -> Self {
        let committed_identity = BridgeAsyncCommittedWritebackIdentity::admit_bridge_owned(
            committed_receipt_identity(
                staged.admitted().completion().completion_identity(),
                authority_receipt.authoritative_artifact_digest(),
            )
            .as_str()
            .to_owned(),
        );
        let causality_transfer = BridgeAsyncWritebackCausalityTransferReceipt::committed(
            staged.admitted().family(),
            staged.admitted().completion().completion_identity(),
            staged
                .admitted()
                .request_identity()
                .request_identity()
                .as_str(),
            authority_receipt.authoritative_artifact_digest(),
            authority_receipt.request_digest(),
        );
        Self {
            committed_identity,
            staged: staged.clone(),
            authority_receipt,
            causality_transfer,
            counters: BridgeAsyncWritebackCounters::committed(),
        }
    }

    pub fn committed_identity(&self) -> &BridgeAsyncCommittedWritebackIdentity {
        &self.committed_identity
    }

    pub fn staged(&self) -> &StagedBridgeAsyncWritebackEffect {
        &self.staged
    }

    pub fn authority_receipt(&self) -> &TruthWritebackReceipt {
        &self.authority_receipt
    }

    pub fn causality_transfer(&self) -> &BridgeAsyncWritebackCausalityTransferReceipt {
        &self.causality_transfer
    }

    pub fn counters(&self) -> &BridgeAsyncWritebackCounters {
        &self.counters
    }
}

impl BridgeAsyncNoopWriteback {
    fn duplicate(
        staged: &StagedBridgeAsyncWritebackEffect,
        authoritative_artifact_digest: &Arc<str>,
    ) -> Self {
        Self {
            noop_identity: BridgeAsyncNoopWritebackIdentity::admit_bridge_owned(format!(
                "bridge-async-noop-writeback:{}",
                noop_receipt_identity(
                    staged.admitted().completion().completion_identity(),
                    BridgeAsyncWritebackNoopClass::DuplicateCompletion,
                )
                .as_str()
            )),
            staged: staged.clone(),
            noop_class: BridgeAsyncWritebackNoopClass::DuplicateCompletion,
            receipt_identity: noop_receipt_identity(
                staged.admitted().completion().completion_identity(),
                BridgeAsyncWritebackNoopClass::DuplicateCompletion,
            ),
            authoritative_artifact_digest: Arc::clone(authoritative_artifact_digest),
            counters: BridgeAsyncWritebackCounters::duplicate_noop(),
        }
    }

    fn canonical(
        staged: &StagedBridgeAsyncWritebackEffect,
        authoritative_artifact_digest: &str,
    ) -> Self {
        Self {
            noop_identity: BridgeAsyncNoopWritebackIdentity::admit_bridge_owned(format!(
                "bridge-async-noop-writeback:{}",
                noop_receipt_identity(
                    staged.admitted().completion().completion_identity(),
                    BridgeAsyncWritebackNoopClass::CanonicalNoop,
                )
                .as_str()
            )),
            staged: staged.clone(),
            noop_class: BridgeAsyncWritebackNoopClass::CanonicalNoop,
            receipt_identity: noop_receipt_identity(
                staged.admitted().completion().completion_identity(),
                BridgeAsyncWritebackNoopClass::CanonicalNoop,
            ),
            authoritative_artifact_digest: Arc::from(authoritative_artifact_digest.to_owned()),
            counters: BridgeAsyncWritebackCounters::canonical_noop(),
        }
    }

    pub fn noop_identity(&self) -> &BridgeAsyncNoopWritebackIdentity {
        &self.noop_identity
    }

    pub fn staged(&self) -> &StagedBridgeAsyncWritebackEffect {
        &self.staged
    }

    pub fn noop_class(&self) -> BridgeAsyncWritebackNoopClass {
        self.noop_class
    }

    pub fn receipt_identity(&self) -> &BridgeAsyncWritebackNoopReceipt {
        &self.receipt_identity
    }

    pub fn authoritative_artifact_digest(&self) -> &str {
        self.authoritative_artifact_digest.as_ref()
    }

    pub fn counters(&self) -> &BridgeAsyncWritebackCounters {
        &self.counters
    }
}

impl BridgeAsyncWritebackRejectedWriteback {
    fn new(
        staged: &StagedBridgeAsyncWritebackEffect,
        rejected_class: BridgeAsyncWritebackRejectedClass,
        detail: impl Into<Arc<str>>,
        counters: BridgeAsyncWritebackCounters,
    ) -> Self {
        Self {
            rejected_identity: BridgeAsyncRejectedWritebackIdentity::admit_bridge_owned(format!(
                "bridge-async-rejected-writeback:{}",
                rejected_receipt_identity(
                    staged.admitted().completion().completion_identity(),
                    rejected_class,
                )
                .as_str()
            )),
            staged: staged.clone(),
            rejected_class,
            receipt_identity: rejected_receipt_identity(
                staged.admitted().completion().completion_identity(),
                rejected_class,
            ),
            detail: detail.into(),
            counters,
        }
    }

    pub fn rejected_identity(&self) -> &BridgeAsyncRejectedWritebackIdentity {
        &self.rejected_identity
    }

    pub fn staged(&self) -> &StagedBridgeAsyncWritebackEffect {
        &self.staged
    }

    pub fn rejected_class(&self) -> BridgeAsyncWritebackRejectedClass {
        self.rejected_class
    }

    pub fn receipt_identity(&self) -> &BridgeAsyncWritebackRejectedReceipt {
        &self.receipt_identity
    }

    pub fn detail(&self) -> &str {
        self.detail.as_ref()
    }

    pub fn counters(&self) -> &BridgeAsyncWritebackCounters {
        &self.counters
    }
}

impl BridgeAsyncWritebackCommitReport {
    pub(crate) fn commit(
        runtime: &RuntimeBridge,
        staged: &StagedBridgeAsyncWritebackEffect,
    ) -> Self {
        let completion = staged.admitted().completion();
        if completion.request_identity().bridge_runtime_key() != runtime.signal_runtime_key {
            return Self::Rejected(BridgeAsyncWritebackRejectedWriteback::new(
                staged,
                BridgeAsyncWritebackRejectedClass::CompletionOwnerMismatch,
                "async completion belongs to a different Bridge runtime",
                BridgeAsyncWritebackCounters::rejected(),
            ));
        }
        let Some(mut retained_digest) = completion.writeback_receipt().try_reserve() else {
            return Self::Rejected(BridgeAsyncWritebackRejectedWriteback::new(
                staged,
                BridgeAsyncWritebackRejectedClass::CompletionCommitUnavailable,
                "async completion writeback could not reserve its receipt",
                BridgeAsyncWritebackCounters::rejected(),
            ));
        };
        if let Some(digest) = retained_digest.as_ref() {
            return Self::Noop(BridgeAsyncNoopWriteback::duplicate(staged, digest));
        }
        match staged.loop_prevention().disposition() {
            BridgeWritebackLoopDisposition::CanonicalNoop => {
                return Self::Noop(BridgeAsyncNoopWriteback::canonical(
                    staged,
                    staged.effect().digest(),
                ));
            }
            BridgeWritebackLoopDisposition::RejectAsUnsafeFeedback => {
                return Self::Rejected(BridgeAsyncWritebackRejectedWriteback::new(
                    staged,
                    BridgeAsyncWritebackRejectedClass::LoopPreventionRejected,
                    format!(
                        "bridge async writeback rejected completion `{}` before authority because loop prevention classified {:?}",
                        staged.admitted().completion().completion_identity(),
                        staged.loop_prevention().disposition()
                    ),
                    BridgeAsyncWritebackCounters::rejected(),
                ));
            }
            BridgeWritebackLoopDisposition::AllowAuthoritativeAttempt => {}
        }
        match runtime.execute_writeback_authority(
            staged.writeback_contract(),
            staged.effect(),
            staged.idempotence(),
        ) {
            Ok((outcome, receipt)) => match outcome.outcome_class() {
                BridgeWritebackOutcomeClass::AuthoritativeCommit => {
                    *retained_digest = Some(receipt.share_authoritative_artifact_digest());
                    Self::Committed(BridgeAsyncCommittedWriteback::new(staged, receipt))
                }
                BridgeWritebackOutcomeClass::CanonicalNoop => {
                    Self::Noop(BridgeAsyncNoopWriteback::canonical(
                        staged,
                        outcome.authoritative_artifact_digest(),
                    ))
                }
                BridgeWritebackOutcomeClass::Rejected => unreachable!(
                    "bridge writeback authority converts rejected receipts into typed errors before async writeback outcome lowering"
                ),
            },
            Err(error) => Self::Rejected(BridgeAsyncWritebackRejectedWriteback::new(
                staged,
                BridgeAsyncWritebackRejectedClass::AuthorityRejected,
                error.to_string(),
                BridgeAsyncWritebackCounters::authority_rejected(),
            )),
        }
    }

    pub fn committed(&self) -> Option<&BridgeAsyncCommittedWriteback> {
        match self {
            Self::Committed(value) => Some(value),
            _ => None,
        }
    }

    pub fn noop(&self) -> Option<&BridgeAsyncNoopWriteback> {
        match self {
            Self::Noop(value) => Some(value),
            _ => None,
        }
    }

    pub fn rejected(&self) -> Option<&BridgeAsyncWritebackRejectedWriteback> {
        match self {
            Self::Rejected(value) => Some(value),
            _ => None,
        }
    }
}
