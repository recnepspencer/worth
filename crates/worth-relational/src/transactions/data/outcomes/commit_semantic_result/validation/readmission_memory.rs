//! One decode's grants attach to the actual independently shared validation leaves.
use super::*;
use crate::graph_member::{NativeMemberKey, ProjectedMemberStorage};
use crate::validation::data::{CustomInvariantRuleId, InvariantViolationFields};
use worth_execution::ExecutionMemoryReservation;
mod leaves;
pub(crate) use leaves::{ValidationLeafMemory, ValidationTouchedMemory};

pub(crate) struct ValidationReadmissionMemory {
    pub(crate) rules: Vec<ValidationLeafMemory>,
    pub(crate) touched: Vec<ValidationTouchedMemory>,
    pub(crate) projected: Vec<ValidationLeafMemory>,
    pub(crate) _tree_memory: Option<ExecutionMemoryReservation>,
}
impl ValidationReadmissionMemory {
    pub(crate) fn attach(
        &mut self,
        actual: &[RelationalInvariantExecutionMeaning],
    ) -> Result<(), &'static str> {
        let mut rules = self.rules.iter_mut();
        let mut touched = self.touched.iter_mut();
        let mut projected = self.projected.iter_mut();
        for execution in actual {
            for check in &execution.checks {
                rule(&mut rules, &check.rule)?;
                if let Some(provenance) = &check.custom_provenance {
                    touched
                        .next()
                        .ok_or("missing touched summary admission")?
                        .install(&provenance.touched)?;
                }
                let violation = match &check.verdict {
                    InvariantVerdict::Violation(value)
                    | InvariantVerdict::Advisory {
                        violation: value, ..
                    } => Some(value),
                    _ => None,
                };
                if let Some(violation) = violation {
                    match &violation.fields {
                        InvariantViolationFields::CustomInvariantFailure { identity, .. } => {
                            install_rule(&mut rules, &identity.semantic_identity().rule_id)?
                        }
                        InvariantViolationFields::CustomInvariantViolation { identity, .. } => {
                            install_rule(&mut rules, &identity.rule_id)?
                        }
                        InvariantViolationFields::DerivedRelationGraph { violation, .. } => {
                            member(&mut projected, &violation.member)?;
                            if let Some(counterpart) = &violation.counterpart {
                                member(&mut projected, counterpart)?;
                            }
                        }
                        _ => {}
                    }
                }
            }
            for decision in &execution.decisions {
                rule(&mut rules, &decision.rule)?;
            }
        }
        if rules.next().is_some() || touched.next().is_some() || projected.next().is_some() {
            return Err("unmatched validation leaf admission");
        }
        Ok(())
    }
}
fn rule<'a>(
    memory: &mut impl Iterator<Item = &'a mut ValidationLeafMemory>,
    actual: &InvariantReportedRule,
) -> Result<(), &'static str> {
    if let InvariantReportedRule::Custom(identity) = actual {
        install_rule(memory, &identity.rule_id)?;
    }
    Ok(())
}
fn install_rule<'a>(
    memory: &mut impl Iterator<Item = &'a mut ValidationLeafMemory>,
    actual: &CustomInvariantRuleId,
) -> Result<(), &'static str> {
    memory
        .next()
        .ok_or("missing custom rule admission")?
        .install(actual.as_str().len(), |grant| {
            actual.retain_decoded_memory(grant)
        })
}
fn member<'a>(
    memory: &mut impl Iterator<Item = &'a mut ValidationLeafMemory>,
    actual: &NativeMemberKey,
) -> Result<(), &'static str> {
    if let NativeMemberKey::Projected(actual) = actual {
        install_projected(memory, actual)?;
    }
    Ok(())
}
fn install_projected<'a>(
    memory: &mut impl Iterator<Item = &'a mut ValidationLeafMemory>,
    actual: &ProjectedMemberStorage,
) -> Result<(), &'static str> {
    let length = actual
        .domain
        .schema
        .0
        .len()
        .checked_add(actual.domain.name.len())
        .and_then(|n| n.checked_add(actual.case.len()))
        .ok_or("projected member length overflow")?;
    memory
        .next()
        .ok_or("missing projected member admission")?
        .install(length, |grant| actual.retain_decoded_memory(grant))
}
