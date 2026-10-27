//! Installed record history composes native changes, exact World ancestry and fresh Query admission.
use worth_query_declaration::facade::{
    application_query::{
        ApplicationHistoryQueryBinding, ApplicationQueryBinding, ApplicationQueryScopeBinding,
    },
    application_schema::ApplicationStructuredValueBinding,
};

mod admission;
mod bounds;
mod capture;
mod capture_plan;
mod cursor;
mod denial;
mod fresh;
mod page;
mod request_stage;
mod session;
mod storage;
mod transition;

pub(super) use admission::HistoryMetadataAdmission;
pub use bounds::WorthQueryApplicationHistoryCaptureLimits;
pub(super) use capture::prepare_capture;
pub use capture::WorthQueryCapturedApplicationHistorySubjects;
pub use capture_plan::WorthQueryAdmittedApplicationHistoryCapture;
pub use cursor::WorthQueryApplicationHistoryCursor;
pub use denial::WorthQueryApplicationHistoryDenial;
pub use page::{
    WorthQueryApplicationHistoryEntry, WorthQueryApplicationHistoryPage,
    WorthQueryApplicationHistoryPageLimits,
};

pub(super) type SubjectRead<Schema, History> =
    <History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead;
pub(super) type SubjectQuery<Schema, History> =
    <SubjectRead<Schema, History> as ApplicationQueryBinding<Schema>>::Query;
pub(super) type SubjectParameters<Schema, History> = <<SubjectRead<Schema, History> as ApplicationQueryBinding<Schema>>::ParameterBinding as ApplicationStructuredValueBinding>::Value;
pub(super) type SubjectResult<Schema, History> = <<SubjectRead<Schema, History> as ApplicationQueryBinding<
    Schema,
>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
pub(super) type SubjectScope<Schema, History> = <<SubjectRead<Schema, History> as ApplicationQueryBinding<
    Schema,
>>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope;
pub(super) type SubjectPlan<'a, Schema, History> = super::WorthQueryAdmittedApplicationQueryPlan<
    'a,
    Schema,
    SubjectQuery<Schema, History>,
    SubjectParameters<Schema, History>,
    SubjectResult<Schema, History>,
    <SubjectRead<Schema, History> as ApplicationQueryBinding<Schema>>::Principal,
    <SubjectRead<Schema, History> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
    SubjectScope<Schema, History>,
>;
type SubjectAccess<'a, Schema, History> = super::WorthQueryApplicationQueryAccessContext<
    'a,
    Schema,
    <SubjectRead<Schema, History> as ApplicationQueryBinding<Schema>>::Principal,
    <SubjectRead<Schema, History> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
    SubjectScope<Schema, History>,
>;
