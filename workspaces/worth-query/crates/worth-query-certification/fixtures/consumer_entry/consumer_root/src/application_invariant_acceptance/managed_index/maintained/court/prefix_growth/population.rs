//! Bounded installed authoring; the independent roster never comes from provider nodes.
use super::*;
pub(super) fn items(local: u32, remote: u32) -> Vec<authoring::ItemSeed> {
    let controls = [
        ("tie-z", "eq-shared"),
        ("tie-a", "eq-shared"),
        ("unicode-composed", "álamo"),
        ("unicode-decomposed", "a\u{301}"),
        ("unicode-greek", "βeta"),
    ];
    controls
        .into_iter()
        .map(|(identity, name)| authoring::ItemSeed {
            identity: identity.into(),
            name: name.into(),
        })
        .chain((0..local).map(|n| authoring::ItemSeed {
            identity: format!("local-{n:08}"),
            name: format!("near-{n:08}"),
        }))
        .chain((0..remote).map(|n| authoring::ItemSeed {
            identity: format!("other-{n:08}"),
            name: format!("away-{n:08}"),
        }))
        .collect()
}
pub(super) fn populate(world: &host::Host, items: &[authoring::ItemSeed]) {
    for (batch, items) in items.chunks(authoring::MAX_ITEMS_PER_BATCH).enumerate() {
        let scope = authentication::request_scope();
        let principal = authenticate(world, &scope);
        let request = world
            .runtime()
            .request(&principal, &scope)
            .on_branch(world.current_world());
        let (result, work) = invoice(|| {
            request
                .advanced()
                .mutate(authoring::Edit::Items {
                    directory: "A".into(),
                    items: items.to_vec(),
                })
                .idempotency(&(0x919d100u64 + batch as u64))
                .execute_in_program(world, ExecutionAllocationPolicy::SystemAllocation)
                .unwrap()
        });
        eprintln!(
            "prefix setup batch={batch} items={} whole SDK={work}",
            items.len()
        );
        assert!(
            matches!(
                result,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "{result:?}"
        );
    }
}
pub(super) fn invalid_batches(
    world: &host::Host,
    principal: &Principal,
    scope: &WorthQueryRequestScope,
    provider: &prefix::Calls,
) {
    let request = world
        .runtime()
        .request(principal, scope)
        .on_branch(world.current_world());
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let counts = (provider.totals(), provider.updates.load(Ordering::SeqCst));
    for (offset, items) in [
        Vec::new(),
        (0..=authoring::MAX_ITEMS_PER_BATCH)
            .map(|n| authoring::ItemSeed {
                identity: format!("invalid-{n}"),
                name: "invalid".into(),
            })
            .collect(),
    ]
    .into_iter()
    .enumerate()
    {
        let result = request
            .advanced()
            .mutate(authoring::Edit::Items {
                directory: "A".into(),
                items,
            })
            .idempotency(&(0x919d10u64 + offset as u64))
            .execute_in_program(world, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(
            matches!(
                result,
                WorthQueryApplicationMutationOutcome::DomainDenied(authoring::Denial::InvalidBatch)
            ),
            "{result:?}"
        );
    }
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head
    );
    assert_eq!(
        (provider.totals(), provider.updates.load(Ordering::SeqCst)),
        counts
    );
    let source = request
        .query(query::ReadSource {
            directory: "A".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(values(source.rows()), expected("A", "shared", ""));
}
