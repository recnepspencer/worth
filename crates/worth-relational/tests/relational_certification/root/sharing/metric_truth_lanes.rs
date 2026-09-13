//! The sharing observation's truth-source lanes are not substitutable.
//!
//! Each metric on `RelationalBranchSharingObservation` is documented as being
//! read from exactly one source. This court convicts the documentation: it
//! drives the lanes apart and shows that a metric from one lane cannot be
//! standing in for a metric from another.
//!
//! It deliberately owns no allocation oracle. The physical byte totals are
//! cross-examined against the independent owner-allocation ledger by
//! `root/accounting/authoritative.rs`; this file only proves lane separation,
//! and keeps its own small fork scaffolding so that it stays independent of
//! the fork-slope court next to it.

use super::world::supply_chain::SupplyChainScale;
use super::world::supply_chain::{assert_oracle_matches, certified_supply_chain_world};
use worth_relational::facade::branch::RelationalBranchIdentity;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::inspection::RelationalBranchSharingObservation;
use worth_relational::facade::runtime::RelationalRuntime;

#[test]
fn phase6_selection_lane_is_independent_of_live_owner_storage() {
    let court = observe_seeded_main(SupplyChainScale::court());
    let standard = observe_seeded_main(SupplyChainScale::standard());

    assert_eq!(court.branch_count(), 1);
    assert_eq!(standard.branch_count(), 1);

    assert!(
        standard.unique_physical_authoritative_bytes()
            > court.unique_physical_authoritative_bytes(),
        "the two worlds must hold visibly different authoritative storage"
    );
    assert_eq!(
        court.branch_metadata_bytes(),
        standard.branch_metadata_bytes(),
        "branch metadata is a selection-lane metric: equal selections report \
         equal bytes however much authoritative storage the roots hold"
    );

    for world in [&court, &standard] {
        for (lane, live_bytes) in live_byte_lanes(world) {
            assert_ne!(
                world.branch_metadata_bytes(),
                live_bytes,
                "branch metadata must not be sourced from the live {lane} lane"
            );
        }
    }
}

#[test]
fn phase6_recorded_cost_lane_is_not_substitutable_for_the_live_lane() {
    let (mut world, expected) = certified_supply_chain_world(SupplyChainScale::court());
    assert_oracle_matches(&world, &expected);
    let main = world.runtime.main_branch_identity();
    let main_before_fork = observe(&world.runtime, std::slice::from_ref(&main));

    fork_from_main(&mut world.runtime, "metric-truth-lane-fork");
    let fork = branch_identity(&world.runtime, "metric-truth-lane-fork");
    let main_after_fork = observe(&world.runtime, &[main]);
    let fork_only = observe(&world.runtime, &[fork]);

    // The fork adopted the root outright, so every live lane reports the same
    // storage through either branch.
    assert!(fork_only.unique_physical_authoritative_bytes() > 0);
    assert_eq!(fork_only.unique_root_count(), 1);
    assert_eq!(fork_only.root_ids(), main_after_fork.root_ids());
    assert_eq!(
        fork_only.unique_physical_authoritative_bytes(),
        main_after_fork.unique_physical_authoritative_bytes()
    );
    assert_eq!(
        fork_only.logical_branch_authoritative_bytes(),
        main_after_fork.logical_branch_authoritative_bytes()
    );
    assert_eq!(
        fork_only.visibility_commitments(),
        main_after_fork.visibility_commitments()
    );
    assert_eq!(
        fork_only.region_locators(),
        main_after_fork.region_locators()
    );

    // The recorded-cost lane separates the two branches anyway, because it
    // reports work that was done rather than storage that exists.
    assert!(
        main_after_fork.publication_touched_region_count() > 0,
        "seeding published on main, so main carries recorded publication work"
    );
    assert_eq!(
        fork_only.publication_touched_region_count(),
        0,
        "a fork starts its own recorded lane and inherits none of main's"
    );
    assert_eq!(fork_only.publication_new_authoritative_bytes(), 0);
    assert_eq!(fork_only.copied_truth_bytes(), 0);
    assert_eq!(fork_only.shared_root_acquisitions(), 1);
    assert_eq!(main_after_fork.shared_root_acquisitions(), 0);
    assert_eq!(
        main_after_fork.publication_touched_region_count(),
        main_before_fork.publication_touched_region_count(),
        "forking records cost on the new cell only and never rewrites the source"
    );
    assert_eq!(
        fork_only.reclaimable_unique_bytes(),
        0,
        "no path records reclamation at this milestone"
    );
}

