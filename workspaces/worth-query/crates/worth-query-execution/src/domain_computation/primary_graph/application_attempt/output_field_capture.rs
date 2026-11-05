//! Original postcommit capture failure custody, distinct from absent Native evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OutputFieldCaptureFailure {
    Rebase {
        reason: crate::domain_computation::primary_graph::vocabulary::provider::diagnostic::RebaseVerificationReason,
        own_effect: super::super::provider::OwnEffectOnReads,
    },
    Retention(StoreDenial),
}
pub use super::retained_decision_facts::StoreDenial;
