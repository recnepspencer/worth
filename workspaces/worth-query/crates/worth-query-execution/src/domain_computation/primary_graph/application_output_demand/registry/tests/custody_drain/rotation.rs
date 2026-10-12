//! A large unaffordable head cannot hide the cheap entry behind it.
use super::*;
#[test]
fn registry_foreign_custody_rotates_the_unaffordable_head_and_refunds_its_quote() {
    let receipts = walk_bounds::receipt_batch();
    let (registry, own) = fixture(&receipts[..2], 1);
    {
        let mut state = registry.state.lock().unwrap();
        for generation in 2..=128 {
            let mut row = record(
                receipts[0].product_branch().occurrence(),
                DemandState::Failed(WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::NoEffect,
                    "completed",
                )),
                0,
            );
            row.source_commits.push(
                receipts[0]
                    .committed_product_publication()
                    .composite_commit()
                    .clone(),
            );
            state
                .records
                .insert(key("custody-unrelated-0", generation, 100), row);
        }
    }
    // Match the own row and cardinality in a registry with no noted entries.
    let (clean, _) = live_fixture(&receipts[..2], 1);
    {
        let mut state = clean.state.lock().unwrap();
        for generation in 2..=128 {
            state.records.insert(
                key("custody-unrelated-0", generation, 100),
                record(
                    receipts[0].product_branch().occurrence(),
                    DemandState::Admitted,
                    1,
                ),
            );
        }
    }
    let mut measured = admission(u64::MAX);
    let _clean_owner = admit(&clean, &own, &mut measured).unwrap();
    let mut limited = admission(measured.charged_work() + ENTRY_PRICE);
    let _owner = admit(&registry, &own, &mut limited).unwrap();
    assert_eq!(
        limited.charged_work(),
        measured.charged_work() + ENTRY_PRICE
    );
    let state = registry.state.lock().unwrap();
    let expensive = receipts[0]
        .committed_product_publication()
        .composite_commit();
    let affordable = receipts[1]
        .committed_product_publication()
        .composite_commit();
    assert!(state.source_custody[expensive].cleanup.queued);
    assert!(!state.source_custody.contains_key(affordable));
    assert_eq!(state.custody_cleanup.head.as_ref(), Some(expensive));
}
