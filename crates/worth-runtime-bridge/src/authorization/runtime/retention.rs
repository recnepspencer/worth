//! One correspondence retention predicate with optional before-work admission.

use std::convert::Infallible;
use std::sync::Arc;

use super::{
    lower_effect, BridgeAuthorizationDecisionEvidence, BridgeAuthorizationRuntime,
    BridgeInstalledAuthorizationCorrespondence,
};

#[derive(Debug, Eq, PartialEq)]
pub enum BridgeAuthorizationRetentionStop<Stop> {
    Admission(Stop),
    AccountingOverflow,
}

enum RetentionStep<'a> {
    Lookup(usize),
    Selected(&'a BridgeInstalledAuthorizationCorrespondence),
}

impl BridgeAuthorizationRuntime {
    pub fn retains(&self, evidence: &BridgeAuthorizationDecisionEvidence) -> bool {
        self.retains_with(evidence, &mut |_| Ok::<_, Infallible>(()))
            .unwrap_or_else(|impossible| match impossible {})
    }

    pub fn retains_admitted<Stop>(
        &self,
        evidence: &BridgeAuthorizationDecisionEvidence,
        mut admit: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<bool, BridgeAuthorizationRetentionStop<Stop>> {
        use BridgeAuthorizationRetentionStop as Denial;
        // Pay the map/evidence header reads before measuring the selected
        // ordered descent. No allocation occurs in this retention predicate.
        admit(3, 0).map_err(Denial::Admission)?;
        self.retains_with(evidence, &mut |step| match step {
            RetentionStep::Lookup(entries) => {
                let levels = if entries == 0 {
                    0
                } else {
                    entries.ilog2() as usize + 1
                };
                let comparisons = entries
                    .min(11)
                    .checked_mul(levels)
                    .and_then(|count| count.checked_mul(32))
                    .and_then(|work| work.checked_add(1))
                    .ok_or(Denial::AccountingOverflow)?;
                admit(
                    u64::try_from(comparisons).map_err(|_| Denial::AccountingOverflow)?,
                    0,
                )
                .map_err(Denial::Admission)
            }
            RetentionStep::Selected(installed) => {
                // Selected policy/evidence headers precede their three rule
                // lengths. The installed Signal authority and all rule
                // decisions remain checked by the unchanged predicate below.
                admit(7, 0).map_err(Denial::Admission)?;
                let compared_rules = installed
                    .rules
                    .len()
                    .min(evidence.rule_decisions().len())
                    .min(evidence.signal().rule_decisions().len());
                // Signal policy and dependency digests, then five selected
                // graph/authority/length axes before the per-rule decisions.
                const FIXED_SELECTED_WORK: usize = 32 + 32 + 5;
                let work = compared_rules
                    .checked_mul(4)
                    .and_then(|work| work.checked_add(FIXED_SELECTED_WORK))
                    .ok_or(Denial::AccountingOverflow)?;
                admit(
                    u64::try_from(work).map_err(|_| Denial::AccountingOverflow)?,
                    0,
                )
                .map_err(Denial::Admission)
            }
        })
    }

    fn retains_with<Stop>(
        &self,
        evidence: &BridgeAuthorizationDecisionEvidence,
        prepare: &mut impl FnMut(RetentionStep<'_>) -> Result<(), Stop>,
    ) -> Result<bool, Stop> {
        prepare(RetentionStep::Lookup(self.correspondences.len()))?;
        let Some(installed) = self.correspondences.get(&evidence.correspondence()) else {
            return Ok(false);
        };
        prepare(RetentionStep::Selected(installed))?;
        Ok(Arc::ptr_eq(&installed.authority, evidence.authority())
            && installed.signal_policy.retains(evidence.signal())
            && evidence.dependency_identity() == evidence.signal().dependency_identity()
            && installed.rules.len() == evidence.rule_decisions().len()
            && installed
                .rules
                .iter()
                .zip(evidence.rule_decisions())
                .zip(evidence.signal().rule_decisions())
                .all(|((rule, decision), signal_decision)| {
                    rule.effect() == decision.effect()
                        && lower_effect(rule.effect()) == signal_decision.effect()
                        && decision.matched() == signal_decision.matched()
                }))
    }
}
