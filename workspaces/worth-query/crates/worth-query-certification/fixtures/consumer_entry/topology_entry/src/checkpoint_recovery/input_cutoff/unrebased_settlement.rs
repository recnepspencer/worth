//! A commit whose facts could not be rebased retains none of them.

use super::*;
use worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
    WorthQueryApplicationRelationSeed, WorthQueryPrimaryGraphBootstrap,
};

/// A body whose profile kind only the alternate producer serves, and that
/// producer declares the Initial posture alone.
const RESOLVED: &str = "manual-a";

/// The same kind under the key family whose commit joins the bodies its
/// decision surveyed at exactly their number: the rebase of that commit
/// fails, and its effect moves none of its reads.
const SURVEYED: &str = "manual-wide-a";

/// A body its own commit raises: the effect moves the height its source
/// query read. The commit rebases.
const RAISED: &str = "manual-raised-a";

/// The same body surveyed: the effect moves its own read and the rebase
/// fails.
const SURVEYED_RAISED: &str = "manual-wide-raised-a";

/// A surveyed body of the planar kind, whose producer also preserves.
const PRESERVED: &str = "surveyed-a";

/// The demand that committed settles on its own publication. The row keeps
/// no facts, so it is no candidate: the next demand asks for the Preserve
/// posture, and this kind has no producer for it.
#[test]
fn a_commit_whose_facts_cannot_be_rebased_settles_and_leaves_no_candidate() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install_manual_bodies();
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);

    let performed = direct_demand(&request, &application, SURVEYED);
    assert_eq!(
        performed.posture(),
        WorthQueryOutputSettlementPosture::Performed
    );
    assert_eq!(performed.producer_contacts_in_this_demand(), 1);
    assert!(performed.application_commit_receipt().is_some());
    drop(performed);

    for _ in 0..2 {
        let before = request.retain_read().unwrap();
        let denial = demand_outcome(&request, &application, SURVEYED)
            .err()
            .expect("a row without facts is not reused");
        assert!(
            matches!(&denial, WorthQueryApplicationOutputDemandDenial::Demand(cause)
                if cause.kind() == WorthQueryOutputDemandDenialKind::MissingApplicableProducer),
            "{denial:?}"
        );
        let after = request.retain_read().unwrap();
        assert_eq!(before.selected_commit(), after.selected_commit());
    }
}

/// The same producer without the survey: its commit rebases, the row keeps
/// its facts, and every later demand reuses it with no producer contact.
#[test]
fn a_commit_whose_facts_rebase_is_reused_where_no_producer_preserves() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install_manual_bodies();
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);

    let performed = direct_demand(&request, &application, RESOLVED);
    assert_eq!(performed.producer_contacts_in_this_demand(), 1);
    drop(performed);

    for _ in 0..2 {
        let before = request.retain_read().unwrap();
        let reused = direct_demand(&request, &application, RESOLVED);
        assert_eq!(reused.producer_contacts_in_this_demand(), 0);
        drop(reused);
        let after = request.retain_read().unwrap();
        assert_eq!(before.selected_commit(), after.selected_commit());
    }
}

/// A commit whose effect moved one of its own reads is superseded at its own
/// publication. The commit that rebased kept the fact and reads it stale; the
/// one that could not rebase carries the same answer out with no fact. Both
/// land, refresh, and are refused the Preserve producer this kind lacks.
#[test]
fn a_commit_that_moved_its_own_read_refreshes_whether_or_not_it_rebased() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install_manual_bodies();
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);

    let demanded = |root: &str| {
        let before = request.retain_read().unwrap();
        let outcome = demand_outcome(&request, &application, root);
        let after = request.retain_read().unwrap();
        let landed = before.selected_commit() != after.selected_commit();
        let standing = match outcome {
            Ok(settled) => format!("{:?}", settled.posture()),
            Err(WorthQueryApplicationOutputDemandDenial::Demand(cause)) => {
                format!("{:?}", cause.kind())
            }
            Err(other) => format!("{other:?}"),
        };
        (landed, standing)
    };
    let rebased = demanded(RAISED);
    assert_eq!(
        rebased,
        (true, "MissingApplicableProducer".to_owned()),
        "the rebased commit is born stale and refreshes"
    );
    assert_eq!(
        demanded(SURVEYED_RAISED),
        rebased,
        "a failed rebase answers as the rebase that succeeded"
    );
}

