use worth_store_operations::OperationalControlRecordKind as Record;

// These publication states remain part of the cold S10 model, but their
// deleted C7/S10 owner records are intentionally not constructible from the
// current production record vocabulary.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum OperationalRecoveryActionBinding {
    None,
    Authorization {
        plan: [u8; 32],
        execution: Option<[u8; 32]>,
        replayed: bool,
    },
    PublicationPrepared(PublicationBinding),
    PublicationPending(PublicationBinding),
    PublicationDisposition {
        publication: [u8; 32],
        observed_authority: [u8; 32],
    },
    FenceReleased {
        publication: [u8; 32],
        fence: [u8; 32],
        fence_plan: [u8; 32],
    },
    BootstrapTransfer {
        authorization_plan: [u8; 32],
        execution_plan: [u8; 32],
        receipt: [u8; 32],
        source_lease: [u8; 32],
        target: [u8; 32],
    },
    BootstrapCompleted {
        receipt: [u8; 32],
        source_lease: [u8; 32],
        verification: [u8; 32],
    },
    PromotionFence {
        authorization_plan: [u8; 32],
        execution_plan: [u8; 32],
        fence: [u8; 32],
        epoch: u64,
    },
    PromotionRecorded {
        authorization_plan: [u8; 32],
        execution_plan: [u8; 32],
        receipt: [u8; 32],
        fence: [u8; 32],
        epoch: u64,
    },
    PromotionPublished {
        receipt: [u8; 32],
        publication: [u8; 32],
        verification: [u8; 32],
        target: [u8; 32],
        epoch: u64,
    },
    PromotionReadmitted {
        publication: [u8; 32],
        serve_lease: [u8; 32],
        epoch: u64,
    },
    RejoinPlanned {
        promotion_receipt: [u8; 32],
        plan: [u8; 32],
        disposition: u8,
    },
    RejoinCompleted {
        plan: [u8; 32],
        receipt: [u8; 32],
        forensic_retention: [u8; 32],
        rebootstrap_target: [u8; 32],
        disposition: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct PublicationBinding {
    publication: [u8; 32],
    candidate: [u8; 32],
    fence: [u8; 32],
    fence_plan: [u8; 32],
    cutover_plan: [u8; 32],
    publication_plan: [u8; 32],
    authority_posture: [u8; 32],
    admission_policy: [u8; 32],
}

pub(super) fn binding_from_record(kind: &Record) -> OperationalRecoveryActionBinding {
    use OperationalRecoveryActionBinding as Binding;
    match kind {
        Record::AuthorizationConsumed {
            plan_fingerprint,
            execution_plan_fingerprint,
            replay_same_operation_identity,
            ..
        } => Binding::Authorization {
            plan: *plan_fingerprint,
            execution: *execution_plan_fingerprint,
            replayed: *replay_same_operation_identity,
        },
        Record::ReplicaBootstrapTransferRecorded {
            authorization_plan_fingerprint,
            execution_plan_fingerprint,
            receipt_identity,
            durable_target_identity,
            source_lease_identity,
            ..
        } => Binding::BootstrapTransfer {
            authorization_plan: *authorization_plan_fingerprint,
            execution_plan: *execution_plan_fingerprint,
            receipt: *receipt_identity,
            source_lease: *source_lease_identity,
            target: *durable_target_identity,
        },
        Record::ReplicaBootstrapCompleted {
            receipt_identity,
            verification_identity,
            source_lease_identity,
        } => Binding::BootstrapCompleted {
            receipt: *receipt_identity,
            source_lease: *source_lease_identity,
            verification: *verification_identity,
        },
        Record::ReplicaPromotionFenceRecorded {
            authorization_plan_fingerprint,
            execution_plan_fingerprint,
            fence_identity,
            promoted_epoch,
        } => Binding::PromotionFence {
            authorization_plan: *authorization_plan_fingerprint,
            execution_plan: *execution_plan_fingerprint,
            fence: *fence_identity,
            epoch: *promoted_epoch,
        },
        Record::ReplicaPromotionRecorded {
            authorization_plan_fingerprint,
            execution_plan_fingerprint,
            receipt_identity,
            fence_identity,
            promoted_epoch,
        } => Binding::PromotionRecorded {
            authorization_plan: *authorization_plan_fingerprint,
            execution_plan: *execution_plan_fingerprint,
            receipt: *receipt_identity,
            fence: *fence_identity,
            epoch: *promoted_epoch,
        },
        Record::ReplicaPromotionPublished {
            receipt_identity,
            verification_identity,
            publication_identity,
            target_identity,
            promoted_epoch,
        } => Binding::PromotionPublished {
            receipt: *receipt_identity,
            publication: *publication_identity,
            verification: *verification_identity,
            target: *target_identity,
            epoch: *promoted_epoch,
        },
        Record::ReplicaPromotionReadmitted {
            publication_identity,
            serve_lease_identity,
            serving_epoch,
        } => Binding::PromotionReadmitted {
            publication: *publication_identity,
            serve_lease: *serve_lease_identity,
            epoch: *serving_epoch,
        },
        Record::OldPrimaryRejoinPlanned {
            promotion_receipt_identity,
            rejoin_plan_fingerprint,
            disposition_tag,
        } => Binding::RejoinPlanned {
            promotion_receipt: *promotion_receipt_identity,
            plan: *rejoin_plan_fingerprint,
            disposition: *disposition_tag,
        },
        Record::OldPrimaryRejoinCompleted {
            rejoin_plan_fingerprint,
            rejoin_receipt_identity,
            forensic_retention_identity,
            rebootstrap_target_identity,
            disposition_tag,
        } => Binding::RejoinCompleted {
            plan: *rejoin_plan_fingerprint,
            receipt: *rejoin_receipt_identity,
            forensic_retention: *forensic_retention_identity,
            rebootstrap_target: *rebootstrap_target_identity,
            disposition: *disposition_tag,
        },
        _ => Binding::None,
    }
}

impl PublicationBinding {
    pub(super) const fn publication(self) -> [u8; 32] {
        self.publication
    }
    pub(super) const fn fence(self) -> [u8; 32] {
        self.fence
    }
    pub(super) const fn fence_plan(self) -> [u8; 32] {
        self.fence_plan
    }
}
