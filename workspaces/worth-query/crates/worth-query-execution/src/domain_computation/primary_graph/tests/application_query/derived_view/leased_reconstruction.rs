//! Request backing, order, dependency and retention differential over real reads.
use super::*;
mod observation;
use observation::{observe, observe_case};
#[derive(Debug)]
pub(super) struct Observation {
    pub(super) values: Vec<String>,
    pub(super) work: u64,
    pub(super) denial: Option<WorthQueryManagedDerivedViewDenial>,
    pub(super) prepared: usize,
    pub(super) dependencies: Vec<(worth_relational::facade::identity::EntityId, String)>,
    pub(super) retention: (bool, usize, Vec<String>, Vec<String>),
    pub(super) roots: Vec<worth_relational::facade::identity::EntityId>,
    pub(super) map: (
        u64,
        usize,
        std::collections::BTreeMap<worth_relational::facade::identity::EntityId, u64>,
    ),
    pub(super) retained_values: Vec<String>,
    pub(super) interruption_point:
        Option<crate::domain_computation::primary_graph::application_query::InterruptionPoint>,
}

#[derive(Default)]
pub(super) struct Case {
    pub(super) interrupt: Option<(usize, bool)>,
    pub(super) query_interruption: Option<ScopeInterruption>,
    pub(super) warm: bool,
}

#[derive(Clone, Copy)]
pub(super) enum ScopeInterruption {
    Cancel,
    Deadline,
}

impl PartialEq for Observation {
    fn eq(&self, other: &Self) -> bool {
        // Later speculative checkpoint inventories are physical, not settled.
        self.values == other.values
            && self.work == other.work
            && self.denial == other.denial
            && self.prepared == other.prepared
            && self.dependencies == other.dependencies
            && self.retention == other.retention
            && self.roots == other.roots
            && self.map.0 == other.map.0
            && self.map.1 == other.map.1
            && self.retained_values == other.retained_values
    }
}

mod interruptions;
#[test]
fn serial_and_leased_pair_maps_agree_including_canonical_projection_and_charge() {
    for count in 0..=2 {
        let serial = observe(None, count, false, false, &[]);
        assert_eq!(serial.denial, None);
        for workers in [1, 2, 4] {
            assert_eq!(observe(Some(workers), count, false, false, &[]), serial);
        }
    }
}

#[test]
fn duplicate_roots_are_refused_before_preparation_and_reads() {
    for workers in [None, Some(1), Some(2), Some(4)] {
        let observed = observe(workers, 2, true, false, &[]);
        assert_eq!(
            observed.denial,
            Some(WorthQueryManagedDerivedViewDenial::DuplicateRoot {
                root: *observed.roots.last().unwrap()
            })
        );
        assert_eq!(observed.prepared, 0);
        assert!(observed.values.is_empty());
    }
}

#[test]
fn owner_panic_settles_and_releases_request_custody() {
    for workers in [None, Some(1), Some(2)] {
        let observed = observe(workers, 1, false, true, &[]);
        assert_eq!(
            observed.denial,
            Some(WorthQueryManagedDerivedViewDenial::OwnerPanic)
        );
        assert!(observed.values.is_empty());
    }
}

#[test]
fn least_failing_root_and_completed_owner_prefix_agree_across_backings() {
    for failures in [vec![0], vec![1], vec![2], vec![1, 2], vec![0, 2]] {
        let serial = observe(None, 3, false, false, &failures);
        match &serial.denial {
            Some(WorthQueryManagedDerivedViewDenial::ReadDenied { root, .. }) => {
                assert_eq!(*root, serial.roots[failures[0]])
            }
            cause => panic!("wrong least-root denial: {cause:?}"),
        }
        assert_eq!(
            serial.values.len(),
            failures[0],
            "the completed prefix is projected before the later stop"
        );
        for workers in [1, 2, 4] {
            assert_eq!(observe(Some(workers), 3, false, false, &failures), serial);
        }
    }
}

#[test]
fn earlier_owner_failure_wins_later_worker_stop_without_changing_settled_charge() {
    for workers in [None, Some(1), Some(2), Some(4)] {
        let later_stop = observe(workers, 3, false, false, &[1]);
        let earlier_failure = observe(workers, 3, false, true, &[1]);
        assert_eq!(
            earlier_failure.denial,
            Some(WorthQueryManagedDerivedViewDenial::OwnerPanic)
        );
        assert_eq!(earlier_failure.map.0, later_stop.map.0);
        assert_eq!(earlier_failure.work, later_stop.work);
    }
}
#[test]
fn failed_reconstruction_preserves_observed_same_basis_retention() {
    for workers in [None, Some(1), Some(2), Some(4)] {
        let observed = observe_case(
            workers,
            3,
            false,
            false,
            &[1],
            Case {
                warm: true,
                ..Case::default()
            },
        );
        assert!(!observed.retention.0);
        assert_eq!(observed.retention.1, 3);
        assert_eq!(observed.retained_values.len(), 3);
    }
}
