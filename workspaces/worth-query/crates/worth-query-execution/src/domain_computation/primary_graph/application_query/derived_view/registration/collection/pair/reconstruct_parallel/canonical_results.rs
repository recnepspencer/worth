//! The sole owner-apply door consumes the map's canonical completed prefix.
use super::super::Denial;
use crate::domain_computation::primary_graph::application_query::read_execution::prepared_pair::PairOutput;
use worth_execution::MapOutcome;
use worth_proof::CanonicalUniqueVec;
use worth_relational::facade::identity::EntityId;

pub(super) struct CanonicalPairResults {
    roots: CanonicalUniqueVec<EntityId>,
    prefix: Vec<PairOutput>,
}
impl CanonicalPairResults {
    pub(super) fn settle(
        roots: CanonicalUniqueVec<EntityId>,
        outcome: MapOutcome<PairOutput, Denial>,
    ) -> (Self, Option<Denial>) {
        #[cfg(test)]
        let charge = match &outcome {
            MapOutcome::Complete { report, .. } | MapOutcome::Stopped { report, .. } => {
                report.charged_work()
            }
        };
        let (prefix, stop) = match outcome {
            MapOutcome::Complete { values, .. } => (values, None),
            MapOutcome::Stopped {
                completed_prefix,
                reason,
                ..
            } => (
                completed_prefix,
                Some(super::denial::map(reason, roots.as_slice())),
            ),
        };
        #[cfg(test)]
        crate::domain_computation::primary_graph::application_query::DispatchWitness::settled(
            charge,
            prefix.len(),
        );
        (Self { roots, prefix }, stop)
    }
    pub(super) fn len(&self) -> usize {
        self.prefix.len()
    }
    pub(super) fn into_prefix(self) -> impl Iterator<Item = (EntityId, PairOutput)> {
        self.roots.into_keys().into_iter().zip(self.prefix)
    }
}
