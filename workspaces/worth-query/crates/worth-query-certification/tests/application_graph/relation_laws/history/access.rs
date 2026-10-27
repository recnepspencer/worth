//! Each helper goes through the supported Host phase, installed binding and real external principal.
use super::super::installation::World;
use super::*;
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use std::num::NonZeroUsize;
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    application_contribution::WorthQueryAdvancementDenial,
    application_entry::{
        WorthQueryApplicationReadObservation as Root, WorthQueryApplicationRequestExt,
    },
    primary_graph::*,
};

pub(super) fn limits() -> WorthQueryApplicationHistoryPageLimits {
    WorthQueryApplicationHistoryPageLimits {
        maximum_entries: NonZeroUsize::MIN,
        maximum_source_visits: NonZeroUsize::MIN,
        maximum_owned_bytes: NonZeroUsize::new(64 * 1024).unwrap(),
    }
}

pub(super) fn observe(world: &World<Payload>) -> Root {
    let runtime = world.application.runtime();
    let request = request_scope();
    let external = authenticate_operator(runtime.installed_schema(), &request);
    runtime
        .request(&external, &request)
        .on_branch(world.application.current_world())
        .retain_current_read()
        .unwrap()
}

pub(super) fn capture<History>(
    world: &World<Payload>,
    key: u64,
) -> WorthQueryCapturedApplicationHistorySubjects<Schema, History>
where
    History: ApplicationHistoryQueryBinding<Schema, SubjectRead = Binding>,
{
    let input = ReadInput { key };
    let runtime = world.application.runtime();
    let request = request_scope();
    let external = authenticate_operator(runtime.installed_schema(), &request);
    runtime
        .with_application_advancement(&request, |phase| {
            let history = runtime
                .installed_schema()
                .installed_history_query::<History>()
                .unwrap();
            let selected = runtime
                .on_branch(world.application.current_world())
                .select(&phase)
                .unwrap();
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    history.subject_read().principal_binding(),
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let scope = selected
                .resolve_entity(
                    &phase,
                    NodeKey::reference(),
                    input.key,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
            let plan = selected
                .admit_application_query(
                    &phase,
                    history.subject_read().query(),
                    &access,
                    ApplicationQueryParameterSet::new(),
                    WorthQueryProductQueryControls::new(NonZeroUsize::MIN, &request),
                )
                .unwrap();
            let execution = phase
                .checked_execution_request_for(runtime.product_runtime())
                .unwrap();
            let capture = runtime
                .admit_application_history_capture(
                    &execution,
                    &history,
                    plan,
                    WorthQueryApplicationHistoryCaptureLimits {
                        maximum_subjects: NonZeroUsize::new(8).unwrap(),
                        maximum_aspects: NonZeroUsize::MIN,
                        maximum_source_visits: NonZeroUsize::new(128).unwrap(),
                        maximum_owned_bytes: limits().maximum_owned_bytes,
                    },
                )
                .unwrap();
            let (read, subjects) = runtime
                .execute_application_history_capture(&phase, capture)
                .unwrap()
                .into_parts();
            assert_eq!(read.rows()[0].key, key);
            if key == 0 {
                assert_eq!(read.rows()[0].children, vec![3, 2, 1]);
            }
            subjects
        })
        .unwrap()
}

pub(super) fn open<History>(
    world: &World<Payload>,
    subjects: &WorthQueryCapturedApplicationHistorySubjects<Schema, History>,
    root: &Root,
) -> WorthQueryApplicationHistoryCursor<Schema, History>
where
    History: ApplicationHistoryQueryBinding<Schema>,
{
    let runtime = world.application.runtime();
    let request = request_scope();
    let external = authenticate_operator(runtime.installed_schema(), &request);
    runtime
        .with_application_advancement(&request, |phase| {
            let history = runtime
                .installed_schema()
                .installed_history_query::<History>()
                .unwrap();
            let truth = root.select_on(&phase, runtime).unwrap();
            let security = runtime
                .on_branch(world.application.current_world())
                .select(&phase)
                .unwrap();
            runtime
                .open_application_history(
                    &phase,
                    &history,
                    &external,
                    subjects,
                    security,
                    &truth,
                    &request,
                    limits(),
                )
                .unwrap()
        })
        .unwrap()
}

pub(super) fn page<History>(
    world: &World<Payload>,
    subjects: &WorthQueryCapturedApplicationHistorySubjects<Schema, History>,
    cursor: &mut WorthQueryApplicationHistoryCursor<Schema, History>,
    root: &Root,
    request: &WorthQueryRequestScope,
) -> Result<
    Result<WorthQueryApplicationHistoryPage, WorthQueryApplicationHistoryDenial>,
    WorthQueryAdvancementDenial,
>
where
    History: ApplicationHistoryQueryBinding<Schema>,
{
    let runtime = world.application.runtime();
    let authentication = request_scope();
    let external = authenticate_operator(runtime.installed_schema(), &authentication);
    runtime.with_application_advancement(request, |phase| {
        let history = runtime
            .installed_schema()
            .installed_history_query::<History>()
            .unwrap();
        let truth = root.select_on(&phase, runtime).unwrap();
        let security = runtime
            .on_branch(world.application.current_world())
            .select(&phase)
            .unwrap();
        runtime.read_application_history(
            &phase,
            &history,
            &external,
            subjects,
            cursor,
            security,
            &truth,
            request,
            limits(),
        )
    })
}
