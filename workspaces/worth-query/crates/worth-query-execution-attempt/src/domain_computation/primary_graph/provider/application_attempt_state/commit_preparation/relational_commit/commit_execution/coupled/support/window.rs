//! R4 windows for retained reads that qualified use reopened (PR C).
use super::*;

/// A retained read qualified use reopened (PR C R4) is checked only over its
/// window: from its own image to the image before its recorded invalidator.
/// That member's append is authenticated by the exact comparison that
/// invalidated the read, and the replacement derivation is checked on its own.
pub(super) fn superseded_window<'s>(
    source: worth_relational::facade::identity::EntityId,
    position: usize,
    support: &'s CandidateReadSupport,
    invalidator: usize,
    members: &[CoupledApplicationMember],
    compare: impl Fn(
        &Candidate,
        &Candidate,
    ) -> Result<
        Option<&'s crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact>,
        crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial,
    >,
) -> Result<(), CoupledSupportPreparationDenial> {
    let rounds = super::changed::rounds(members);
    let verdict = window_verdict(
        position,
        invalidator,
        rounds.iter().map(|round| (round.position, round.pin)),
        support.pin(),
        compare,
    );
    match verdict {
        Ok(None) => Ok(()),
        Ok(Some(fact)) => Err(CoupledSupportPreparationDenial::Changed(
            Failure::coupled_read_support(
                source,
                None,
                format!(
                    "{}: superseded {} changed before its recorded invalidator: {}",
                    support.subject(),
                    support.description(),
                    fact.locator_identity(),
                ),
            ),
        )),
        Err(cause) => require_answer(Err(cause), source, || support.subject().to_owned()),
    }
}

/// R4's verdict over one window, over any image type: the read's own image
/// compared with the image of the last round before its recorded invalidator
/// (a chain's superseded rounds included), or with its own image when no
/// round lies inside. A change there stays visible however valid the later
/// supersession (plan line 401).
pub(super) fn window_verdict<'p, P, Fact, Stop>(
    position: usize,
    invalidator: usize,
    rounds: impl IntoIterator<Item = (usize, &'p P)>,
    start: &'p P,
    compare: impl Fn(&'p P, &'p P) -> Result<Option<Fact>, Stop>,
) -> Result<Option<Fact>, Stop> {
    let end = rounds
        .into_iter()
        .filter(|(round, _)| *round > position && *round < invalidator)
        .map(|(_, image)| image)
        .last()
        .unwrap_or(start);
    compare(start, end)
}
