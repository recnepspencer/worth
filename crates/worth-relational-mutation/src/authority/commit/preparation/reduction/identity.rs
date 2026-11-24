use crate::contracts::validation::data::execution::{
    InvariantExecutionPoint, InvariantFailureEffect, InvariantReportedRule, InvariantWitnessKey,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ValidationResultIdentity {
    pub(crate) execution_point: InvariantExecutionPoint,
    pub(crate) failure_effect: InvariantFailureEffect,
    pub(crate) rule: InvariantReportedRule,
    pub(crate) witness: InvariantWitnessKey,
}

impl ValidationResultIdentity {
    pub(crate) fn from_parts(
        execution_point: InvariantExecutionPoint,
        failure_effect: InvariantFailureEffect,
        rule: InvariantReportedRule,
        witness: InvariantWitnessKey,
    ) -> Self {
        Self {
            execution_point,
            failure_effect,
            rule,
            witness,
        }
    }
}
