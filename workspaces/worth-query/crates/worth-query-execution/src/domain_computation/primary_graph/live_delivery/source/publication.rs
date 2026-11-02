//! Publish the real World successor; retire queue custody after the Source gate.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryAdmittedApplicationEmissionBatch;

impl WorthQueryLiveDeliverySource {
    pub(in crate::domain_computation::primary_graph) fn publish(
        &self,
        mut reservation: WorthQueryLivePublicationReservation,
        publication: WorthQueryCommittedProductPublication,
        emissions: WorthQueryAdmittedApplicationEmissionBatch,
        predicate: Option<crate::domain_computation::primary_graph::application_query::derived_view::PreparedPredicatePublication>,
    ) -> usize {
        let emitted = emissions.len();
        let emissions = emissions.into_emissions();
        use crate::domain_computation::primary_graph::application_query::derived_view::PreparedPredicatePublication;
        let (predicate, resync) = match predicate {
            Some(PreparedPredicatePublication::Matched(notices)) => (Some(notices), false),
            Some(PreparedPredicatePublication::Untracked) => (None, true),
            None => (None, false),
        };
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.published_commit_count = state.published_commit_count.saturating_add(1);
        if !reservation.active {
            return emitted;
        }
        assert!(reservation.key.admits(&publication));
        let closed = state.closed;
        let partition = state
            .partitions
            .get_mut(&reservation.key)
            .expect("an active reservation retains its product partition");
        assert!(partition.reservation_live);
        assert!(
            partition
                .last_product_commit
                .as_ref()
                .is_none_or(|last| publication.composite_commit() > last),
            "application product publication causality is not strictly ordered"
        );
        let observation = publication
            .take_successor_observation()
            .expect("reserved live delivery carries World's exact successor observation");
        assert_eq!(observation.branch_identity(), publication.product_branch());
        assert_eq!(
            observation.lifecycle_incarnation(),
            publication.product_incarnation()
        );
        assert_eq!(
            observation.selected_commit(),
            publication.composite_commit()
        );
        for _ in 0..reservation.evictions {
            // Capacity was funded and allocated before World effects.
            reservation.retired_batches.push(
                partition
                    .batches
                    .pop_front()
                    .expect("reserved eviction exists"),
            );
        }
        partition.reservation_live = false;
        reservation.active = false;
        let sequence = partition.next_sequence;
        partition.next_sequence = sequence
            .checked_add(1)
            .expect("reservation proved live sequence capacity");
        partition.last_product_commit = Some(publication.composite_commit().clone());
        if resync {
            if let Some(epoch) = partition.resync_epoch.checked_add(1) {
                partition.resync_epoch = epoch;
            } else {
                partition.retired = true;
            }
        }
        if partition.subscriber_count == 0 || closed {
            let retired = partition
                .is_unowned()
                .then(|| state.remove_partition(&reservation.key));
            drop(state);
            drop(retired);
            return emitted;
        }
        let slot = reservation
            .slot
            .take()
            .expect("active reservation owns a batch slot");
        slot.0
            .set(WorthQueryLiveCommitBatch {
                sequence,
                publication,
                product: crate::basis::WorthQueryProductObservationLease::new(observation),
                emissions,
                predicate,
            })
            .unwrap_or_else(|_| panic!("reserved live batch initializes once"));
        partition.batches.push_back(slot);
        // In particular, the reservation's evictions cannot final-Drop under state.
        drop(state);
        emitted
    }
}
