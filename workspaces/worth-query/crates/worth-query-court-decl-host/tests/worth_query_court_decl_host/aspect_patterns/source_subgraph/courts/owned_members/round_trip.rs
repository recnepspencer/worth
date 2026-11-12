//! Shared ordinary roster actions used by the roster courts.
use super::*;
pub(in super::super) fn image(count: usize) -> Image {
    Image {
        key: "roster".into(),
        value: 7,
        members: (0..count)
            .map(|i| Row {
                key: format!("part-{i}"),
                value: 20 + i as u64,
                next: (i + 1 < count).then(|| format!("part-{}", i + 1)),
                operand: None,
            })
            .collect(),
    }
}
pub(in super::super) fn created(
    host: &world::Host,
    request: &world::Request<'_, '_, '_>,
    image: Image,
) -> WorthQueryApplicationCommitReceipt {
    let creation_key = format!("create:{}", image.key);
    let result = request
        .advanced()
        .mutate(Create(image))
        .without_source()
        .idempotency(&creation_key)
        .execute_in_program(
            host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = result else {
        panic!("{result:?}");
    };
    receipt
}
pub(in super::super) fn reverse(
    request: &world::Request<'_, '_, '_>,
    receipt: &WorthQueryApplicationCommitReceipt,
    key: &str,
) {
    let limits = WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::MIN);
    let selected = request.corrections().select(receipt, limits).unwrap();
    let preparation = request
        .corrections()
        .reverse(&selected)
        .idempotency(key)
        .limits(limits)
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = preparation else {
        panic!("inverse did not prepare");
    };
    let result = prepared.publish().unwrap();
    assert!(
        matches!(
            result,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{result:?}"
    );
}
