//! Complete reuse-decision inventory. A fresh installation is the reuse-off
//! control; it is an actual run, never the expected-value reference.
//!
//! | Input | Reachability / evidence |
//! |---|---|
//! | Code / computation identity | Proved at owner boundary: `installation::another_owner_with_the_same_types_runs_in_full`; applications cannot carry retained state between installations. |
//! | Reducer installation | Proved at owner boundary: `installation::a_reinstalled_reducer_runs_in_full_with_its_own_reduction`. |
//! | Owner installation | Proved at owner boundary: `installation::a_reinstalled_owner_runs_in_full`. |
//! | Producer edition | Proved at owner boundary: `carrying::another_input_or_edition_or_an_evicted_state_runs_in_full`; program adoption preserves the installed edition and is driven by `courtroom::adoption`. |
//! | Canonical input | Sensitive seeded case: `parallel_history_reuse::canonical_input`. |
//! | Membership facts | Varied here: `membership_join`, `membership_leave`, `membership_replacement`. |
//! | Item digest | Varied here: `persisting_item_digest_swap`; digest drift on persisting item ids changes their declared operations. |
//! | Item key facts | Sensitive seeded case: `parallel_history_reuse::key_facts`. |
//! | Partition gather facts | Sensitive seeded case: `parallel_history_reuse::gather_facts`. |
//! | Shared gather facts | Sensitive seeded case: `parallel_history_reuse::shared_facts`. |
//! | Typed absence | Proved by `computation_partition::oracle::seeded_absence` and input-cutoff custody; the seeded courtroom asserts exact full-build calls. |
//! | Schema changes | Unreachable with retained computation state: the runtime keeps its installed schema fixed, and restoration reconstructs lineage with an absent computation prior. |
//! | Program changes | Proved by `computation_partition::oracle::courtroom::adoption`; edits under the adopted program preserve the owner, with model-derived partition calls. |
//! | Lineage | Proved by `computation_partition::oracle::branch_sharing` and `courtroom::branches`, with independent model counts on forks, nested forks, switches and ancestor deletion. |
//! | Merge | Product history has no merge entry; native unique-value merge lookup is proved by `merge_unique_values::counts::merge_unique_lookup_count_is_bounded_by_its_writes`. |
//! | Undo | `courtroom::undo::undo_directly_on_its_target_reuses_the_retained_partitions` and `undo::redo` prove direct-target undo and immediate redo; `undo::after_publication` proves terminal ProviderRejected / InvariantExecution after publication moves the parent; `undo::owner_refusals` proves eight program-owner refusal cases. |
//! | Commit racing settlement | `courtroom::interleaving::another_writer_between_passes_leaves_pending_after_one_contact` constructs and asserts another publication between managed passes. |
//! | Deep scope hierarchy and sibling-disjoint subtrees | `scope_hierarchy::a_deep_native_leaf_has_identical_work_with_eight_or_sixty_four_sibling_subtrees` proves six ancestor scopes, one marked fact, five downstream edges, and zero sibling candidate/Ready work at 8 and 64 sibling subtrees. |
//! | Restoration | Proved by `computation_partition::oracle::courtroom::lifecycle`, with restored and republished exact causes and model-derived full-build calls. |
//! | Eviction | Proved at owner boundary: `carrying::another_input_or_edition_or_an_evicted_state_runs_in_full`; no application control directly evicts this computation's state. |
//! | Faults | Varied here: `faults`; aggregate failed-run charges wait for slice 7.7 advancement reports. |
//! | Work ceiling | Proved at owner boundary: `carrying::growing_past_the_ceiling_is_denied_as_a_full_run_is_denied`; declarations are fixed for an installed owner. |
//! | Retained ceiling | Proved by `computation_partition::oracle::seeded_absence` and `branch_sharing::eviction`; refused child state forces a model-derived full build while the parent still reuses. |
//! | Placement | Full runs with no prior agree: owner test `certified::every_placement_agrees_and_only_a_certified_one_reduces_again`; reuse across placements waits for 7.8 part two. |
//! | Unobservable reads | Proven at the owner boundary: `unobservable::an_unobservable_key_gathers_its_partition_again_and_denies_nothing`. |
//! | Routing collision | Proven at the owner boundary: `collision::a_collision_met_routing_again_makes_the_run_in_full`. |
//! | Request work | `courtroom::request_work::{settled_rows,first_admission}` judges admission, advance, settlement and release for ordinary and performed requests that fit their custody. First-admission populations differ by 1000; settled-row populations cross the ledger's first level with multiple rows on a disjoint triangle; whole-map visits are exactly zero and logical charge is equal. Unrelated means outside the selected branch and scope; key-payload comparison in the selected index is logical work. Navigation is bounded by operations times H. |
//! | Other whole-map lifecycle walks | `lifecycle/closed_retirement` builds claimant indexes on closed superseded-row retirement; occurrence retirement releases an occurrence on branch retirement. Both are outside the measured request path. |
//! | Other global custody questions | Reached from a request only under capacity pressure: `source_readmission` -> `lifecycle/cached_reclaim` charges cached rows examined; `required_custody/caller_chain` asks who can release custody after retention refusal, charging its rows/chain comparisons. Its work exhaustion remains a Terminal WorkBudgetExceeded stop; it cannot invent retryability. |
//! | Occurrence helpers without admission | `refreshed_rejoin::{awaited_by_stale_owner,replaced_under_refresh,replaced_by_published}`, `succession`, `settlement_retirement`, and `supersession::reject_older_successor` use only an occurrence's bounded run where no admission is in scope. They are independent of unrelated population and stay uncharged. |
//! | Observer scan | `ready_backing::has_required_row_for_test` is a test-only custody assertion, not a request traversal. |
//! | Cold source supersession | `supersession::supersede_predecessors` uses occurrence seeks and keyed removals on ordinary and performed first admission. `source_custody::retire_stale_records` is triggered by discovered-source binding and uses the maintained source-coordinate index across producers. `source_custody/retention` and completed-custody cleanup use that same index; the cold request oracle convicts their former global probes. |
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