/// The same failure where a producer preserves. The commit settles its own
/// demand and each demand after it while its publication is the one
/// selected: no producer runs and nothing lands. Any later publication, here
/// one for an unrelated body, supersedes it. The next demand runs the
/// Preserve producer, whose commit adds no body to the ones it surveyed,
/// rebases, and is reused from then on.
#[test]
fn a_failed_rebase_settles_until_a_later_publication_where_a_producer_preserves() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install_manual_bodies();
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);

    // Whether the demand landed a commit, and its producer contacts.
    let demanded = || {
        let before = request.retain_read().unwrap();
        let settled = direct_demand(&request, &application, PRESERVED);
        let contacts = settled.producer_contacts_in_this_demand();
        drop(settled);
        let after = request.retain_read().unwrap();
        (
            before.selected_commit() != after.selected_commit(),
            contacts,
        )
    };
    assert_eq!(demanded(), (true, 1), "the commit whose rebase fails");
    for _ in 0..2 {
        assert_eq!(demanded(), (false, 0), "its own publication is selected");
    }

    drop(direct_demand(&request, &application, RESOLVED));
    assert_eq!(demanded(), (true, 1), "superseded, the row is preserved");
    for _ in 0..2 {
        assert_eq!(demanded(), (false, 0), "the commit that rebased is reused");
    }
}

fn demand_outcome<'application, 'principal, 'scope>(
    request: &worth_query_host::facade::application_entry::WorthQueryApplicationRequest<
        'application,
        'principal,
        'scope,
        CheckpointSchema,
    >,
    application: &'application support::Application,
    root: &str,
) -> Result<
    WorthQueryApplicationOutputDemandSettlement<PlanarQuery>,
    WorthQueryApplicationOutputDemandDenial,
> {
    let mut demand = request
        .demand(PlanarOutputDemand::new(root))
        .controls(controls())
        .start_in_program::<CheckpointProgram, CheckpointRoot>(application)?;
    for _ in 0..256 {
        match demand.advance(request)? {
            WorthQueryApplicationOutputDemandProgress::Pending => std::thread::yield_now(),
            WorthQueryApplicationOutputDemandProgress::Settled(settled) => return Ok(settled),
        }
    }
    panic!("the demand did not end within its synchronous bound")
}

fn install_manual_bodies() -> support::Application {
    let source_work =
        worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile::standard()
            .limits()
            .source_currentness_work();
    support::install_program_with_seed::<CheckpointProgram>(
        None,
        Default::default(),
        32,
        128 * 1_024 * 1_024,
        u64::try_from(source_work).unwrap(),
        seed,
    )
}

fn seed(graph: &mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>) {
    let key = |name: &str| {
        WorthQueryApplicationEntityKey::<CheckpointSchema, Body>::new(name.to_owned()).unwrap()
    };
    let body = |name: &str, x, y, standing| {
        WorthQueryApplicationEntitySeed::new(Body::reference::<CheckpointSchema>(), key(name))
            .field(BodyKey::reference::<CheckpointSchema>(), name.to_owned())
            .field(Length::reference::<CheckpointSchema>(), length(standing))
            .field(PositionX::reference::<CheckpointSchema>(), length(x))
            .field(PositionY::reference::<CheckpointSchema>(), length(y))
    };
    // A commit gives its anchor the length one above its height. Every ring
    // starts two above its heights, where no commit puts it, but for
    // `manual-b`: the one body already standing at the length a surveyed
    // commit gives an anchor of height 1.
    let apart = [3, 3, 12];
    for (family, lengths) in [
        ("manual-", [3, 2, 12]),
        ("manual-wide-", apart),
        ("manual-raised-", apart),
        ("manual-wide-raised-", apart),
        ("surveyed-", apart),
    ] {
        let ring = [("a", 1, 1), ("b", 10, 1), ("c", 1, 10)];
        for ((name, x, y), standing) in ring.into_iter().zip(lengths) {
            graph
                .bind_entity(body(&format!("{family}{name}"), x, y, standing))
                .unwrap();
        }
        for (from, to) in [("a", "b"), ("b", "c"), ("c", "a")] {
            graph
                .bind_relation(WorthQueryApplicationRelationSeed::new(
                    PlanarSuccessor::reference::<CheckpointSchema>(),
                    format!("{family}{from}-to-{to}"),
                    key(&format!("{family}{from}")),
                    key(&format!("{family}{to}")),
                ))
                .unwrap();
        }
    }
}
