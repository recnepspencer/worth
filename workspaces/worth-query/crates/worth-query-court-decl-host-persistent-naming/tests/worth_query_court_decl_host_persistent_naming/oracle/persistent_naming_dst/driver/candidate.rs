//! Candidate steps: one edit envelope whose first stage makes one mutation
//! and whose last stage, the probe, reads one name on the unpublished
//! candidate. The world's answer (or typed refusal) must equal the
//! reference; an answer commits the envelope, a refusal leaves zero effects.
//! A name read of a record the first stage created answers with the name
//! the envelope gives it, which must be the name its commit records (#426).
use std::collections::BTreeMap;

use super::{placements, Driver};
use crate::oracle::persistent_naming_dst::candidate::{is_refusal, First, Read};
use crate::oracle::persistent_naming_dst::reference::Answer;
use crate::persistent_naming::coupled::candidate::Part;
use crate::persistent_naming::probe::{self, Observation};
use crate::persistent_naming::Member;
use worth_query_decl::facade::application_naming::{NameSelection, WorthQueryName};
use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome as Envelope;

const LABEL: &str = "candidate";

/// The candidate-basis refusal a decision-reader denial renders as.
fn refusal(rendered: &str) -> Result<&'static str, String> {
    [
        ("EndedInCandidate(", "RefusedEnded"),
        ("CandidateNameHistory", "RefusedHistory"),
        ("UnpublishedCandidateRecord(", "RefusedUnpublished"),
    ]
    .into_iter()
    .find_map(|(prefix, status)| rendered.starts_with(prefix).then_some(status))
    .ok_or_else(|| format!("not a candidate-basis refusal: {rendered}"))
}

impl Driver {
    pub(super) fn candidate(
        &mut self,
        first: &First,
        read: Read,
        expected: &Answer,
        step: usize,
    ) -> Result<Vec<(String, usize)>, String> {
        let (stage, created, plan) = match first {
            First::Edit(name) => (Part::Edit(self.key_of(*name), 900), Vec::new(), None),
            First::Commit(op) => {
                let (plan, created) = self.plan(op, step)?;
                let token = self.token(step, "candidate-first");
                (Part::Sculpt(token, plan.clone()), created, Some(plan))
            }
        };
        let token = self.token(step, "candidate-probe");
        let ended = first.ended();
        let keys = match read {
            Read::Resolve { name, scope, all } => {
                let held = self.names[&name];
                let selection = if all {
                    NameSelection::AllDescendants
                } else {
                    NameSelection::Exact
                };
                match scope {
                    None => probe::hold_selecting(&token, LABEL, held, selection),
                    Some(owner) => probe::hold_in(&token, LABEL, held, selection, owner),
                }
                // Every basis record the first stage leaves live, so each
                // resolved record maps to its key (each is a `name_of` too).
                let kept = self.keys.iter().filter(|(name, _)| !ended.contains(name));
                kept.map(|(_, key)| key.clone()).collect()
            }
            Read::NameOf(name) => vec![self.key_of(name)],
            Read::NameOfCreated => vec![created[0].0.clone()],
        };
        let parts = vec![stage, Part::Probe(token.clone(), keys)];
        let outcome = self.world.parts(&self.token(step, "candidate"), parts);
        let observed = probe::take(&token);
        let pending = created.first().map(|(_, model)| *model);
        let answered = self.candidate_answer(read, &observed, pending)?;
        if answered != *expected {
            return Err(format!(
                "candidate read {read:?} after {first:?} answered {answered:?}, model {expected:?}"
            ));
        }
        let committed = matches!(outcome, Envelope::Committed(_));
        if committed == is_refusal(expected) {
            return Err(format!(
                "the envelope must commit exactly when its read answers: {outcome:?}"
            ));
        }
        if !committed {
            if created.iter().any(|(key, _)| self.world.live(key))
                || ended.iter().any(|name| !self.world.live(&self.keys[name]))
            {
                return Err("a refused candidate read must leave zero effects".to_owned());
            }
            return Ok(Vec::new());
        }
        if let (Read::NameOfCreated, Some((key, _))) = (read, created.first()) {
            let read = observed.iter().find_map(|observation| match observation {
                Observation::Named { name, .. } => Some(*name),
                _ => None,
            });
            let after = self
                .world
                .names(&self.token(step, "candidate-after"), &[key.as_str()], 0);
            if read != Some(after[0]) {
                return Err(format!(
                    "`{key}` read {read:?} on the candidate, its commit recorded {:?}",
                    after[0]
                ));
            }
        }
        if let (Some(plan), First::Commit(op)) = (plan, first) {
            self.committed.push(vec![plan]);
            self.naming_commits += 1;
            for ((key, _), placement) in created.iter().zip(placements(op)) {
                self.admit_record(key, placement)?;
            }
        }
        Ok(created)
    }

