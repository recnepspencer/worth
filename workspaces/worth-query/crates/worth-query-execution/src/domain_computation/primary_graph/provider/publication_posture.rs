//! One Query posture decision; Native independently re-admits the selected posture.
use worth_relational::facade::mvcc::RelationalRequiredOutputRouteAbsence;

pub(in crate::domain_computation::primary_graph) enum ProducerPublicationPosture {
    Ordinary(RelationalRequiredOutputRouteAbsence),
    Birth(RelationalRequiredOutputRouteAbsence),
    Preserve,
}

pub(in crate::domain_computation::primary_graph) fn select(
    absence: Option<RelationalRequiredOutputRouteAbsence>,
    all_created: bool,
) -> ProducerPublicationPosture {
    match absence {
        Some(absence) if !all_created => ProducerPublicationPosture::Ordinary(absence),
        Some(absence) => ProducerPublicationPosture::Birth(absence),
        None => ProducerPublicationPosture::Preserve,
    }
}
