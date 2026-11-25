//! A tracked name read records exactly what its answer rested on: a later
//! commit that splits, merges, retires or births a name the read depended on
//! stales it, and the re-evaluated read sees the change; unrelated commits
//! leave it clean.

use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_naming::{
    NameResolutionDenial, NameSelection, ResolvedName,
};
use worth_query_execution_naming::facade::{
    ExecutionNameResolveDenial, NameReadCurrentness, NameReadDependencies,
};

use crate::naming_execution_fixture::{execution_lease_for_workers, CadNamingSchema, Face};
use crate::naming_tracked_world::{NamedFace, Rebirth, TrackedWorld};

fn is_stale(currentness: &NameReadCurrentness) -> bool {
    matches!(currentness, NameReadCurrentness::Stale(_))
}

#[test]
fn lineage_changes_stale_only_the_reads_that_rested_on_them() {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let world = TrackedWorld::new();
    let resolve = |held: &NamedFace, deps: &mut NameReadDependencies| {
        let head = world.head();
        world
            .reader(&execution, &head)
            .resolve_entity::<CadNamingSchema, Face>(&held.name, &NameSelection::Exact, deps)
    };

    let (seeded, _) = world.seed(&execution, "occ.seed", &["f1", "f2"]);
    let (f1, f2) = (seeded[0], seeded[1]);
    let mut on_f1 = NameReadDependencies::new();
    assert_eq!(resolve(&f1, &mut on_f1), Ok(ResolvedName::Exact(f1.id)));
    let mut on_f2 = NameReadDependencies::new();
    let head = world.head();
    world
        .reader(&execution, &head)
        .entity_name_of::<CadNamingSchema, Face>(f2.id, &mut on_f2)
        .expect("f2 is named");

    let (_, unrelated) = world.seed(&execution, "occ.unrelated", &["u1"]);
    assert_eq!(
        on_f1.currentness_after_commit(&unrelated),
        NameReadCurrentness::Clean
    );
    assert_eq!(
        on_f2.currentness_after_commit(&unrelated),
        NameReadCurrentness::Clean
    );

    let (pieces, split) = world.split(&execution, f1, "occ.cut", &[("a", 1), ("b", 2)]);
    let (a, b) = (pieces[0], pieces[1]);
    let NameReadCurrentness::Stale(staleness) = on_f1.currentness_after_commit(&split) else {
        panic!("splitting the resolved name stales the read");
    };
    assert_eq!(staleness.lineages.len(), 1, "only f1's own lineage ended");
    assert!(staleness.births.is_empty());
    assert_eq!(
        on_f2.currentness_after_commit(&split),
        NameReadCurrentness::Clean
    );
    let mut reevaluated = NameReadDependencies::new();
    assert!(
        matches!(
            resolve(&f1, &mut reevaluated),
            Err(ExecutionNameResolveDenial::Resolution(
                NameResolutionDenial::Ambiguous { .. }
            ))
        ),
        "re-evaluating the stale read sees the split"
    );

    let mut on_b = NameReadDependencies::new();
    resolve(&b, &mut on_b).expect("b is live");
    let mut on_a = NameReadDependencies::new();
    resolve(&a, &mut on_a).expect("a is live");
    let merge = world.merge_keeping(&execution, a, b);
    assert!(
        is_stale(&on_b.currentness_after_commit(&merge)),
        "merging b away stales it"
    );
    assert!(
        is_stale(&reevaluated.currentness_after_commit(&merge)),
        "f1's frontier changed"
    );
    assert_eq!(
        on_a.currentness_after_commit(&merge),
        NameReadCurrentness::Clean,
        "the keeper continues its record and name"
    );

    let retire = world.retire(&execution, f2);
    assert!(
        is_stale(&on_f2.currentness_after_commit(&retire)),
        "retiring f2 stales its name read"
    );
    assert_eq!(
        on_a.currentness_after_commit(&retire),
        NameReadCurrentness::Clean
    );

    // Names are minted from declared provenance, so another runtime can hold
    // one this runtime has not born yet: the read records the awaited birth.
    let elsewhere = TrackedWorld::new();
    let (future, _) = elsewhere.seed(&execution, "occ.future", &["later"]);
    let mut on_future = NameReadDependencies::new();
    assert_eq!(
        resolve(&future[0], &mut on_future),
        Err(ExecutionNameResolveDenial::Resolution(
            NameResolutionDenial::UnknownName
        ))
    );
    let (_, unrelated_again) = world.seed(&execution, "occ.other", &["later"]);
    assert_eq!(
        on_future.currentness_after_commit(&unrelated_again),
        NameReadCurrentness::Clean
    );
    let (born, birth) = world.seed(&execution, "occ.future", &["later"]);
    let NameReadCurrentness::Stale(staleness) = on_future.currentness_after_commit(&birth) else {
        panic!("birthing the awaited name stales the read");
    };
    assert_eq!(staleness.births, vec![*future[0].name.as_bytes()]);
    assert_eq!(
        born[0].name, future[0].name,
        "provenance mints the same name in both runtimes"
    );
}

#[test]
fn restoring_an_ended_name_stales_the_read_that_walked_through_it() {
    let lease = execution_lease_for_workers(2);
    let execution = ExecutionRequest::leased(&lease);
    let world = TrackedWorld::new();
    let (seeded, _) = world.seed(&execution, "occ.restore", &["s", "t"]);
    let (s, t) = (seeded[0], seeded[1]);
    let retired = world.retire(&execution, s);
    let mut on_s = NameReadDependencies::new();
    let head = world.head();
    let answer = world
        .reader(&execution, &head)
        .resolve_entity::<CadNamingSchema, Face>(&s.name, &NameSelection::Exact, &mut on_s);
    assert_eq!(
        answer,
        Ok(ResolvedName::Retired {
            at_commit: retired.commit.commit_id.0
        })
    );
    assert!(!on_s
        .ended_lineages_on(head.identity().branch_id())
        .is_empty());
    let (_, unrelated) = world.rebirth(
        &execution,
        t,
        ("occ.restore", "t", "t2"),
        Rebirth::Regenerate,
    );
    assert_eq!(
        on_s.currentness_after_commit(&unrelated),
        NameReadCurrentness::Clean
    );
    let (restored, restore) =
        world.rebirth(&execution, s, ("occ.restore", "s", "s2"), Rebirth::Restore);
    assert!(
        is_stale(&on_s.currentness_after_commit(&restore)),
        "restoring s stales its read"
    );
    let head = world.head();
    assert_eq!(
        world
            .reader(&execution, &head)
            .resolve_entity::<CadNamingSchema, Face>(
                &s.name,
                &NameSelection::Exact,
                &mut NameReadDependencies::new()
            ),
        Ok(ResolvedName::Exact(restored.id)),
        "re-evaluation sees the restored record"
    );
}
