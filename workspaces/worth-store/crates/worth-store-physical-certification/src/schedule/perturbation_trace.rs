use sha2::{Digest, Sha256};
use worth_foundational::CanonicalDigestId;

use super::{SchedulePerturbationSeed, ScheduleReplayDenial};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulePerturbationDecision {
    family: Box<str>,
    choice: Box<str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedulePerturbationTrace {
    seed: SchedulePerturbationSeed,
    decisions: Box<[SchedulePerturbationDecision]>,
    digest: CanonicalDigestId,
}

impl SchedulePerturbationDecision {
    pub fn new(
        family: impl Into<Box<str>>,
        choice: impl Into<Box<str>>,
    ) -> Result<Self, ScheduleReplayDenial> {
        let family = family.into();
        let choice = choice.into();
        if family.trim().is_empty() || choice.trim().is_empty() {
            return Err(ScheduleReplayDenial::EmptyPerturbationDecision);
        }
        Ok(Self { family, choice })
    }

    pub fn family(&self) -> &str {
        &self.family
    }

    pub fn choice(&self) -> &str {
        &self.choice
    }

    pub fn encoded(&self) -> String {
        format!("{}={}", self.family, self.choice)
    }
}

impl SchedulePerturbationTrace {
    pub fn new(
        seed: SchedulePerturbationSeed,
        decisions: impl IntoIterator<Item = SchedulePerturbationDecision>,
    ) -> Result<Self, ScheduleReplayDenial> {
        let decisions = decisions.into_iter().collect::<Vec<_>>();
        if decisions.is_empty() {
            return Err(ScheduleReplayDenial::EmptyPerturbationTrace);
        }
        let mut families = decisions
            .iter()
            .map(|decision| decision.family())
            .collect::<Vec<_>>();
        families.sort_unstable();
        if families.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(ScheduleReplayDenial::DuplicatePerturbationDecisionFamily);
        }
        let digest = trace_digest(seed, &decisions);
        Ok(Self {
            seed,
            decisions: decisions.into_boxed_slice(),
            digest: CanonicalDigestId::new(digest),
        })
    }

    pub const fn seed(&self) -> SchedulePerturbationSeed {
        self.seed
    }

    pub fn decisions(&self) -> &[SchedulePerturbationDecision] {
        &self.decisions
    }

    pub const fn digest(&self) -> [u8; 32] {
        *self.digest.bytes()
    }
}

fn trace_digest(
    seed: SchedulePerturbationSeed,
    decisions: &[SchedulePerturbationDecision],
) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth.store.physical.schedule-perturbation-trace.v1");
    digest.update(seed.value().to_le_bytes());
    for decision in decisions {
        update_text(&mut digest, decision.family());
        update_text(&mut digest, decision.choice());
    }
    digest.finalize().into()
}

fn update_text(digest: &mut Sha256, value: &str) {
    digest.update((value.len() as u64).to_le_bytes());
    digest.update(value.as_bytes());
}