#[test]
fn phase6_coordination_lane_stays_branch_local_under_a_shared_root() {
    let (mut world, expected) = certified_supply_chain_world(SupplyChainScale::court());
    assert_oracle_matches(&world, &expected);
    let main = world.runtime.main_branch_identity();
    fork_from_main(&mut world.runtime, "metric-truth-lane-cell");
    let fork = branch_identity(&world.runtime, "metric-truth-lane-cell");

    let main_only = observe(&world.runtime, std::slice::from_ref(&main));
    let fork_only = observe(&world.runtime, std::slice::from_ref(&fork));
    let both = observe(&world.runtime, &[main, fork]);

    // One shared root, two separate coordination cells.
    assert_eq!(both.unique_root_count(), 1);
    assert_eq!(main_only.coordination_cells().len(), 1);
    assert_eq!(fork_only.coordination_cells().len(), 1);
    assert_eq!(both.coordination_cells().len(), 2);
    assert_ne!(
        main_only.coordination_cells(),
        fork_only.coordination_cells(),
        "branches sharing a root still coordinate through their own cells"
    );

    // The lane is exactly the selected branches' own cells: nothing global,
    // and no unselected branch, contributes to it.
    assert_eq!(
        both.coordination_contacts(),
        main_only.coordination_contacts() + fork_only.coordination_contacts()
    );
    assert_eq!(
        both.coordination_waits(),
        0,
        "branch-local publication never contends on the selected cells"
    );
}

fn observe(
    runtime: &RelationalRuntime,
    branches: &[RelationalBranchIdentity],
) -> RelationalBranchSharingObservation {
    runtime
        .observe_branch_sharing(branches)
        .expect("the selected branches remain inspectable")
}

fn observe_seeded_main(scale: SupplyChainScale) -> RelationalBranchSharingObservation {
    let (world, expected) = certified_supply_chain_world(scale);
    assert_oracle_matches(&world, &expected);
    let main = world.runtime.main_branch_identity();
    observe(&world.runtime, &[main])
}

fn live_byte_lanes(observation: &RelationalBranchSharingObservation) -> [(&'static str, u64); 6] {
    [
        (
            "logical_branch_root_metadata_bytes",
            observation.logical_branch_root_metadata_bytes(),
        ),
        (
            "unique_physical_root_metadata_bytes",
            observation.unique_physical_root_metadata_bytes(),
        ),
        (
            "logical_branch_partition_payload_bytes",
            observation.logical_branch_partition_payload_bytes(),
        ),
        (
            "unique_physical_partition_payload_bytes",
            observation.unique_physical_partition_payload_bytes(),
        ),
        (
            "logical_branch_authoritative_bytes",
            observation.logical_branch_authoritative_bytes(),
        ),
        (
            "unique_physical_authoritative_bytes",
            observation.unique_physical_authoritative_bytes(),
        ),
    ]
}

fn fork_from_main(runtime: &mut RelationalRuntime, name: &str) {
    let (_, source_basis) = runtime
        .observe_fork_source(&BranchId("main".to_owned()))
        .expect("main remains forkable");
    runtime
        .fork_branch(BranchId(name.to_owned()), source_basis)
        .expect("fork remains metadata-only");
}

fn branch_identity(runtime: &RelationalRuntime, name: &str) -> RelationalBranchIdentity {
    runtime
        .branch_identity(&BranchId(name.to_owned()))
        .expect("branch identity is owner-issued")
}
