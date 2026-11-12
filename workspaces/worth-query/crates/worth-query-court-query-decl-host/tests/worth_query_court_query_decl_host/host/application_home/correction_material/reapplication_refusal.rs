//! Reapplication remains refused until its durable selected-lineage continuation lands.
use super::*;
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationCorrectionDenial as Entry,
    primary_graph::WorthQueryCorrectionDerivationDenial as Derivation,
    product::WorthQueryDurableStateAdmissionDenial as Durable,
};
#[test]
fn directory_reapplication_refuses_the_actual_cold_alternative_before_preparation() {
    let path = fixture::location();
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = open(path.clone(), execution);
    let original = edit(&home);
    let coordinates = original.correction_source().coordinates();
    drop(original);
    let limits =
        WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::new(8).unwrap());
    {
        let scope = scope();
        let principal = label::authentication::authenticate_external(
            home.runtime().installed_schema(),
            "alice",
            Duration::from_secs(120),
            &scope,
        );
        let source = home
            .runtime()
            .request(&principal, &scope)
            .corrections()
            .select_coordinates(&coordinates, limits)
            .unwrap();
        let Prepared::Prepared(prepared) = home
            .runtime()
            .request(&principal, &scope)
            .corrections()
            .reverse(&source)
            .idempotency("actual-reversal")
            .limits(limits)
            .prepare()
            .unwrap()
        else {
            panic!("new reverse");
        };
        assert!(matches!(
            prepared.publish().unwrap(),
            Outcome::Committed { .. }
        ));
    }
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("actual reversal close: {cause:?}"))
        .reclaim();
    let home = open(path.clone(), execution);
    {
        let scope = scope();
        let principal = label::authentication::authenticate_external(
            home.runtime().installed_schema(),
            "alice",
            Duration::from_secs(120),
            &scope,
        );
        let request = home.runtime().request(&principal, &scope);
        let source = request
            .corrections()
            .select_coordinates(&coordinates, limits)
            .unwrap();
        let page = request
            .corrections()
            .alternatives(&source, limits, std::num::NonZeroUsize::new(8).unwrap())
            .unwrap();
        assert!(page.continuation().is_none());
        assert_eq!(page.entries().len(), 1);
        let alternative = page.select(&page.entries()[0], limits).unwrap();
        let denial = request
            .corrections()
            .reapply(&alternative)
            .idempotency("unsupported-reapplication")
            .limits(limits)
            .prepare()
            .err()
            .expect("no durable reapplication custody");
        assert!(
            matches!(
                denial,
                Entry::Derivation(Derivation::DurableStateAdmission(
                    Durable::UnsupportedCorrectionReapplication
                ))
            ),
            "{denial:?}"
        );
        let rows = request
            .query(label::AccountSummaryParameters)
            .execute()
            .unwrap();
        assert_eq!(rows.rows()[0].label, "primary");
    }
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("no reapplication pin: {cause:?}"))
        .reclaim();
    let home = open(path, execution);
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("reapplication refusal reopen: {cause:?}"))
        .reclaim();
}