    /// What the probe observed for `read`, as a model answer. A name no
    /// model name holds yet is the `pending` model name the first stage
    /// births, and only for a read of the record it created.
    fn candidate_answer(
        &self,
        read: Read,
        observed: &[Observation],
        pending: Option<usize>,
    ) -> Result<Answer, String> {
        let by_key: BTreeMap<&String, usize> =
            self.keys.iter().map(|(name, key)| (key, *name)).collect();
        let by_name: BTreeMap<WorthQueryName<Member>, usize> = self
            .names
            .iter()
            .map(|(model, name)| (*name, *model))
            .collect();
        let keyed = |key: &String| {
            by_key
                .get(key)
                .copied()
                .ok_or_else(|| format!("`{key}` holds no model name"))
        };
        let named = |name: &WorthQueryName<Member>| {
            by_name
                .get(name)
                .copied()
                .ok_or_else(|| "an unknown name".to_owned())
        };
        let born = |name: &WorthQueryName<Member>| match (read, named(name)) {
            (Read::NameOfCreated, Err(_)) => pending.ok_or_else(|| "an unknown name".to_owned()),
            (_, known) => known,
        };
        let answer = |status: &'static str, targets: Result<_, String>| {
            targets.map(|targets| Answer { status, targets })
        };
        let refused = |rendered: &str| answer(refusal(rendered)?, Ok(Default::default()));
        if let Read::Resolve { .. } = read {
            let found = observed.iter().find(|observation| match observation {
                Observation::ResolvedTo { label, .. }
                | Observation::Plural { label, .. }
                | Observation::Retired { label }
                | Observation::Ambiguous { label, .. }
                | Observation::NotInOwner { label, .. }
                | Observation::ResolveRefused { label, .. } => label == LABEL,
                _ => false,
            });
            return match found {
                Some(Observation::ResolvedTo { key, .. }) => {
                    answer("Exact", keyed(key).map(|name| [name].into()))
                }
                Some(Observation::Plural { keys, .. }) => {
                    answer("Plural", keys.iter().map(keyed).collect())
                }
                Some(Observation::Retired { .. }) => answer("Retired", Ok(Default::default())),
                Some(Observation::Ambiguous { candidates, .. }) => {
                    answer("Ambiguous", candidates.iter().map(named).collect())
                }
                Some(Observation::NotInOwner { candidates, .. }) => answer(
                    "NoCandidateInOwner",
                    candidates.iter().map(|(name, _)| named(name)).collect(),
                ),
                Some(Observation::ResolveRefused { refusal, .. }) => refused(refusal),
                other => Err(format!("the candidate resolution answered {other:?}")),
            };
        }
        let read_key = observed.iter().find_map(|observation| match observation {
            Observation::Named { name, .. } => {
                Some(answer("Named", born(name).map(|n| [n].into())))
            }
            Observation::NameRefused { refusal, .. } => Some(refused(refusal)),
            _ => None,
        });
        read_key.unwrap_or_else(|| Err(format!("the candidate name read answered {observed:?}")))
    }
}
