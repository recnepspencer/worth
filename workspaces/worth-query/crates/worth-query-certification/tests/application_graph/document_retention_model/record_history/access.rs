//! The audit panel uses only installed Decl meaning and actual Host admission.
use super::super::{
    host::DocumentRetentionRuntime,
    operator_identity::{authenticate_operator, request_scope},
    programs::RetentionProgramP0,
    retention_entry::DOCUMENT_IDENTITY,
};
use super::*;
use std::num::NonZeroUsize;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationReadObservation as Root, WorthQueryApplicationRequestExt,
    },
    primary_graph::*,
};

pub(super) type App = DocumentRetentionRuntime<RetentionProgramP0>;
pub(super) type Subjects<H> =
    WorthQueryCapturedApplicationHistorySubjects<DocumentRetentionSchema, H>;
pub(super) type Cursor<H> = WorthQueryApplicationHistoryCursor<DocumentRetentionSchema, H>;
pub(super) type Payload = WorthQueryAdmittedApplicationCapabilityAccess<
    DocumentRetentionSchema,
    capability::PayloadCapability,
    capability::PayloadRead,
    capability::Input,
>;
pub(super) type Metadata = WorthQueryAdmittedApplicationCapabilityAccess<
    DocumentRetentionSchema,
    capability::MetadataCapability,
    capability::MetadataRead,
    capability::Input,
