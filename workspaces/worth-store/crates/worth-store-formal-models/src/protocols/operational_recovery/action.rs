#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OperationalRecoveryActionKind {
    WorkflowOpened,
    SourceLeasePersisted,
    MaterializationOpened,
    MaterializationRecorded,
    IndependentVerificationRecorded,
    Abandoned,
    AuthorizationConsumed,
    OwnerExecutionOpened,
    OwnerEffectStarted,
    OwnerReceiptPersisted,
    WorkflowOwnerReceiptPersisted,
    DispositionRecorded,
    StagingCompleted,
    PublicationPrepared,
    PublicationPending,
    PublicationDisposition,
    FenceReleased,
    ReplicaBootstrapTransferRecorded,
    ReplicaBootstrapCompleted,
    ReplicaPromotionFenceRecorded,
    ReplicaPromotionRecorded,
    ReplicaPromotionPublished,
    ReplicaPromotionReadmitted,
    OldPrimaryRejoinPlanned,
    OldPrimaryRejoinCompleted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationalRecoveryAction {
    pub(super) authority_identity: [u8; 32],
    pub(super) operation_identity: String,
    pub(super) transition_identity: String,
    pub(super) kind: OperationalRecoveryActionKind,
    pub(super) owner_tag: Option<u8>,
    pub(super) binding: super::binding::OperationalRecoveryActionBinding,
}

impl OperationalRecoveryAction {
    pub const fn authority_identity(&self) -> [u8; 32] {
        self.authority_identity
    }

    pub fn operation_identity(&self) -> &str {
        &self.operation_identity
    }

    pub fn transition_identity(&self) -> &str {
        &self.transition_identity
    }

    pub const fn kind(&self) -> OperationalRecoveryActionKind {
        self.kind
    }

    pub const fn owner_tag(&self) -> Option<u8> {
        self.owner_tag
    }
}

#[cfg(test)]
mod tests {
    use super::{OperationalRecoveryAction, OperationalRecoveryActionKind as Action};
    use crate::protocols::operational_recovery::{
        OperationalRecoveryInvariant, OperationalRecoveryModel,
    };

    #[test]
    fn promotion_without_a_durable_external_fence_is_rejected() {
        let mut model = OperationalRecoveryModel::default();
        model
            .apply(&action("authorize", Action::AuthorizationConsumed))
            .unwrap();

        let denial = model
            .apply(&action("promote", Action::ReplicaPromotionRecorded))
            .unwrap_err();

        assert_eq!(
            denial.invariant(),
            OperationalRecoveryInvariant::ExternalFenceBeforePromotion
        );
    }

    #[test]
    fn valid_promotion_trace_reaches_each_concrete_transition() {
        let mut model = OperationalRecoveryModel::default();
        for event in [
            action("authorize", Action::AuthorizationConsumed),
            action("fence", Action::ReplicaPromotionFenceRecorded),
            action("promote", Action::ReplicaPromotionRecorded),
            action("publish", Action::ReplicaPromotionPublished),
            action("readmit", Action::ReplicaPromotionReadmitted),
            action("rejoin", Action::OldPrimaryRejoinPlanned),
            action("rejoin-complete", Action::OldPrimaryRejoinCompleted),
        ] {
            model.apply(&event).unwrap();
        }

        assert_eq!(model.reached_transitions().len(), 7);
    }

    #[test]
    fn bootstrap_completion_without_transfer_is_rejected() {
        let mut model = OperationalRecoveryModel::default();
        model
            .apply(&action("authorize", Action::AuthorizationConsumed))
            .unwrap();
        let denial = model
            .apply(&action("complete", Action::ReplicaBootstrapCompleted))
            .unwrap_err();
        assert_eq!(
            denial.invariant(),
            OperationalRecoveryInvariant::BootstrapTransferBeforeCompletion
        );
    }

    #[test]
    fn promotion_readmission_without_publication_is_rejected() {
        let mut model = OperationalRecoveryModel::default();
        for event in [
            action("authorize", Action::AuthorizationConsumed),
            action("fence", Action::ReplicaPromotionFenceRecorded),
            action("promote", Action::ReplicaPromotionRecorded),
        ] {
            model.apply(&event).unwrap();
        }
        let denial = model
            .apply(&action("readmit", Action::ReplicaPromotionReadmitted))
            .unwrap_err();
        assert_eq!(
            denial.invariant(),
            OperationalRecoveryInvariant::PromotionPublicationBeforeReadmission
        );
    }

    #[test]
    fn owner_receipt_without_open_execution_is_rejected() {
        let mut model = OperationalRecoveryModel::default();
        model
            .apply(&action("authorize", Action::AuthorizationConsumed))
            .unwrap();
        let denial = model
            .apply(&action("receipt", Action::OwnerReceiptPersisted))
            .unwrap_err();
        assert_eq!(
            denial.invariant(),
            OperationalRecoveryInvariant::OwnerExecutionBeforeReceipt
        );
    }

    #[test]
    fn staging_requires_both_distinct_workflow_owner_receipts() {
        let mut model = OperationalRecoveryModel::default();
        model
            .apply(&action("authorize", Action::AuthorizationConsumed))
            .unwrap();
        for owner_tag in [1, 2] {
            model.apply(&owner_receipt(owner_tag)).unwrap();
        }
        model
            .apply(&action("staged", Action::StagingCompleted))
            .expect("both owner receipts admit staging completion");
    }

    #[test]
    fn backup_verification_requires_the_complete_materialization_prefix() {
        let mut model = OperationalRecoveryModel::default();
        model
            .apply(&action("open", Action::WorkflowOpened))
            .unwrap();
        let denial = model
            .apply(&action("verify", Action::IndependentVerificationRecorded))
            .unwrap_err();
        assert_eq!(
            denial.invariant(),
            OperationalRecoveryInvariant::MaterializationBeforeVerification
        );
    }

    #[test]
    fn completed_backup_is_terminal_in_the_model() {
        let mut model = OperationalRecoveryModel::default();
        for event in [
            action("open", Action::WorkflowOpened),
            action("lease", Action::SourceLeasePersisted),
            action("materialize-open", Action::MaterializationOpened),
            action("materialize", Action::MaterializationRecorded),
            action("verify", Action::IndependentVerificationRecorded),
        ] {
            model.apply(&event).unwrap();
        }
        let denial = model
            .apply(&action("late", Action::MaterializationRecorded))
            .unwrap_err();
        assert_eq!(
            denial.invariant(),
            OperationalRecoveryInvariant::TerminalOperationHasNoLaterTransition
        );
    }

    fn action(transition: &str, kind: Action) -> OperationalRecoveryAction {
        OperationalRecoveryAction {
            authority_identity: [0; 32],
            operation_identity: "promotion-1".to_owned(),
            transition_identity: transition.to_owned(),
            kind,
            owner_tag: None,
            binding: super::super::binding::OperationalRecoveryActionBinding::None,
        }
    }

    fn owner_receipt(owner_tag: u8) -> OperationalRecoveryAction {
        OperationalRecoveryAction {
            owner_tag: Some(owner_tag),
            ..action(
                &format!("owner-{owner_tag}"),
                Action::WorkflowOwnerReceiptPersisted,
            )
        }
    }
}
