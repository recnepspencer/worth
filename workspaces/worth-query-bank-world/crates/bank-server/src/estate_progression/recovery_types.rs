//! Bank-owned recovery phases and descriptive outcomes.

#[derive(Debug)]
pub struct BankCommitRecoveryHandle {
    pub(super) query: worth_query_host::facade::primary_graph::WorthQueryRecoveryHandle,
}

impl BankCommitRecoveryHandle {
    pub(super) const fn query(
        &self,
    ) -> &worth_query_host::facade::primary_graph::WorthQueryRecoveryHandle {
        &self.query
    }

    pub fn installed_operation(&self) -> &str {
        self.query.binding().installed_aftermath_operation_slot()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankRecoveryDurability {
    StoreCapabilityRequired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankRecoverySupportTruth {
    DegradedRecoveryReport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankRecoveryPosture {
    Reversible,
    Compensatable,
    Reconcilable,
    Irreversible,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BankRecoveryInspection {
    durability: BankRecoveryDurability,
    support_truth: BankRecoverySupportTruth,
    posture: BankRecoveryPosture,
    work: crate::BankCommitCanonicalWorkEvidence,
}

impl BankRecoveryInspection {
    pub(super) fn from_query(
        view: worth_query_host::facade::primary_graph::WorthQueryRecoveryInspectionView,
    ) -> Self {
        use worth_query_host::facade::domain::PublishedAftermathPosture;
        let posture = match view.published_posture() {
            PublishedAftermathPosture::Reversible => BankRecoveryPosture::Reversible,
            PublishedAftermathPosture::Compensatable => BankRecoveryPosture::Compensatable,
            PublishedAftermathPosture::Reconcilable => BankRecoveryPosture::Reconcilable,
            PublishedAftermathPosture::Irreversible => BankRecoveryPosture::Irreversible,
        };
        Self {
            durability: BankRecoveryDurability::StoreCapabilityRequired,
            support_truth: BankRecoverySupportTruth::DegradedRecoveryReport,
            posture,
            work: crate::BankCommitCanonicalWorkEvidence::from_query(
                view.recovery_inspection_work(),
            ),
        }
    }

    pub const fn durability(&self) -> BankRecoveryDurability {
        self.durability
    }

    pub const fn support_truth(&self) -> BankRecoverySupportTruth {
        self.support_truth
    }

    pub const fn posture(&self) -> BankRecoveryPosture {
        self.posture
    }

    pub const fn canonical_work(&self) -> crate::BankCommitCanonicalWorkEvidence {
        self.work
    }

    pub const fn recovery_inspection_work(&self) -> crate::BankCommitCanonicalWorkEvidence {
        self.work
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BankRecoveryTransitionReceipt {
    installed_operation: String,
}

impl BankRecoveryTransitionReceipt {
    pub(super) fn new(installed_operation: impl Into<String>) -> Self {
        Self {
            installed_operation: installed_operation.into(),
        }
    }

    pub fn installed_operation(&self) -> &str {
        &self.installed_operation
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankRecoveryIdempotencyResolution {
    Unseen,
    AlreadyCommitted,
    IntentDrift,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankRecoveryClaimStatus {
    Unclaimed,
    Live,
    Consumed,
    Completed,
    Expired,
    Disposed,
    ForceTerminated,
}

impl BankRecoveryClaimStatus {
    pub(super) fn from_query(
        status: worth_query_host::facade::primary_graph::WorthQueryRecoveryClaimStatus,
    ) -> Self {
        use worth_query_host::facade::primary_graph::WorthQueryRecoveryClaimStatus as Query;
        match status {
            Query::Unclaimed => Self::Unclaimed,
            Query::Live => Self::Live,
            Query::Consumed => Self::Consumed,
            Query::Completed => Self::Completed,
            Query::Expired => Self::Expired,
            Query::Disposed => Self::Disposed,
            Query::ForceTerminated => Self::ForceTerminated,
        }
    }
}

#[derive(Debug)]
pub struct BankRecoverySafeRetryReceipt {
    external_completion: bool,
    fresh_attempt: bool,
    durability: BankRecoveryDurability,
}

#[derive(Debug)]
pub enum BankRecoverySafeRetryDenial {
    Retained {
        denial: super::BankEstateProgressionDenial,
        handle: Box<BankCommitRecoveryHandle>,
    },
    Terminal(super::BankEstateProgressionDenial),
}

impl BankRecoverySafeRetryDenial {
    pub(super) fn retained(
        denial: super::BankEstateProgressionDenial,
        handle: BankCommitRecoveryHandle,
    ) -> Self {
        Self::Retained {
            denial,
            handle: Box::new(handle),
        }
    }

    pub fn denial(&self) -> &super::BankEstateProgressionDenial {
        match self {
            Self::Retained { denial, .. } | Self::Terminal(denial) => denial,
        }
    }

    pub fn into_parts(
        self,
    ) -> (
        super::BankEstateProgressionDenial,
        Option<BankCommitRecoveryHandle>,
    ) {
        match self {
            Self::Retained { denial, handle } => (denial, Some(*handle)),
            Self::Terminal(denial) => (denial, None),
        }
    }
}

impl BankRecoverySafeRetryReceipt {
    pub(super) fn from_query(
        admission: worth_query_host::facade::primary_graph::WorthQueryRecoverySafeRetryAdmission,
    ) -> Self {
        let dispatch = admission.dispatch();
        let attempt = dispatch.causal_ladder().attempt();
        let fresh_attempt = attempt.predecessor().is_some_and(|predecessor| {
            predecessor.predecessor() == dispatch.causal_ladder().emission().identity()
        });
        Self {
            external_completion: dispatch.is_external_completion(),
            fresh_attempt,
            durability: BankRecoveryDurability::StoreCapabilityRequired,
        }
    }

    pub const fn is_external_completion(&self) -> bool {
        self.external_completion
    }

    pub const fn has_fresh_attempt(&self) -> bool {
        self.fresh_attempt
    }

    pub const fn durability(&self) -> BankRecoveryDurability {
        self.durability
    }
}

pub enum BankRecoveryExpiryEvaluation {
    Current,
    Expired(BankRecoveryExpiryDecision),
}

pub struct BankRecoveryExpiryDecision {
    pub(super) query: worth_query_host::facade::primary_graph::WorthQueryRecoveryExpiryDecision,
}

pub(super) fn map_idempotency(
    resolution: worth_query_host::facade::primary_graph::WorthQueryApplicationIdempotencyResolution,
) -> BankRecoveryIdempotencyResolution {
    use worth_query_host::facade::primary_graph::WorthQueryApplicationIdempotencyResolution;
    match resolution {
        WorthQueryApplicationIdempotencyResolution::Unseen => {
            BankRecoveryIdempotencyResolution::Unseen
        }
        WorthQueryApplicationIdempotencyResolution::AlreadyCommitted(_) => {
            BankRecoveryIdempotencyResolution::AlreadyCommitted
        }
        WorthQueryApplicationIdempotencyResolution::IntentDrift => {
            BankRecoveryIdempotencyResolution::IntentDrift
        }
    }
}

pub(super) fn map_expiry(
    evaluation: worth_query_host::facade::primary_graph::WorthQueryRecoveryExpiryEvaluation,
) -> BankRecoveryExpiryEvaluation {
    match evaluation {
        worth_query_host::facade::primary_graph::WorthQueryRecoveryExpiryEvaluation::Current(_) => {
            BankRecoveryExpiryEvaluation::Current
        }
        worth_query_host::facade::primary_graph::WorthQueryRecoveryExpiryEvaluation::Expired(
            query,
        ) => BankRecoveryExpiryEvaluation::Expired(BankRecoveryExpiryDecision { query }),
    }
}
