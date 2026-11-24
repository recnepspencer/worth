//! `#314`: an installed operation merges two named members, keeping one in
//! place, while the kept member lives in a partition the merging commit
//! never touches. Relational must find the keeper live at the commit basis,
//! keep it in place, and resolve both held names to it; a keeper whose name
//! is no longer live still refuses typed with zero effects.
use super::outputs::sculpt::{self, Held, Keeper, Plan, Step};
use super::outputs::*;
use super::*;
use worth_query_decl::facade::application_naming::{NameSelection, WorthQueryName};

/// The partition the live member keyed `key` is stored in.
fn partition_of(world: &World, key: &str) -> u32 {
    world::member_identity(&world.host, world.host.current_world(), key)
        .unwrap_or_else(|| panic!("`{key}` must be live"))
        .0
}

/// Names `kept` in an issued partition and `absorbed` in model `a`'s.
fn born_apart(world: &World, token: &str) -> (WorthQueryName<Member>, WorthQueryName<Member>) {
    world.committed(
        &format!("{token}-kept-born"),
        Plan {
            issued: true,
            creates: keys(&["kept"]),
            steps: vec![Step::Birth("kept".into(), birth("keeper", "authored"))],
            ..Plan::default()
        },
    );
    world.committed(
        &format!("{token}-absorbed-born"),
        Plan {
            creates: keys(&["absorbed"]),
            steps: vec![Step::Birth(
                "absorbed".into(),
                birth("absorbed", "authored"),
            )],
            ..Plan::default()
        },
    );
    assert_ne!(
        partition_of(world, "kept"),
        partition_of(world, "absorbed"),
        "the keeper must live outside the partition the merge touches"
    );
    let observed = world.probe(token, &["kept", "absorbed"], 0);
    (named(&observed, "kept"), named(&observed, "absorbed"))
}

fn merge_keeping(kept: Held) -> Plan {
    Plan {
        reads: keys(&["absorbed"]),
        deletes: keys(&["absorbed"]),
        steps: vec![Step::Merge(
            vec![kept.clone(), read("absorbed")],
            Keeper::InPlace(kept),
        )],
        ..Plan::default()
    }
}

#[test]
fn keep_parent_in_place_merges_across_an_untouched_partition() {
    let world = World::install();
    let (kept, absorbed) = born_apart(&world, "lineage-apart");
    let identity = world::member_identity(&world.host, world.host.current_world(), "kept");
    world.committed("lineage-merge-apart", merge_keeping(Held::Name(kept)));
    assert_eq!(
        world::member_identity(&world.host, world.host.current_world(), "kept"),
        identity,
        "the keeper stays in place: the same record, untouched"
    );
    assert!(!world.live("absorbed"));
    for selection in [NameSelection::Exact, NameSelection::AllDescendants] {
        let token = format!("lineage-merged-{selection:?}");
        probe::hold_selecting(&token, "kept", kept, selection.clone());
        probe::hold_selecting(&token, "absorbed", absorbed, selection);
        let observed = world.probe(&token, &["kept"], 1);
        assert_eq!(named(&observed, "kept"), kept, "the keeper keeps its name");
        assert_eq!(resolution(&observed, "kept"), &resolved_to("kept", "kept"));
        assert_eq!(
            resolution(&observed, "absorbed"),
            &resolved_to("absorbed", "kept"),
            "the absorbed name follows to the untouched keeper"
        );
    }
}

#[test]
fn untouched_keeper_whose_name_ended_refuses_with_zero_effects() {
    let world = World::install();
    let (kept, _) = born_apart(&world, "lineage-ended");
    world.committed(
        "lineage-retire-kept",
        Plan {
            reads: keys(&["kept"]),
            deletes: keys(&["kept"]),
            steps: vec![Step::Retire(read("kept"))],
            ..Plan::default()
        },
    );
    let before = world::member_identity(&world.host, world.host.current_world(), "absorbed");
    let outcome = world.try_sculpt("lineage-merge-ended", merge_keeping(Held::Name(kept)));
    let refused = match &outcome {
        Ok(outcome) => naming_refusal(outcome).is_some() || output_refusal(outcome).is_some(),
        Err(_) => sculpt::authoring_refusal("lineage-merge-ended").is_some(),
    };
    assert!(
        refused,
        "a keeper whose name ended refuses typed: {outcome:?}"
    );
    assert_eq!(
        world::member_identity(&world.host, world.host.current_world(), "absorbed"),
        before,
        "the refused merge leaves zero effects"
    );
}
