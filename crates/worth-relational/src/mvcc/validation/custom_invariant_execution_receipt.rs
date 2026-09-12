use std::sync::Arc;

use crate::validation::data::{
    CustomInvariantProvenance, CustomInvariantSemanticIdentity, InvariantDecisionKind,
    InvariantExecutionPoint, InvariantReportedRule, InvariantVerdict,
};
use crate::validation::engine::InvariantExecutionResult;

/// Immutable evidence of one installed custom invariant's execution against a
/// Relational-owned proposed candidate.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CustomInvariantExecutionReceipt {
    identity: CustomInvariantSemanticIdentity,
    execution_point: InvariantExecutionPoint,
    verdict: InvariantDecisionKind,
    provenance: CustomInvariantProvenance,
    work_units: std::num::NonZeroU64,
}

impl CustomInvariantExecutionReceipt {
    pub fn identity(&self) -> &CustomInvariantSemanticIdentity {
        &self.identity
    }

    pub const fn work_units(&self) -> std::num::NonZeroU64 {
        self.work_units
    }

    pub fn rule_id(&self) -> &crate::validation::data::CustomInvariantRuleId {
        &self.identity.rule_id
    }

    pub const fn semantic_version(
        &self,
    ) -> crate::validation::data::CustomInvariantSemanticVersion {
        self.identity.semantic_version
    }

    pub const fn execution_point(&self) -> InvariantExecutionPoint {
        self.execution_point
    }

    pub const fn verdict(&self) -> InvariantDecisionKind {
        self.verdict
    }

    pub const fn provenance(&self) -> &CustomInvariantProvenance {
        &self.provenance
    }
}

pub(crate) fn collect_custom_invariant_execution_receipts(
    executions: [&InvariantExecutionResult; 3],
    registry: &crate::validation::FrozenCustomInvariantRegistry,
) -> Arc<[CustomInvariantExecutionReceipt]> {
    let mut receipts = executions
        .into_iter()
        .flat_map(InvariantExecutionResult::results)
        .filter_map(|result| {
            let InvariantReportedRule::Custom(identity) = &result.rule else {
                return None;
            };
            registry
                .get(identity, result.execution_point)
                .expect("executed custom invariant remains in its frozen registry");
            Some(CustomInvariantExecutionReceipt {
                identity: identity.clone(),
                execution_point: result.execution_point,
                verdict: match result.verdict {
                    InvariantVerdict::Pass => InvariantDecisionKind::Passed,
                    InvariantVerdict::Advisory { .. } => InvariantDecisionKind::Advisory,
                    InvariantVerdict::Violation(_) => InvariantDecisionKind::Violated,
                },
                provenance: result
                    .custom_provenance()
                    .expect("custom execution results carry candidate provenance")
                    .clone(),
                work_units: result
                    .custom_provenance()
                    .expect("custom execution results carry candidate provenance")
                    .work_units,
            })
        })
        .collect::<Vec<_>>();
    receipts.sort_by(|left, right| {
        (
            left.rule_id().as_str(),
            left.semantic_version().major,
            left.semantic_version().minor,
            left.execution_point(),
        )
            .cmp(&(
                right.rule_id().as_str(),
                right.semantic_version().major,
                right.semantic_version().minor,
                right.execution_point(),
            ))
    });
    receipts.into()
}
