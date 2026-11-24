//! Type-erased handles onto existing `Arc` allocations carried by
//! `PerformedRelationalCommit` so rustdoc auto-trait synthesis stays bounded.

use std::any::Any;
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;

use worth_relational_branch::internal_port::history::data::positioned_canonical_commit::PositionedCanonicalCommit;

use crate::runtime::state::runtime_state::publication_settlement_registry::record::PendingRelationalPublicationSettlement;

trait PositionedCommitBacking: Send + Sync {
    fn as_positioned(&self) -> &PositionedCanonicalCommit;
    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
}

impl PositionedCommitBacking for PositionedCanonicalCommit {
    fn as_positioned(&self) -> &PositionedCanonicalCommit {
        self
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }
}

pub(super) struct SharedPositionedCommit(Arc<dyn PositionedCommitBacking>);

impl SharedPositionedCommit {
    pub(super) fn from_arc(commit: Arc<PositionedCanonicalCommit>) -> Self {
        Self(commit)
    }

    pub(super) fn into_arc(self) -> Arc<PositionedCanonicalCommit> {
        self.0
            .into_any()
            .downcast::<PositionedCanonicalCommit>()
            .expect("SharedPositionedCommit always wraps PositionedCanonicalCommit")
    }
}

impl Deref for SharedPositionedCommit {
    type Target = PositionedCanonicalCommit;

    fn deref(&self) -> &Self::Target {
        PositionedCommitBacking::as_positioned(&*self.0)
    }
}

impl fmt::Debug for SharedPositionedCommit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}

trait PendingSettlementBacking: Send + Sync {
    fn as_record(&self) -> &PendingRelationalPublicationSettlement;
    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
}

impl PendingSettlementBacking for PendingRelationalPublicationSettlement {
    fn as_record(&self) -> &PendingRelationalPublicationSettlement {
        self
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }
}

pub(super) struct SharedPendingSettlement(Arc<dyn PendingSettlementBacking>);

impl SharedPendingSettlement {
    pub(super) fn from_arc(record: Arc<PendingRelationalPublicationSettlement>) -> Self {
        Self(record)
    }

    pub(super) fn to_arc(&self) -> Arc<PendingRelationalPublicationSettlement> {
        Arc::clone(&self.0)
            .into_any()
            .downcast::<PendingRelationalPublicationSettlement>()
            .expect("SharedPendingSettlement always wraps PendingRelationalPublicationSettlement")
    }
}

impl Deref for SharedPendingSettlement {
    type Target = PendingRelationalPublicationSettlement;

    fn deref(&self) -> &Self::Target {
        PendingSettlementBacking::as_record(&*self.0)
    }
}
