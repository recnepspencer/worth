//! Original fixed preparations and required-origin interests close after source C moves.
use super::*;
mod producer;
mod source;
use std::{num::NonZeroUsize, time::Duration};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
    product::{
        WorthQueryProductBranch, WorthQueryProductBranchAdmissionDenial,
        WorthQueryProductBranchAdmissionRefusalKind,
    },
};

fn with_scope<T>(
    app: &App,
    scope: &WorthQueryRequestScope,
    f: impl FnOnce(&Request<'_, '_, '_>) -> T,
) -> T {
    let adapter = authentication::admit(app.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        scope,
    ))
    .unwrap();
    f(&app.request(&principal, scope))
}
fn stopped_abort<'a>(app: &App, group: Group<'a>) -> Group<'a> {
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let (group, cause) = match with_scope(app, &scope, |request| {
        cancellation.cancel();
        group.abort(request)
    }) {
        Err(refused) => refused,
        Ok(_) => panic!("stopped close retains the same grouped root"),
    };
    assert!(
        matches!(
            cause,
            WorthQueryHistoryGroupDenial::Close(WorthQueryOperationCloseDenial::Execution(
                WorthQueryAdvancementDenial::Interrupted(
                    WorthQueryManagedComputationInterruption::Cancelled
                )
            ))
        ),
        "{cause:?}"
    );
    assert_eq!(group.posture(), WorthQueryHistoryGroupPosture::Aborting);
    group
}
fn retired(app: &App, candidate: WorthQueryProductBranch) {
    let cause = app
        .with_host_advancement(
            |phase| match app.runtime().on_branch(candidate).select(&phase) {
                Err(cause) => cause,
                Ok(_) => panic!("original required candidate must retire"),
            },
        )
        .unwrap();
    assert_eq!(
        cause,
        WorthQueryProductBranchAdmissionDenial::Refused(
            WorthQueryProductBranchAdmissionRefusalKind::RetiredBranch
        )
    );
    assert!(app
        .runtime()
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap()
        .rows()
        .is_empty());
    assert!(app.runtime().branches().pending_cleanup().is_empty());
}

#[test]
fn abort_closes_the_original_fixed_preparation_and_started_tree_after_source_movement() {
    for start_tree in [false, true] {
        let world = install_full();
        let app = &world.application;
        let document = attach(app, "anchor-a");
        let source = with_request(app, |request| document.selection(request).unwrap());
        let mut group = group(document, app, 0x9182_5810);
        until(app, &mut group, |group| {
            let (_, execution) = group.parts();
            execution
                .source_operation(0)
                .is_some_and(|operation| operation.prepared_source_receipt().is_some())
        });
        let candidate = {
            let (_, execution) = group.parts();
            execution.source_operation(0).unwrap().branch()
        };
        if start_tree {
            with_request(app, |request| {
                let (document, execution) = group.parts();
                execution.advance(document, request).unwrap();
            });
        }
        let group = stopped_abort(app, group);
        ordinary(
            app,
            source.physical_endpoint().product_branch(),
            5,
            0x9182_5811,
        );
        assert!(matches!(
            with_request(app, |request| group.document().read(request)),
            Err(WorthQueryApplicationHistoryDenial::Document(
                WorthQueryDocumentHistoryDenial::HeadDiverged
            ))
        ));
        let document =
            with_request(app, |request| group.abort(request)).unwrap_or_else(|(_, cause)| {
                panic!("original required custody must dispose without C readmission: {cause:?}")
            });
        assert!(app
            .runtime()
            .product_publication_recovery_page(None, std::num::NonZeroUsize::new(8).unwrap())
            .unwrap()
            .rows()
            .is_empty());
        assert!(app.runtime().branches().pending_cleanup().is_empty());
        retired(app, candidate);
        with_branch_request(
            app,
            source.physical_endpoint().product_branch(),
            |request| {
                assert_eq!(
                    request
                        .query(PlanarOutputRead {
                            body_key: "anchor-a".into()
                        })
                        .execute()
                        .unwrap()
                        .rows()[0]
                        .value,
                    length(6)
                )
            },
        );
        close(document, app);
    }
}
