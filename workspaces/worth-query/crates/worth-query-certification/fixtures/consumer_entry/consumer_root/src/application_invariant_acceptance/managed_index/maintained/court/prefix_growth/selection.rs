//! Complete census outside edit invoices; sparse Parent results keep their actual order.
use super::*;
fn roster<'a>(
    items: &'a [authoring::ItemSeed],
    x: &'a str,
) -> impl Iterator<Item = (&'a str, &'a str)> {
    std::iter::once(("x", x)).chain(
        items
            .iter()
            .map(|item| (item.identity.as_str(), item.name.as_str())),
    )
}
pub(super) fn census(
    world: &host::Host,
    principal: &Principal,
    scope: &WorthQueryRequestScope,
    items: &[authoring::ItemSeed],
    x: &str,
) {
    let request = world
        .runtime()
        .request(principal, scope)
        .on_branch(world.current_world());
    let (source, work) = invoice(|| {
        request
            .query(query::ReadSource {
                directory: "A".into(),
            })
            .execute()
            .unwrap()
    });
    eprintln!(
        "prefix complete Source census N={} whole SDK={work}",
        items.len() + 1
    );
    assert_eq!(
        source.rows().len(),
        items.len() + 1,
        "the actual ordinary Source is complete above16"
    );
    let mut expected: Vec<_> = roster(items, x).collect();
    expected.sort_by(|a, b| b.1.cmp(a.1).then_with(|| b.0.cmp(a.0)));
    assert_eq!(
        source
            .rows()
            .iter()
            .map(|row| (row.identity.as_str(), row.name.as_str()))
            .collect::<Vec<_>>(),
        expected
    );
}
pub(super) fn check(
    world: &host::Host,
    principal: &Principal,
    scope: &WorthQueryRequestScope,
    calls: &prefix::Calls,
    items: &[authoring::ItemSeed],
    x: &str,
    stage: &str,
) {
    let request = world
        .runtime()
        .request(principal, scope)
        .on_branch(world.current_world());
    let count = items.len() + 1;
    let bound = 8 * (usize::BITS - count.leading_zeros()) as usize + 16;
    for probe in [
        "absent",
        "shared",
        "taken",
        "free",
        "eq-shared",
        "ál",
        "a\u{301}",
        "β",
        "near-00000001",
        "away-00000001",
    ] {
        let mut expected: Vec<_> = roster(items, x)
            .filter(|(_, name)| name.starts_with(probe))
            .collect();
        expected.sort_by(|a, b| a.0.cmp(b.0));
        assert!(expected.len() <= 2, "authored sparse density stays fixed");
        let before = (
            calls.summary_comparisons.load(Ordering::SeqCst),
            calls.matched_sources.load(Ordering::SeqCst),
            calls.successful_matches.load(Ordering::SeqCst),
        );
        let (actual, work) = invoice(|| request.query(find("A", probe)).execute().unwrap());
        assert_eq!(
            actual
                .rows()
                .iter()
                .map(|row| (row.identity.as_str(), row.name.as_str()))
                .collect::<Vec<_>>(),
            expected,
            "{stage} {probe:?}"
        );
        let comparisons = calls.summary_comparisons.load(Ordering::SeqCst) - before.0;
        let attempts = calls.matched_sources.load(Ordering::SeqCst) - before.1;
        let matched = calls.successful_matches.load(Ordering::SeqCst) - before.2;
        assert_eq!(matched, expected.len());
        assert!(attempts <= comparisons && comparisons <= bound && work > 0, "{stage} {probe:?} comparisons={comparisons} attempts={attempts} work={work}, bound={bound}");
        eprintln!("prefix {stage} Source={count} probe={probe:?} comparisons={comparisons} attempts={attempts} matches={matched} whole SDK read={work}");
    }
}
