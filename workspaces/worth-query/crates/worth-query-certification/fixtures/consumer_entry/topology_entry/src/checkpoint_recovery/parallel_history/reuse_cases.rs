//! Complete reuse-decision inventory. Sealed reuse-off preparation is the
//! control; a fresh installation is a second actual value/charge run.
//!
//! | Input | Reachability / evidence |
//! |---|---|
//! | Code / computation identity | `parallel_history_reuse` installs two distinct declared computation identities in one application; switching code carries no partition, across prior/current widths. Owner reinstall is also proved by `installation::another_owner_with_the_same_types_runs_in_full`. |
//! | Reducer installation | Proved at owner boundary: `installation::a_reinstalled_reducer_runs_in_full_with_its_own_reduction`. |
//! | Owner installation | Proved at owner boundary: `installation::a_reinstalled_owner_runs_in_full`. |
//! | Producer edition | Proved at owner boundary: `partitioned_computation::incremental::tests::carrying::another_input_or_edition_or_an_evicted_state_runs_in_full`; program adoption preserves the installed edition and is driven by `courtroom::adoption`. |
//! | Canonical input | Sensitive seeded case: `parallel_history_reuse::canonical_input`. |
//! | Membership facts | Varied here: `membership_join`, `membership_leave`, `membership_replacement`. |
//! | Empty membership | Matrix cells cover empty-to-member, member-to-empty, and empty-to-empty input replacement; before/after kernel counts (0/1, 1/0, 0/0) and actual retained-prior presence are judged independently of run labels. |
//! | Item digest | Varied here: `persisting_item_digest_swap`; digest drift on persisting item ids changes their declared operations. |
//! | Item key facts | Sensitive seeded case: `parallel_history_reuse::key_facts`. |
//! | Partition gather facts | Sensitive seeded case: `parallel_history_reuse::gather_facts`. |
//! | Shared gather facts | Sensitive seeded case: `parallel_history_reuse::shared_facts`. |
//! | Typed absence | `computation_partition::oracle::absence::checkpoint_absence_rebuilds_every_owner_call_like_fresh` checks genuine restore absence against an independent fresh computation and exact owner calls. |
//! | Schema changes | Unreachable with retained computation state: the runtime keeps its installed schema fixed, and restoration reconstructs lineage with an absent computation prior. |
//! | Program changes | Proved by `computation_partition::oracle::courtroom::adoption`; edits under the adopted program preserve the owner, with model-derived partition calls. |
//! | Lineage | Proved by `computation_partition::oracle::branch_sharing` and `courtroom::branches`, with independent model counts on forks, nested forks, switches and ancestor deletion. |
//! | Merge | Product history has no merge entry; native unique-value merge lookup is proved by `merge_unique_values::counts::merge_unique_lookup_count_is_bounded_by_its_writes`. |
//! | Recorded correction | `courtroom::undo::recorded_correction_reuses_the_retained_partitions` proves accepted correction against independent fresh execution and model calls. `undo::after_publication` proves successful correction after a real output publication and SourceDiverged refusal after an intervening source-value edit. |
//! | Commit racing settlement | `courtroom::interleaving::another_writer_between_passes_leaves_pending_after_one_contact` constructs and asserts another publication between managed passes. |
//! | Deep scope hierarchy and sibling-disjoint subtrees | `scope_hierarchy::a_deep_native_leaf_has_identical_work_with_eight_or_sixty_four_sibling_subtrees` proves six ancestor scopes, one marked fact, five downstream edges, and zero sibling candidate/Ready work at 8 and 64 sibling subtrees. |
//! | Restoration | Proved by `computation_partition::oracle::courtroom::lifecycle::seeded_edits_cross_restore_and_republication_with_model_counts`, with restored and republished exact causes and model-derived full-build calls. |
//! | Faults | `faults` and every prior/current width cell pin the canonical typed stop, exact changed/carried partition counts, and whole advancement work/span. |
//! | Work ceiling | Proved at owner boundary: `carrying::growing_past_the_ceiling_is_denied_as_a_full_run_is_denied`; declarations are fixed for an installed owner. |
//! | Placement | `parallel_history_reuse::worker_matrix` judges prior/current widths 1/2/5 with observed dispatch widths; serial-versus-leased prior handoff remains named for 7.6 part two. |
//! | Unobservable reads | Proven at the owner boundary: `unobservable::an_unobservable_key_gathers_its_partition_again_and_denies_nothing`. |
//! | Routing collision | Proven at the owner boundary: `collision::a_collision_met_routing_again_makes_the_run_in_full`. |
//! | Native request locality | `required_chain::exact_invalidation::locality` retains the one/ten/hundred authored journey, exact producer and source-query contacts, native marking, fact verification and whole-registry visits. It supplies no work or retained-byte estimates. |
//! | Other whole-map lifecycle walks | `lifecycle/closed_retirement` builds claimant indexes on closed superseded-row retirement; occurrence retirement releases an occurrence on branch retirement. Both are outside the measured request path. |
//! | Native occurrence helpers | `refreshed_rejoin`, `succession`, `settlement_retirement` and `supersession::reject_older_successor` retain exact occurrence identity and currentness. |
//! | Observer scan | `ready_backing::has_required_row_for_test` is a test-only custody assertion, not a request traversal. |
//! | Cold source supersession | `supersession::supersede_predecessors` uses occurrence seeks and keyed removals on ordinary and performed first admission. `source_custody::retire_stale_records` is triggered by discovered-source binding and uses the maintained source-coordinate index across producers. `source_custody/retention` and completed-custody cleanup use that same index; the cold request oracle convicts their former global probes. |
//! | Preparation and incremental total charge | Declared full-run work and managed-result work are equal with reuse on/off. Whole advancement work/span remainders are compared across every posture per input family; no attribution is invented for private authority encoding or preparation. |
//! | Worker count / wave order | The core matrix permutes demand starts, declarations and source edits; dispatch seeds are owned-unit evidence, not matrix rows; checks typed outcomes, canonical kept prefix and exact shared work/span remainders, with actual worker identities bounded by widths 1/2/5. |
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
        if self.members().next().is_none() {
            // The declared canonical reducer's identity is -0.0; no kernel
            // or combine runs when there are no partition members.
            return Some((-0.0f64).to_bits());
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
