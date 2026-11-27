//! Named-records allocator axis: undo and checkout of one deleted member over
//! 10 vs 100 more named members the navigation leaves unchanged allocate at
//! most 1.1x, by count and by requested bytes. Each size gets its own host; a
//! one-member host runs both kinds first, so process-wide first use is
//! outside both samples.
use super::*;
use history::{navigate, record_edit, DeleteAction, Navigate};
use worth_testkit_foundational::allocation_probe::{assert_flat, AllocationCount, AllocationProbe};

/// Delete `direct`, then measure `undo_step` back and `checkout` of the
/// deletion, with `extra` unchanged named members in the owned closure.
fn with_members(extra: usize) -> (AllocationCount, AllocationCount) {
    let host = world::install_with_members(extra);
    let document = world::attach(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let branch = |document: &world::Document<'_>| {
        document
            .selection(&request)
            .unwrap()
            .physical_endpoint()
            .product_branch()
    };
    let original = world::member_identity(&host, branch(&document), "direct").unwrap();
    let document = record_edit!(
        &host,
        document,
        &request,
        delete::Intent {
            key: "direct".into()
        },
        DeleteAction,
        "delete-direct"
    );
    let deleted = document.current_step(&request).unwrap();
    let ((), undone) =
        AllocationProbe::measure(|| navigate(&document, &request, Navigate::Undo, 0x5a72_0001));
    assert_eq!(
        world::member_identity(&host, branch(&document), "direct"),
        Some(original)
    );
    let ((), checked_out) = AllocationProbe::measure(|| {
        navigate(
            &document,
            &request,
            Navigate::Checkout(deleted.id()),
            0x5a72_0002,
        )
    });
    assert_eq!(
        world::member_identity(&host, branch(&document), "direct"),
        None
    );
    let filler = format!("filler-{}", extra - 1);
    assert!(world::member_identity(&host, branch(&document), &filler).is_some());
    (undone, checked_out)
}

/// Navigation prepares the model's whole owned graph, on `prepare` and again
/// on the candidate advance, so both kinds grow with the roster (#462).
#[test]
#[ignore = "held on public #462"]
fn undo_and_checkout_allocate_flat_in_unchanged_named_members() {
    with_members(1);
    let (small_undo, small_checkout) = with_members(10);
    let (large_undo, large_checkout) = with_members(100);
    assert_flat(&[
        ("named/undo_step", small_undo, large_undo),
        ("named/checkout", small_checkout, large_checkout),
    ]);
}
