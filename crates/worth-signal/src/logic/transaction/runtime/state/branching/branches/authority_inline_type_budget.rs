//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<SignalForkedBranchState<(), (), ()>>("crates/worth-signal/src/logic/transaction/runtime/state/branching/branches/authority.rs", "SignalForkedBranchState", 128 * 1024);
        owner.field::<BranchState<(), (), ()>>("state");
        owner.field::<crate::data::graph::signal_graph::SignalGraphForkWork>("work");
    }
    {
        let mut owner = census.owner::<SignalBranchPersistentSharing>("crates/worth-signal/src/logic/transaction/runtime/state/branching/branches/authority.rs", "SignalBranchPersistentSharing", 128 * 1024);
        owner.field::<crate::data::graph::signal_graph::SignalGraphPersistentSharing>("graph");
        owner.field::<bool>("config_roots_shared");
        owner.field::<bool>("derived_roots_shared");
    }
    {
        let mut owner = census.owner::<SignalBranchPersistentIdentity<(), (), ()>>("crates/worth-signal/src/logic/transaction/runtime/state/branching/branches/authority.rs", "SignalBranchPersistentIdentity", 128 * 1024);
        owner.field::<crate::data::graph::signal_graph::SignalGraphPersistentIdentity>("graph");
        owner.field::<SignalRuntimeConfig<()>>("config");
        owner.field::<DerivedState<(), ()>>("derived");
        owner.field::<Vec<usize>>("hot_page_identities");
    }
    {
        let mut owner = census.owner::<LatestMergeReference>("crates/worth-signal/src/logic/transaction/runtime/state/branching/branches/authority.rs", "LatestMergeReference", 128 * 1024);
        owner.field::<SignalBranchId>("source_branch_id");
        owner.field::<Option<SignalSnapshotId>>("source_snapshot_id");
        owner.field::<Option<SignalSnapshotId>>("target_snapshot_id_before");
        owner.field::<Option<SignalSnapshotId>>("target_snapshot_id_after");
        owner.field::<BranchMergeKind>("merge_kind");
        owner.field::<BranchMergeStrategy>("merge_strategy");
    }
    {
        let mut owner = census.owner::<BranchAncestryState>("crates/worth-signal/src/logic/transaction/runtime/state/branching/branches/authority.rs", "BranchAncestryState", 128 * 1024);
        owner.field::<SignalBranchId>("branch_id");
        owner.field::<Option<SignalBranchId>>("parent_branch_id");
        owner.field::<Option<SignalSnapshotId>>("forked_from_snapshot_id");
        owner.field::<Option<LatestMergeReference>>("latest_merge_reference");
    }
    {
        let mut owner = census.owner::<BranchState<(), (), ()>>("crates/worth-signal/src/logic/transaction/runtime/state/branching/branches/authority.rs", "BranchState", 128 * 1024);
        owner.field::<AuthorityState<()>>("authority");
        owner.field::<DerivedState<(), ()>>("derived");
        owner.field::<BranchAncestryState>("ancestry");
        owner.field::<BranchMutationLedger>("mutation_ledger");
        owner.field::<Option<
            crate::branch::owner_services::conditional_execution::SignalInstalledDefinitionBinding,
        >>("installed_definition");
    }
}
