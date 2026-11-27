//! Branches over named outputs (persistent naming Phase 7, Arch 5, 10).
//!
//! Query offers no producer-rerunning branch merge: an application moves
//! its slot only by adopting a candidate forked from the slot's current
//! head. Two branches that each touch named outputs therefore never merge
//! silently: the second adoption refuses typed with zero effects, each
//! branch keeps resolving a colliding identity to its own holder, and a
//! name born on one branch is unknown on the other. Native branch merge
//! refuses named outputs at preparation (Relational certification
//! `named_selection::branches`).
use super::outputs::sculpt::{Intent, Plan, Step};
use super::outputs::{birth, keys, read, resolution, resolved_to, Outcome, World};
use super::probe::Observation;
use super::*;
use worth_query_host::facade::product::{
    WorthQueryExpectedProductBranch, WorthQueryExpectedProductBranchAdoptionDenial as Denial,
    WorthQueryProductBranch,
};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation;

fn fork(world: &World) -> WorthQueryExpectedProductBranch<'_, Schema> {
    let host = &world.host;
    host.with_host_advancement(|phase| {
        let selected = host.on_branch(host.current_world()).select(&phase).unwrap();
        host.branches()
            .fork_expected(selected)
            .unwrap()
            .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
            .create(&phase)
            .unwrap()
    })
    .unwrap()
}

fn sculpt_on(world: &World, branch: WorthQueryProductBranch, token: &str, plan: Plan) {
    super::outputs::sculpt::author(token, plan);
    let outcome = world
        .request()
        .on_branch(branch)
        .advanced()
        .mutate(Intent {
            token: token.to_owned(),
        })
        .without_source()
        .idempotency(&format!("sculpt:{token}"))
        .execute_in_program(&world.host, SystemAllocation);
    assert!(
        matches!(outcome, Ok(Outcome::Committed { .. })),
        "{token}: {outcome:?}"
    );
}

fn probe_on(
    world: &World,
    branch: WorthQueryProductBranch,
    token: &str,
    keys: &[&str],
) -> Vec<Observation> {
    let _ = world
        .request()
        .on_branch(branch)
        .advanced()
        .mutate(probe::Intent {
            token: token.to_owned(),
            keys: super::outputs::keys(keys),
            value: 400,
        })
        .without_source()
        .idempotency(&format!("probe:{token}:0"))
        .execute_in_program(&world.host, SystemAllocation);
    probe::take(token)
}

fn born(key: &str, occurrence: &str) -> Plan {
    Plan {
        creates: keys(&[key]),
        steps: vec![Step::Birth(key.into(), birth(occurrence, "authored"))],
        ..Plan::default()
    }
}

#[test]
fn diverged_named_branches_never_merge_or_rebind_silently() {
    let world = World::install();
    let root = world.host.current_world();
    world.committed("br-shared", born("br-shared", "br-shared"));
    let shared = world.names("br-shared-name", &["br-shared"], 0)[0];
    let (left, right) = (fork(&world), fork(&world));
    let (a, b) = (left.branch(), right.branch());
    // The same authored occurrence on both branches: one colliding identity.
    sculpt_on(&world, a, "br-a-born", born("br-a", "br-same"));
    sculpt_on(&world, b, "br-b-born", born("br-b", "br-same"));
    let retire = Plan {
        reads: keys(&["br-shared"]),
        deletes: keys(&["br-shared"]),
        steps: vec![Step::Retire(read("br-shared"))],
        ..Plan::default()
    };
    sculpt_on(&world, b, "br-b-retire", retire);

    let mut slot = root;
    let adopt = |candidate: WorthQueryExpectedProductBranch<'_, Schema>, slot: &mut _| {
        world
            .host
            .with_host_advancement(|phase| {
                candidate.adopt(slot, &phase).map_err(|f| f.into_parts().1)
            })
            .unwrap()
    };
    assert_eq!(adopt(left, &mut slot).expect("a fast-forward adopts"), root);
    assert_eq!(slot, a);
    let refused = adopt(right, &mut slot).expect_err("a diverged branch never merges");
    assert!(
        matches!(
            refused,
            Denial::HeadMoved | Denial::HeadReplaced | Denial::SourceBasis(_)
        ),
        "{refused:?}"
    );
    assert_eq!(slot, a, "the refused adoption leaves zero effects");

    let same = |branch, token: &str, key: &str| {
        let observed = probe_on(&world, branch, token, &[key]);
        super::outputs::named(&observed, key)
    };
    let collided = same(a, "br-name-a", "br-a");
    assert_eq!(
        same(b, "br-name-b", "br-b"),
        collided,
        "one colliding identity"
    );
    for (branch, token, holder, shared_live) in
        [(a, "br-on-a", "br-a", true), (b, "br-on-b", "br-b", false)]
    {
        probe::hold(token, "same", collided);
        probe::hold(token, "shared", shared);
        let observed = probe_on(&world, branch, token, &[holder, "br-shared"]);
        assert_eq!(resolution(&observed, "same"), &resolved_to("same", holder));
        let expected = if shared_live {
            resolved_to("shared", "br-shared")
        } else {
            Observation::Retired {
                label: "shared".into(),
            }
        };
        assert_eq!(resolution(&observed, "shared"), &expected);
    }
    probe::hold("br-on-root", "same", collided);
    let observed = probe_on(&world, root, "br-on-root", &[]);
    assert!(
        matches!(resolution(&observed, "same"), Observation::ResolveRefused { refusal, .. } if refusal.contains("UnknownName")),
        "a name born only on branches is unknown on the root: {observed:?}"
    );
}
