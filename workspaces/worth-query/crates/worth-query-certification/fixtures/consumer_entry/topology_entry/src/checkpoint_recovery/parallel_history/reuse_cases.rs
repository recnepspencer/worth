//! Complete reuse-decision inventory. A fresh installation is the reuse-off
//! control; it is an actual run, never the expected-value reference.
//!
//! | Input | Reachability / evidence |
//! |---|---|
//! | Code / computation identity | Proved at owner boundary: `installation::another_owner_with_the_same_types_runs_in_full`; applications cannot carry retained state between installations. |
//! | Owner installation | Proved at owner boundary: `installation::a_reinstalled_owner_runs_in_full`. |
//! | Producer edition | Proved at owner boundary: `carrying::another_input_or_edition_or_an_evicted_state_runs_in_full`; adoption carrying retained state would make it application-reachable. |
//! | Canonical input | Sensitive seeded case: `parallel_history_reuse::canonical_input`. |
//! | Membership facts | Varied here: `membership_join`, `membership_leave`, `membership_replacement`. |
//! | Item digest | Varied here: `persisting_item_digest_swap`; digest drift on persisting item ids changes their declared operations. |
//! | Item key facts | Sensitive seeded case: `parallel_history_reuse::key_facts`. |
//! | Partition gather facts | Sensitive seeded case: `parallel_history_reuse::gather_facts`. |
//! | Shared gather facts | Sensitive seeded case: `parallel_history_reuse::shared_facts`. |
//! | Typed absence | Waits for slice 6.9: typed prior absence and input-cutoff custody. |
//! | Schema changes | Waits for slice 6.12: schema adoption carrying retained state. |
//! | Program changes | Waits for slice 6.12: program adoption carrying retained state. |
//! | Lineage | Waits for slice 6.11: fork/branch retained-state sharing. |
//! | Restoration | Waits for slice 6.12: retained-state restoration lifecycle events. |
//! | Eviction | Proved at owner boundary: `carrying::another_input_or_edition_or_an_evicted_state_runs_in_full`; no application control directly evicts this computation's state. |
//! | Faults | Varied here: `faults`; aggregate failed-run charges wait for slice 7.7 advancement reports. |
//! | Work ceiling | Proved at owner boundary: `carrying::growing_past_the_ceiling_is_denied_as_a_full_run_is_denied`; declarations are fixed for an installed owner. |
//! | Retained ceiling | Waits for slice 6.12: lifecycle eviction/refusal with a retained prior. |
//! | Placement | Full runs with no prior agree: owner test `certified::every_placement_agrees_and_only_a_certified_one_reduces_again`; reuse across placements waits for 7.8 part two. |
//! | Unobservable reads | Proven at the owner boundary: `unobservable::an_unobservable_key_gathers_its_partition_again_and_denies_nothing`. |
//! | Routing collision | Proven at the owner boundary: `collision::a_collision_met_routing_again_makes_the_run_in_full`. |
//! | Preparation and incremental total charge | Not judged: input/item encodings include the private handler authority, and preparation (including key/routing work) has no separate report. Full compute/tree work is judged through the existing execution observer; incremental compute charges have no separate report on this base. |
//! | Worker count / wave order | Waits for slice 7.8 part two: the leased posture and canonical wave axis. |
//!
//! Owner evidence above is existing coverage, not a claim that this module reruns it.
/// The one declaration of checkpoint units shared by seed, owner, and model.
pub(crate) const SHARED_WORK: u64 = 1;
pub(crate) const ITEM_WORK: [u64; 6] = [2, 3, 5, 7, 11, 13];

#[derive(Clone)]
pub(crate) struct ReuseItem {
    pub number: u64,
    pub key: u32,
    pub value: u64,
    pub work: u64,
    pub member: bool,
    pub exists: bool,
    pub fault: bool,
}
#[derive(Clone)]
pub(crate) struct ReuseFacts {
    pub items: Vec<ReuseItem>,
    pub weights: [u64; 2],
    pub shared_work: u64,
    pub odd: bool,
}
impl ReuseFacts {
    pub(crate) fn members(&self) -> impl Iterator<Item = &ReuseItem> {
        self.items.iter().filter(|item| item.exists && item.member)
    }
    pub(crate) fn shared_partitions(&self) -> u64 {
        self.members()
            .map(|item| item.key)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .filter(|key| key.is_multiple_of(2))
            .count() as u64
    }
    pub(crate) fn expected_value(&self) -> Option<u64> {
        if self.members().any(|item| item.fault) {
            return None;
        }
        let sum = self
            .members()
            .map(|item| item.value * (item.number + 1))
            .sum::<u64>()
            + self.shared_partitions() * self.weights[usize::from(self.odd)];
        Some((sum as f64).to_bits())
    }
    /// Full-run compute charges: declared kernel operations and tree work.
    /// Preparation encoding is not separately observable at this layer.
    pub(crate) fn expected_compute_charge(&self, tree: u64) -> u64 {
        self.members()
            .map(|item| item.work * (item.number + 1))
            .sum::<u64>()
            + self.shared_partitions() * self.shared_work
            + tree
    }
}
