//! Coupled steps: one edit envelope whose two authored stages each create a
//! record and birth one fresh name, composed into one commit; both stages
//! minting the same fresh name must refuse `DuplicateBirth` with zero
//! effects. Replay of the canonical history re-runs each envelope whole.
use super::{authored, place, Driver};
use crate::oracle::persistent_naming_dst::ops::Placement;
use crate::persistent_naming::coupled::{envelope_refusal, stage, Stage};
use crate::persistent_naming::outputs::sculpt::{Plan, Step};
use crate::persistent_naming::outputs::{birth, Outcome};
use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome as Envelope;
use worth_query_host::facade::application_naming::LineageNameDenial;

/// One stage creating `key` at `at` and naming it model name `name`'s birth.
fn birth_stage(key: &str, name: usize, at: Placement) -> Plan {
    let mut plan = Plan::default();
    place(&mut plan, &[(key.to_owned(), at)]);
    plan.steps.push(Step::Birth(
        key.to_owned(),
        birth(&authored(name), "authored"),
    ));
    plan
}

impl Driver {
    fn stages(&self, step: usize, role: &str, plans: &[Plan]) -> Vec<Stage> {
        plans
            .iter()
            .enumerate()
            .map(|(index, plan)| stage(&self.token(step, &format!("{role}-{index}")), plan.clone()))
            .collect()
    }

    pub(super) fn coupled(
        &mut self,
        names: [usize; 2],
        at: [Placement; 2],
        step: usize,
    ) -> Result<Vec<(String, usize)>, String> {
        let keys = [self.fresh_key(), self.fresh_key()];
        let plans = vec![
            birth_stage(&keys[0], names[0], at[0]),
            birth_stage(&keys[1], names[1], at[1]),
        ];
        let stages = self.stages(step, "coupled", &plans);
        let outcome = self.world.envelope(&self.token(step, "coupled"), stages);
        if !matches!(outcome, Envelope::Committed(_)) {
            return Err(format!(
                "step {step}: the envelope must commit: {outcome:?}"
            ));
        }
        self.committed.push(plans);
        // One composed commit records both stages' birth events.
        self.naming_commits += 2;
        for (key, placement) in keys.iter().zip(at) {
            self.admit_record(key, placement)?;
        }
        Ok(vec![
            (keys[0].clone(), names[0]),
            (keys[1].clone(), names[1]),
        ])
    }

    pub(super) fn coupled_duplicate(
        &mut self,
        name: usize,
        at: Placement,
        step: usize,
    ) -> Result<Vec<(String, usize)>, String> {
        let keys = [self.fresh_key(), self.fresh_key()];
        let plans = [
            birth_stage(&keys[0], name, at),
            birth_stage(&keys[1], name, at),
        ];
        let stages = self.stages(step, "coupled-dup", &plans);
        let outcome = self
            .world
            .envelope(&self.token(step, "coupled-dup"), stages);
        if !matches!(
            envelope_refusal(&outcome),
            Some(LineageNameDenial::DuplicateBirth { .. })
        ) {
            return Err(format!(
                "two stages minting one name must refuse DuplicateBirth: {outcome:?}"
            ));
        }
        if keys.iter().any(|key| self.world.live(key)) {
            return Err("a refused envelope must leave zero effects".to_owned());
        }
        Ok(Vec::new())
    }

    /// Re-runs committed commit `ordinal` (`plans` its stages) on the fresh
    /// world: a direct operation, or the whole envelope.
    pub(super) fn replay(&mut self, ordinal: usize, plans: &[Plan]) -> Result<(), String> {
        let committed = if let [plan] = plans {
            let outcome = self
                .world
                .try_sculpt(&self.token(ordinal, "replay"), plan.clone())?;
            matches!(outcome, Outcome::Committed { .. })
        } else {
            let stages = self.stages(ordinal, "replay", plans);
            let outcome = self.world.envelope(&self.token(ordinal, "replay"), stages);
            matches!(outcome, Envelope::Committed(_))
        };
        if committed {
            Ok(())
        } else {
            Err(format!("replayed commit {ordinal} must commit"))
        }
    }
}
