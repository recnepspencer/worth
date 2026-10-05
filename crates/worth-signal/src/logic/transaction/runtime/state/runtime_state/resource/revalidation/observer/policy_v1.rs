//! Fixed policy tags of the historical observer-demand-revalidation.v1 digest.
//! Public vocabulary must not reinterpret persisted canonical bytes.
use crate::logic::transaction::{ObservationDeliveryMode, ObservationPolicy, ObservationTrigger};

pub(super) fn encode(policy: ObservationPolicy) -> &'static str {
    match (policy.trigger(), policy.delivery_mode()) {
        (ObservationTrigger::Visited, ObservationDeliveryMode::PerCommittedTransaction) => {
            "Touched:PerCommittedTransaction"
        }
        (ObservationTrigger::Recomputed, ObservationDeliveryMode::PerCommittedTransaction) => {
            "Recomputed:PerCommittedTransaction"
        }
        (
            ObservationTrigger::MeaningfulChange,
            ObservationDeliveryMode::PerCommittedTransaction,
        ) => "MeaningfulChange:PerCommittedTransaction",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn historical_digest_tags_survive_the_public_visited_rename() {
        assert_eq!(
            encode(ObservationPolicy::visited()),
            "Touched:PerCommittedTransaction"
        );
        assert_eq!(
            encode(ObservationPolicy::recomputed()),
            "Recomputed:PerCommittedTransaction"
        );
        assert_eq!(
            encode(ObservationPolicy::meaningful_change()),
            "MeaningfulChange:PerCommittedTransaction"
        );
    }
}
