//! Original postcommit capture failure custody, distinct from absent Native evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum OutputFieldCaptureFailure {
    Rebase {
        reason: super::super::provider::RebaseVerificationReason,
        own_effect: super::super::provider::OwnEffectOnReads,
    },
    Retention(super::retained_decision_facts::StoreDenial),
}
