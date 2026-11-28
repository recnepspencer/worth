#[cfg(test)]
use crate::internal_port::validation::data::rules::InvariantRuleOwnerPort as _;
pub mod canonical_registration_identity;
pub mod canonical_registration_tags;
pub mod relation_integrity_registration_plan;

#[cfg(test)]
#[cfg(test)]
mod registration_examples;
#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

use self::canonical_registration_identity::{
    canonical_catalog_registration_digest_hex, canonical_registration_bytes,
};
use super::descriptor::InvariantRuleDescriptor;
use super::execution::{InvariantExecutionPoint, InvariantFailureEffect};
use super::groups::InvariantCostClass;
use super::results::{InvariantAdvisory, InvariantViolation};
use super::rules::{InvariantRule, RecordKindTag};
use crate::validation::data::execution::InvariantVerdict;
pub use relation_integrity_registration_plan::append_relation_integrity_registrations;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvariantCatalog {
    pub registrations: Vec<InvariantRegistration>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvariantRegistration {
    pub descriptor: InvariantRuleDescriptor,
    pub rule: InvariantRule,
    pub execution_point: InvariantExecutionPoint,
    pub failure_effect: InvariantFailureEffect,
}

impl InvariantRegistration {
    pub fn mutation_sensitive_blocking(rule: InvariantRule) -> Self {
        Self::block_commit(rule, InvariantExecutionPoint::MutationSensitive)
    }

    pub fn commit_boundary_blocking(rule: InvariantRule) -> Self {
        Self::block_commit(rule, InvariantExecutionPoint::CommitBoundary)
    }

    pub fn snapshot_publication_blocking(rule: InvariantRule) -> Self {
        Self::block_publication(rule, InvariantExecutionPoint::SnapshotPublication)
    }

    pub fn certification_boundary_blocking(rule: InvariantRule) -> Self {
        Self::block_publication(rule, InvariantExecutionPoint::CertificationBoundary)
    }

    pub fn harness_audit_only(rule: InvariantRule) -> Self {
        Self::audit_only(rule, InvariantExecutionPoint::HarnessAudit)
    }
}

impl Default for InvariantCatalog {
    fn default() -> Self {
        Self {
            registrations: vec![
                InvariantRegistration::mutation_sensitive_blocking(
                    InvariantRule::LiveRecordRequiresSidecar(RecordKindTag::Entity),
                ),
                InvariantRegistration::mutation_sensitive_blocking(
                    InvariantRule::LiveRecordRequiresSidecar(RecordKindTag::Relation),
                ),
            ],
        }
    }
}

impl InvariantCatalog {
    pub fn canonicalized(&self) -> Self {
        let mut registrations = self.registrations.clone();
        registrations.sort_by_cached_key(canonical_registration_bytes);
        registrations.dedup();
        Self { registrations }
    }

    pub fn canonical_registration_digest(&self) -> String {
        canonical_catalog_registration_digest_hex(&self.canonicalized().registrations)
    }
}

/// Operations used by the Relational owners.
pub trait InvariantRegistrationOwnerPort: Sized {
    fn for_rule(
        rule: InvariantRule,
        execution_point: InvariantExecutionPoint,
        failure_effect: InvariantFailureEffect,
    ) -> Self;
    fn block_commit(rule: InvariantRule, execution_point: InvariantExecutionPoint) -> Self;
    fn block_publication(rule: InvariantRule, execution_point: InvariantExecutionPoint) -> Self;
    fn audit_only(rule: InvariantRule, execution_point: InvariantExecutionPoint) -> Self;
    fn cost(&self) -> InvariantCostClass;
    fn groups(&self) -> super::groups::InvariantGroupSet;
    fn verdict_for_violation(&self, violation: InvariantViolation) -> InvariantVerdict;
}

impl InvariantRegistrationOwnerPort for InvariantRegistration {
    fn for_rule(
        rule: InvariantRule,
        execution_point: InvariantExecutionPoint,
        failure_effect: InvariantFailureEffect,
    ) -> Self {
        let descriptor = rule.descriptor_for(failure_effect);
        Self {
            descriptor,
            rule,
            execution_point,
            failure_effect,
        }
    }

    fn block_commit(rule: InvariantRule, execution_point: InvariantExecutionPoint) -> Self {
        Self::for_rule(rule, execution_point, InvariantFailureEffect::BlockCommit)
    }

    fn block_publication(rule: InvariantRule, execution_point: InvariantExecutionPoint) -> Self {
        Self::for_rule(
            rule,
            execution_point,
            InvariantFailureEffect::BlockPublication,
        )
    }

    fn audit_only(rule: InvariantRule, execution_point: InvariantExecutionPoint) -> Self {
        Self::for_rule(rule, execution_point, InvariantFailureEffect::AuditOnly)
    }

    fn cost(&self) -> InvariantCostClass {
        self.descriptor.cost_class
    }

    fn groups(&self) -> super::groups::InvariantGroupSet {
        self.descriptor.groups
    }

    fn verdict_for_violation(&self, violation: InvariantViolation) -> InvariantVerdict {
        match self.failure_effect {
            InvariantFailureEffect::AuditOnly => InvariantVerdict::Advisory {
                violation,
                advisory: InvariantAdvisory::AuditOnly,
            },
            InvariantFailureEffect::BlockCommit | InvariantFailureEffect::BlockPublication => {
                InvariantVerdict::Violation(violation)
            }
        }
    }
}

impl InvariantCatalog {
    #[cfg(test)]
    pub(crate) fn contains_registration_kind(&self, rule: &InvariantRule) -> bool {
        self.registrations
            .iter()
            .any(|registration| registration.rule.same_registration_kind(rule))
    }
}
