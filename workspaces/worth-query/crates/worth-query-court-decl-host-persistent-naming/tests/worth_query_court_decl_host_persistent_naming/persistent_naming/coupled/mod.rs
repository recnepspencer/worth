//! Slice 3B-2: coupled members name their outputs. Every authored stage of
//! one edit envelope (and every coupled producer the envelope runs) lowers
//! its own naming plan; Relational's coupled assembly rescopes each plan with
//! that member's commands and composes them into the envelope's one plan, so
//! commit finalization mints and validates every member's names exactly as
//! for a direct operation, and refuses the whole envelope typed otherwise.
use super::outputs::sculpt::{self, Plan, ScopeStep, Step};
use super::outputs::{birth, keys, named, piece, World};
use super::*;
use worth_query_decl::facade::application_program::ApplicationNoOutputGraph;
use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome as Envelope;
use worth_query_host::facade::application_naming::{
    LineageNameDenial, OccurrencePath, WorthQueryName,
};

pub(crate) mod candidate;
mod composition;
mod endpoints;
mod history;
mod invalidation;
pub(crate) mod producer;
mod refusals;

/// One stage of an envelope: the sculpt plan it runs under `token`.
pub struct Stage {
    pub token: String,
    pub plan: Plan,
}

pub fn stage(token: &str, plan: Plan) -> Stage {
    Stage {
        token: token.to_owned(),
        plan,
    }
}

impl World {
    /// Runs `stages` in order as one edit envelope under `key`: every stage
    /// a genuine authored sculpt run, the whole one Native commit.
    pub fn envelope(&self, key: &str, stages: Vec<Stage>) -> Envelope {
        let mut envelope = self
            .request()
            .edit_envelope()
            .idempotency(key.to_owned())
            .runtime_stages(&self.host)
            .ordered_authored()
            .unwrap();
        for Stage { token, plan } in stages {
            sculpt::author(&token, plan);
            envelope
                .push_stage::<ApplicationNoOutputGraph, _>(sculpt::Intent { token })
                .unwrap();
        }
        envelope
            .execute_performed(
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
    }

    pub fn committed_envelope(&self, key: &str, stages: Vec<Stage>) {
        let outcome = self.envelope(key, stages);
        assert!(
            matches!(outcome, Envelope::Committed(_)),
            "{key}: {outcome:?}"
        );
    }

    /// The names `keys` carry on the current world, read by one probe.
    pub fn names(&self, token: &str, keys: &[&str], ordinal: u64) -> Vec<WorthQueryName<Member>> {
        let observed = self.probe(token, keys, ordinal);
        keys.iter().map(|key| named(&observed, key)).collect()
    }
}

/// Relational's typed commit-time naming refusal behind an envelope.
pub fn envelope_refusal(outcome: &Envelope) -> Option<&LineageNameDenial> {
    match outcome {
        Envelope::Uncommitted(WorthQueryApplicationUncommitted::Denied(denial)) => {
            denial.persistent_name_refusal()
        }
        _ => None,
    }
}

pub fn scoped(occurrence: &str, steps: Vec<ScopeStep>) -> Step {
    Step::Scoped(OccurrencePath::root(occurrence).unwrap(), steps)
}
