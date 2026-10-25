//! Fresh public chooser reads canonical alternatives after real native index disposal.
use super::super::super::source_delivery::correction as commands;
use super::*;
use std::num::NonZeroUsize;
use worth_query_host::facade::{application_entry::*, primary_graph::*};

#[test]
#[expect(
    clippy::result_large_err,
    reason = "Keep complete typed Query refusal evidence, following the Query workspace policy"
)]
fn public_family_history_chooser_survives_handle_and_index_disposal() {
    let _guard = checkpoint_recovery_test_guard();
    let mut app = correction::install(0);
    let root = {
        let (scope, principal) = authenticate(&app);
        let request = app.request(&principal, &scope);
        correction::settled(&app, &request, true);
        let source = correction::source(&app, &request);
        correction::settled(&app, &request, false);
        let reversal = correction::reverse(&app, &request, &source);
        correction::settled(&app, &request, false);
        reversal
            .aftermath_causality()
            .unwrap()
            .recorded_source()
            .coordinates()
    };
    let baseline = entries(&app, &root);
    assert_eq!(baseline.len(), 1);
    let removed = app
        .with_host_advancement(|phase| app.release_correction_history_indexes_for_test(&phase))
        .unwrap()
        .unwrap();
    assert!(removed > 0);
    {
        let (scope, principal) = authenticate(&app);
        let request = app.request(&principal, &scope);
        let selected = request
            .corrections()
            .select_coordinates(&root, commands::limits())
            .unwrap();
        assert!(matches!(
            request
                .corrections()
                .alternatives(&selected, commands::limits(), NonZeroUsize::MIN),
            Err(WorthQueryApplicationCorrectionDenial::Derivation(
                WorthQueryCorrectionDerivationDenial::History(
                    WorthQueryAftermathCausalityReadDenial::IndexUnavailable { .. }
                )
            ))
        ));
    }
    let rebuilt = app
        .with_host_advancement(|phase| {
            app.rebuild_correction_history_indexes_for_test(&phase, app.current_world())
        })
        .unwrap()
        .unwrap();
    assert_eq!(rebuilt.generations.len(), 3);
    assert_eq!(entries(&app, &root), baseline);
    app.close_conditional_runtime().unwrap();
}
fn entries(
    app: &correction::App,
    root: &WorthQueryCorrectionSourceCoordinates,
) -> Vec<WorthQueryCorrectionAlternativeRef> {
    let (scope, principal) = authenticate(app);
    let request = app.request(&principal, &scope);
    let selected = request
        .corrections()
        .select_coordinates(root, commands::limits())
        .unwrap();
    let page = request
        .corrections()
        .alternatives(&selected, commands::limits(), NonZeroUsize::MIN)
        .unwrap()
        .into_page();
    assert!(page.continuation().is_none());
    assert!(
        page.resolution().is_some(),
        "unique active resolution completes through native traversal"
    );
    let alternative = request
        .corrections()
        .select_alternative_coordinates(&page, root, commands::limits())
        .unwrap();
    drop(alternative);
    page.entries().to_vec()
}