>;
pub(super) fn limits() -> WorthQueryApplicationHistoryPageLimits {
    WorthQueryApplicationHistoryPageLimits {
        maximum_entries: NonZeroUsize::MIN,
        maximum_source_visits: NonZeroUsize::MIN,
        maximum_owned_bytes: NonZeroUsize::new(64 * 1024).unwrap(),
    }
}
fn capture_limits() -> WorthQueryApplicationHistoryCaptureLimits {
    WorthQueryApplicationHistoryCaptureLimits {
        maximum_subjects: NonZeroUsize::MIN,
        maximum_aspects: NonZeroUsize::MIN,
        maximum_source_visits: NonZeroUsize::new(16).unwrap(),
        maximum_owned_bytes: limits().maximum_owned_bytes,
    }
}
pub(super) fn observe(application: &App) -> Root {
    observe_on(application, application.runtime().current_world())
}
pub(super) fn observe_on(
    application: &App,
    branch: worth_query_host::facade::product::WorthQueryProductBranch,
) -> Root {
    let runtime = application.runtime();
    let request = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &request);
    runtime
        .request(&principal, &request)
        .on_branch(branch)
        .retain_current_read()
        .unwrap()
}
/// The returned capabilities are actual admitted values, deliberately retained across revocation.
pub(super) fn capabilities(application: &App) -> (Payload, Metadata) {
    let runtime = application.runtime();
    let request = request_scope();
    let external = authenticate_operator(runtime.installed_schema(), &request);
    runtime
        .with_application_advancement(&request, |phase| {
            let selected = runtime
                .on_branch(runtime.current_world())
                .select(&phase)
                .unwrap();
            let binding = runtime
                .installed_schema()
                .principal_binding(DocumentPrincipalBinding::reference())
                .unwrap();
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let payload = runtime
                .installed_schema()
                .capability(
                    capability::PayloadCapability::reference(),
                    capability::PayloadRead::reference(),
                )
                .unwrap();
            let metadata = runtime
                .installed_schema()
                .capability(
                    capability::MetadataCapability::reference(),
                    capability::MetadataRead::reference(),
                )
                .unwrap();
            let input = || capability::Input {
                identity: DOCUMENT_IDENTITY.to_owned(),
            };
            (
                selected
                    .admit_capability_access(&phase, &principal, &payload, input(), &request)
                    .unwrap(),
                selected
                    .admit_capability_access(&phase, &principal, &metadata, input(), &request)
                    .unwrap(),
            )
        })
        .unwrap()
}
pub(super) fn capture_public(application: &App) -> Subjects<PublicChanges> {
    let runtime = application.runtime();
    let request = request_scope();
    let external = authenticate_operator(runtime.installed_schema(), &request);
    runtime
        .with_application_advancement(&request, |phase| {
            let history = runtime
                .installed_schema()
                .installed_history_query::<PublicChanges>()
                .unwrap();
            let selected = runtime
                .on_branch(runtime.current_world())
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
                    DocumentIdentityField::reference(),
                    DOCUMENT_IDENTITY.to_owned(),
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
                .admit_application_history_capture(&execution, &history, plan, capture_limits())
                .unwrap();
            runtime
                .execute_application_history_capture(&phase, capture)
                .unwrap()
                .into_parts()
                .1
        })
        .unwrap()
}
pub(super) fn capture_governed(application: &App) -> Subjects<GovernedChanges> {
    let (payload, metadata) = capabilities(application);
    let runtime = application.runtime();
    let request = request_scope();
    let external = authenticate_operator(runtime.installed_schema(), &request);
    runtime
        .with_application_advancement(&request, |phase| {
            let history = runtime
                .installed_schema()
                .installed_history_query::<GovernedChanges>()
                .unwrap();
            let selected = runtime
                .on_branch(runtime.current_world())
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
                    DocumentIdentityField::reference(),
                    DOCUMENT_IDENTITY.to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let access = WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
            let plan = selected
                .admit_governed_application_query(
                    &phase,
                    history.subject_read().query(),
                    &access,
                    payload,
                    query::parameters(DOCUMENT_IDENTITY),
                    WorthQueryProductQueryControls::new(NonZeroUsize::MIN, &request),
                )
                .unwrap();
            let execution = phase
                .checked_execution_request_for(runtime.product_runtime())
                .unwrap();
            let capture = runtime
                .admit_governed_application_history_capture(
                    &execution,
                    &history,
                    plan,
                    metadata,
                    capture_limits(),
                )
                .unwrap();
            let (read, subjects) = runtime
                .execute_application_history_capture(&phase, capture)
                .unwrap()
                .into_parts();
            assert_eq!(read.rows()[0].identity, DOCUMENT_IDENTITY);
            subjects
        })
        .unwrap()
}
pub(super) fn open_public(
    application: &App,
    subjects: &Subjects<PublicChanges>,
    root: &Root,
) -> Cursor<PublicChanges> {
    let runtime = application.runtime();
    let request = request_scope();
    let external = authenticate_operator(runtime.installed_schema(), &request);
    runtime
        .with_application_advancement(&request, |phase| {
            let history = runtime
                .installed_schema()
                .installed_history_query::<PublicChanges>()
                .unwrap();
            let truth = root.select_on(&phase, runtime).unwrap();
            let security = runtime
                .on_branch(truth.product().product_branch())
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
pub(super) fn page_public(
    application: &App,
    subjects: &Subjects<PublicChanges>,
    cursor: &mut Cursor<PublicChanges>,
    root: &Root,
) -> Result<WorthQueryApplicationHistoryPage, WorthQueryApplicationHistoryDenial> {
    let runtime = application.runtime();
    let request = request_scope();
    let external = authenticate_operator(runtime.installed_schema(), &request);
    runtime
        .with_application_advancement(&request, |phase| {
            let history = runtime
                .installed_schema()
                .installed_history_query::<PublicChanges>()
                .unwrap();
            let truth = root.select_on(&phase, runtime).unwrap();
            let security = runtime
                .on_branch(truth.product().product_branch())
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
                &request,
                limits(),
            )
        })
        .unwrap()
}
pub(super) fn open_governed(
    application: &App,
    subjects: &Subjects<GovernedChanges>,
    root: &Root,
) -> Cursor<GovernedChanges> {
    let (payload, metadata) = capabilities(application);
    let runtime = application.runtime();
    let request = request_scope();
    let external = authenticate_operator(runtime.installed_schema(), &request);
    runtime
        .with_application_advancement(&request, |phase| {
            let history = runtime
                .installed_schema()
                .installed_history_query::<GovernedChanges>()
                .unwrap();
            let truth = root.select_on(&phase, runtime).unwrap();
            let security = runtime
                .on_branch(runtime.current_world())
                .select(&phase)
                .unwrap();
            runtime
                .open_governed_application_history(
                    &phase,
                    &history,
                    &external,
                    subjects,
                    security,
                    &truth,
                    payload,
                    metadata,
                    &request,
                    limits(),
                )
                .unwrap()
        })
        .unwrap()
}
pub(super) fn page_governed(
    application: &App,
    subjects: &Subjects<GovernedChanges>,
    cursor: &mut Cursor<GovernedChanges>,
    root: &Root,
    capabilities: (Payload, Metadata),
    maximum_bytes: NonZeroUsize,
) -> Result<WorthQueryApplicationHistoryPage, WorthQueryApplicationHistoryDenial> {
    let runtime = application.runtime();
    let request = request_scope();
    let external = authenticate_operator(runtime.installed_schema(), &request);
    runtime
        .with_application_advancement(&request, |phase| {
            let history = runtime
                .installed_schema()
                .installed_history_query::<GovernedChanges>()
                .unwrap();
            let truth = root.select_on(&phase, runtime).unwrap();
            let security = runtime
                .on_branch(runtime.current_world())
                .select(&phase)
                .unwrap();
            let mut limits = limits();
            limits.maximum_owned_bytes = maximum_bytes;
            runtime.read_governed_application_history(
                &phase,
                &history,
                &external,
                subjects,
                cursor,
                security,
                &truth,
                capabilities.0,
                capabilities.1,
                &request,
                limits,
            )
        })
        .unwrap()
}
