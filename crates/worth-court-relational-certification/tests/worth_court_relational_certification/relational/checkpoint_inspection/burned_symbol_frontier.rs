//! Advancing the burned symbol frontier never bypasses source-backed census or symbol damage checks.
use super::*;

#[test]
fn burned_symbol_frontier_does_not_hide_published_symbol_damage() {
    let host = request();
    let execution = ExecutionRequest::serial(&host);
    let (before, identities) = original(&execution);
    let base = members_source(&before);
    let inspected = RelationalNativeCheckpoint::inspect_bytes_with_source(
        &execution,
        before.bytes(),
        base.clone(),
    )
    .unwrap();
    for identity in identities {
        assert!(
            inspected.live_records.contains(&identity),
            "source-backed inspection must populate live census with {identity:?}"
        );
    }
    let rewrite_frontier = |n: u32| {
        rewritten(before.bytes(), false, |f| {
            f.insert(
                "symbol_identity_frontier".into(),
                rmp_serde::to_vec(&n).unwrap(),
            );
        })
    };
    let next = inspected.symbol_identity_frontier.checked_add(7).unwrap();
    let advanced = rewrite_frontier(next);
    let after =
        RelationalNativeCheckpoint::inspect_bytes_with_source(&execution, &advanced, base.clone())
            .unwrap();
    assert_eq!(after.symbol_identity_frontier, next);
    assert_eq!(
        (after.model, after.live_records),
        (inspected.model, inspected.live_records)
    );
    assert!(RelationalNativeCheckpoint::inspect_bytes_with_source(
        &execution,
        &rewrite_frontier(0),
        base.clone(),
    )
    .is_err());
    let envelope = fields(&before.bytes()[CHECKPOINT_MAGIC.len()..]);
    let symbols = sequence(fields(envelope["manifest"])["symbols"]);
    let symbol: RelationalDurableMemberLocator =
        rmp_serde::from_slice(fields(symbols[0])["member"]).unwrap();
    let mut payload = base.members.get(&symbol).unwrap().bytes().to_vec();
    *payload.last_mut().unwrap() ^= 1;
    let bytes = ExecutionImmutableBytes::from_external_bytes(Arc::new(payload.into_boxed_slice()));
    let tampered = base.with_published_members(&[RelationalCheckpointMember::New {
        locator: symbol,
        bytes,
    }]);
    assert!(
        RelationalNativeCheckpoint::inspect_bytes_with_source(
            &execution,
            before.bytes(),
            tampered.clone(),
        )
        .is_err(),
        "symbol payloads still undergo owner readmission and digest validation"
    );
    assert!(
        RelationalNativeCheckpoint::inspect_bytes_with_source(&execution, &advanced, tampered)
            .is_err(),
        "advancing the burned symbol frontier must not bypass symbol payload validation"
    );
}
