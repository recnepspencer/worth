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
    // Every round in the window counts, a chain's superseded rounds included.
    let rounds = super::changed::rounds(members);
    let end = rounds
        .iter()
        .filter(|round| round.position > position && round.position < invalidator)
        .map(|round| round.pin)
        .last()
        .unwrap_or(support.pin());
    match compare(support.pin(), end) {
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
