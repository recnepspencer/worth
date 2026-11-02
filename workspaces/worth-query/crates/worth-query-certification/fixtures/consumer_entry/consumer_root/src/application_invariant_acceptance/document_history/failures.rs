//! Composed command refusals keep the same executable's real domain owners.
use super::*;
use model::{with_branch_request, with_request};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
mod authority;
mod delivery;
mod group;
mod selection;

fn with_scope<T>(
    app: &Runtime,
    scope: &WorthQueryRequestScope,
    perform: impl FnOnce(&Request<'_, '_, '_>) -> T,
) -> T {
    let adapter = authentication::admit(app.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        scope,
    ))
    .unwrap();
    perform(&app.request(&principal, scope))
}

fn install() -> installation::ConsumerWorld<FixedProgram> {
    setup::install(&crate::installed_schema())
}

fn pair(
    app: &Runtime,
) -> (
    Configured<'_>,
    WorthQueryHistoryDescriptionContext,
    WorthQueryHistoryStep,
) {
    let document = setup::configure(model::attach(app, "anchor-a"), app);
    let context = with_request(app, |request| {
        view::page(&document, request, None).unwrap().context
    });
    let (document, first) = authoring::record(document, app, &context, 2, 0x9182_7700);
    let (document, _) = authoring::record(document, app, &context, 5, 0x9182_7701);
    (document, context, first)
}

fn selected(document: &Configured<'_>, app: &Runtime) -> WorthQueryHistorySelection {
    with_request(app, |request| document.selection(request).unwrap())
}

fn capture(
    document: &Configured<'_>,
    app: &Runtime,
    context: &WorthQueryHistoryDescriptionContext,
    intent: WorthQueryHistoryNavigationIntent,
    key: u128,
) -> navigation::Command {
    with_request(app, |request| {
        navigation::capture(document, request, context, intent, key)
    })
    .unwrap()
}

fn refresh(document: &mut Configured<'_>, app: &Runtime) {
    with_request(app, |request| {
        document
            .refresh_context(
                request,
                WorthQueryHistoryContextRefreshLimits {
                    maximum_commits: NonZeroUsize::new(16).unwrap(),
                    maximum_records: NonZeroUsize::new(1024).unwrap(),
                    maximum_evidence_bytes: std::num::NonZeroU64::new(1_048_576).unwrap(),
                },
            )
            .unwrap()
    });
}

fn members(first: u64, second: u64, key: u64) -> [(Original, u64); 2] {
    let input = |y| {
        Original(PlanarInitialAdjustment {
            scope_key: "anchor-a".into(),
            replacement_y: length(y),
        })
    };
    [(input(first), key), (input(second), key + 1)]
}
