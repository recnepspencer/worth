//! Attribution of a changed read to the later round that changed it. The
//! timeline holds every round that applied, including a chain's superseded
//! rounds (PR C §1.5, R4): a successor replaces its chain's writer in place,
//! but the images its earlier rounds produced stay where they applied.
use super::*;

/// One applied round: its member (the chain's current writer for a
/// superseded round), where it applied, the stage that owed it, and the
/// image right after it.
pub(super) struct Round<'m> {
    pub(super) member: &'m CoupledApplicationMember,
    pub(super) position: usize,
    owed: Option<usize>,
    pub(super) pin: &'m Candidate,
}
impl Round<'_> {
    /// Whether this round runs one commit per stage at or after the authored
    /// stage appended at `stage`: that stage or a later one, or a producer
    /// round owed at or after it (a root selection's stage window, #238).
    pub(super) fn runs_from_stage(&self, stage: usize) -> bool {
        match &self.member.kind {
            CoupledApplicationMemberKind::AuthoredStage { .. } => self.position >= stage,
            CoupledApplicationMemberKind::Producer { .. } => {
                self.owed.is_some_and(|owed| owed >= stage)
            }
        }
    }
    /// Whether this round runs after `reader` one commit per stage: a later
    /// authored stage, or a producer round owed at or after the reader.
    pub(super) fn settles_after(&self, reader: usize) -> bool {
        match &self.member.kind {
            CoupledApplicationMemberKind::AuthoredStage { .. } => self.position > reader,
            CoupledApplicationMemberKind::Producer { .. } => {
                self.owed.is_some_and(|owed| owed >= reader)
            }
        }
    }
}

/// Every round the members applied, in position order: each member's
/// superseded rounds, then the member itself.
pub(super) fn rounds(members: &[CoupledApplicationMember]) -> Vec<Round<'_>> {
    let mut rounds = Vec::new();
    for member in members {
        let attempt = &member
            .owned
            .as_ref()
            .expect("actual extracted writer")
            .attempt;
        if let Some(preparation) = attempt.coupled_preparation() {
            rounds.extend(preparation.superseded_rounds.iter().map(|round| Round {
                member,
                position: round.position,
                owed: Some(round.owed),
                pin: &round.after_own,
            }));
        }
        rounds.push(Round {
            member,
            position: member.position,
            owed: member.owed,
            pin: attempt
                .coupled_after_own()
                .expect("actual writer post-own image"),
        });
    }
    rounds.sort_by_key(|round| round.position);
    rounds
}

/// Only a changed fact walks the later writers. Reuse the post-own pins already
/// held by the admitted writers; healthy facts pay no intermediate comparisons.
/// `compare` names the fact that differs between two adjacent images, so each
/// refusal names the fact its own writer changed. An authored reader accepts
/// changes made by members that sequentially run after it; any other change
/// refuses, attributed to its writer.
pub(super) fn changed(
    reader: &WorthQueryPrimaryGraphApplicationAttempt,
    source: worth_relational::facade::identity::EntityId,
    position: usize,
    members: &[CoupledApplicationMember],
    observed: &Candidate,
    settles: impl Fn(&Round<'_>) -> bool,
    compare: impl Fn(
        &Candidate,
        &Candidate,
    ) -> Result<
        Option<String>,
        crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial,
    >,
    final_locator: String,
    detail: impl Fn(&str) -> String,
) -> Result<(), CoupledSupportPreparationDenial> {
    let mut settled = false;
    let mut earlier = observed;
    for round in rounds(members)
        .into_iter()
        .filter(|round| round.position > position)
    {
        let writer = round.member;
        let answer = compare(earlier, round.pin);
        earlier = round.pin;
        match answer {
            Ok(None) => continue,
            Err(cause) => return require_answer(Err(cause), source, String::new),
            Ok(Some(_)) if settles(&round) => settled = true,
            Ok(Some(locator)) => {
                let reader = reader
                    .coupled_authored_subject()
                    .map(|(stage, binding)| format!("authored stage {} {binding}", stage.ordinal()))
                    .unwrap_or_else(|| format!("reader {source:?}"));
                let later = match &writer.kind {
                    CoupledApplicationMemberKind::AuthoredStage {
                        identity, binding, ..
                    } => format!("authored stage {} {binding}", identity.ordinal()),
                    CoupledApplicationMemberKind::Producer { producer, meaning } => {
                        format!("producer {producer} {}", meaning.family)
                    }
                };
                let failure = Failure::coupled_read_support(
                    source,
                    None,
                    format!("{reader}; later writer {later}: {}", detail(&locator)),
                )
                .with_coupled_writer(writer.source());
                return Err(CoupledSupportPreparationDenial::Changed(failure));
            }
        }
    }
    if settled {
        return Ok(());
    }
    require_answer(Ok(false), source, || {
        format!(
            "{}; no admitted later writer could be attributed",
            detail(&final_locator)
        )
    })
}
