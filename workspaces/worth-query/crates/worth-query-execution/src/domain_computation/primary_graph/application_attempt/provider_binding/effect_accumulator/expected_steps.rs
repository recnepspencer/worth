use std::collections::BTreeMap;
use std::sync::Arc;

use super::super::effect_lowering::WorthQueryLoweredProviderEffect;
use super::super::WorthQueryApplicationAttemptDenial;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAttemptDenialKind;
use crate::domain_computation::{
    WorthQueryProvisionalEffectAction, WorthQueryProvisionalEffectStep,
};

/// One ordered, exact summary owned by effect lowering and shared through commit.
#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryExpectedEffectSteps {
    steps: Arc<[WorthQueryProvisionalEffectStep]>,
    preparation_work: WorthQueryExpectedEffectStepPreparationWork,
}

/// Preparation cardinality: one map lookup per lowered step and one exact
/// equality check per repeated identity. These are not map-comparison counts
/// or, by themselves, proof of the lookup algorithm's complexity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryExpectedEffectStepPreparationWork
{
    key_lookups: usize,
    duplicate_equalities: usize,
}

impl WorthQueryExpectedEffectStepPreparationWork {
    pub(in crate::domain_computation::primary_graph) const fn key_lookups(self) -> usize {
        self.key_lookups
    }

    pub(in crate::domain_computation::primary_graph) const fn duplicate_equalities(self) -> usize {
        self.duplicate_equalities
    }
}

impl WorthQueryExpectedEffectSteps {
    pub(super) fn from_lowered(
        lowered: Vec<WorthQueryLoweredProviderEffect>,
    ) -> Result<Self, WorthQueryApplicationAttemptDenial> {
        Self::from_step_groups(lowered.into_iter().filter_map(|effect| match effect {
            WorthQueryLoweredProviderEffect::Mutation { steps, .. } => Some(steps),
            WorthQueryLoweredProviderEffect::Emission(_) => None,
        }))
    }

    fn from_step_groups(
        groups: impl IntoIterator<Item = Vec<WorthQueryProvisionalEffectStep>>,
    ) -> Result<Self, WorthQueryApplicationAttemptDenial> {
        let mut positions = BTreeMap::new();
        let mut steps = Vec::new();
        let mut key_lookups = 0;
        let mut duplicate_equalities = 0;
        for step in groups.into_iter().flatten() {
            key_lookups += 1;
            let key = action_key(step.action());
            if let Some(&position) = positions.get(&key) {
                duplicate_equalities += 1;
                if steps[position] != step {
                    return Err(WorthQueryApplicationAttemptDenial::new(
                        WorthQueryApplicationAttemptDenialKind::ConflictingEffectStep,
                        key.to_string(),
                    ));
                }
            } else {
                positions.insert(key, steps.len());
                steps.push(step);
            }
        }
        Ok(Self {
            steps: steps.into(),
            preparation_work: WorthQueryExpectedEffectStepPreparationWork {
                key_lookups,
                duplicate_equalities,
            },
        })
    }

    pub(in crate::domain_computation::primary_graph) fn steps(
        &self,
    ) -> &[WorthQueryProvisionalEffectStep] {
        &self.steps
    }

    pub(in crate::domain_computation::primary_graph) fn shared_steps(
        &self,
    ) -> Arc<[WorthQueryProvisionalEffectStep]> {
        Arc::clone(&self.steps)
    }

    pub(in crate::domain_computation::primary_graph) const fn preparation_work(
        &self,
    ) -> WorthQueryExpectedEffectStepPreparationWork {
        self.preparation_work
    }
}

fn action_key(action: &WorthQueryProvisionalEffectAction) -> Arc<str> {
    match action {
        WorthQueryProvisionalEffectAction::Create { symbolic_identity } => {
            Arc::clone(symbolic_identity)
        }
        WorthQueryProvisionalEffectAction::Replace { target_identity } => {
            Arc::clone(target_identity)
        }
        WorthQueryProvisionalEffectAction::Retire { target_identity } => {
            Arc::clone(target_identity)
        }
        WorthQueryProvisionalEffectAction::DeriveView { view_identity } => {
            Arc::clone(view_identity)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create(identity: String) -> WorthQueryProvisionalEffectStep {
        WorthQueryProvisionalEffectStep::new(
            "mutation",
            WorthQueryProvisionalEffectAction::Create {
                symbolic_identity: identity.into(),
            },
        )
        .unwrap()
    }

    #[test]
    fn many_distinct_steps_have_one_key_lookup_each_and_keep_input_order() {
        let input = (0..10_000)
            .map(|n| create(format!("step-{n}")))
            .collect::<Vec<_>>();
        let summary = WorthQueryExpectedEffectSteps::from_step_groups([input.clone()]).unwrap();
        assert_eq!(summary.preparation_work().key_lookups(), 10_000);
        assert_eq!(summary.preparation_work().duplicate_equalities(), 0);
        assert_eq!(summary.steps(), input);
    }

    #[test]
    fn exact_duplicates_collapse_without_erasing_distinct_actions() {
        let first = create("first".into());
        let second = create("second".into());
        let summary = WorthQueryExpectedEffectSteps::from_step_groups([
            vec![first.clone(), second.clone()],
            vec![first.clone()],
        ])
        .unwrap();
        assert_eq!(summary.steps(), [first, second]);
        assert_eq!(summary.preparation_work().key_lookups(), 3);
        assert_eq!(summary.preparation_work().duplicate_equalities(), 1);
    }

    #[test]
    fn same_action_identity_with_different_semantics_is_not_silently_deduplicated() {
        let first = create("first".into());
        let different = create("first".into())
            .with_symbolic_dependencies(["other"])
            .unwrap();
        let denial = WorthQueryExpectedEffectSteps::from_step_groups([vec![first, different]])
            .err()
            .expect("conflicting step must deny");
        assert_eq!(
            denial.kind(),
            WorthQueryApplicationAttemptDenialKind::ConflictingEffectStep
        );
        assert_eq!(denial.subject(), "first");
    }

    #[test]
    fn distinct_actions_on_one_fact_identity_conflict_before_registration() {
        let replace = WorthQueryProvisionalEffectStep::new(
            "mutation",
            WorthQueryProvisionalEffectAction::Replace {
                target_identity: "fact".into(),
            },
        )
        .unwrap();
        let retire = WorthQueryProvisionalEffectStep::new(
            "mutation",
            WorthQueryProvisionalEffectAction::Retire {
                target_identity: "fact".into(),
            },
        )
        .unwrap();
        let denial = WorthQueryExpectedEffectSteps::from_step_groups([vec![replace, retire]])
            .err()
            .expect("two actions on one fact identity must deny");
        assert_eq!(
            denial.kind(),
            WorthQueryApplicationAttemptDenialKind::ConflictingEffectStep
        );
    }
}
