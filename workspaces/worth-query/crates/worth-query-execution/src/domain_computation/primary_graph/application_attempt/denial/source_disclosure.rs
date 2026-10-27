//! Original source parameter and execution refusals retain their native categories.
use super::{
    AttemptDenialCause, WorthQueryApplicationAttemptDenial as Denial,
    WorthQueryApplicationAttemptDenialKind as Kind,
};
use crate::domain_computation::{
    WorthQueryProviderSessionControlStopKind as Control, WorthQueryProviderSessionDenialKind,
};
use worth_query_admission::facade::application_query::WorthQueryApplicationQueryParameterDenial;

impl Denial {
    pub(in crate::domain_computation::primary_graph) fn source_disclosure_execution(
        cause: Result<WorthQueryProviderSessionDenialKind, Control>,
    ) -> Self {
        let kind = match cause {
            Ok(cause) => Kind::ExecutionRequest(cause),
            Err(Control::Cancelled) => Kind::Cancelled,
            Err(Control::TimedOut) => Kind::DeadlineExceeded,
        };
        Self::new(kind, "original source disclosure")
    }
    pub(in crate::domain_computation::primary_graph) fn source_disclosure_parameters(
        cause: WorthQueryApplicationQueryParameterDenial,
    ) -> Self {
        Self {
            kind: Kind::SourceDisclosureCaptureDenied,
            subject: "original source disclosure parameters".to_owned(),
            cause: AttemptDenialCause::SourceDisclosureParameters(Box::new(cause)),
        }
    }
    pub fn source_disclosure_parameter_denial(
        &self,
    ) -> Option<&WorthQueryApplicationQueryParameterDenial> {
        match &self.cause {
            AttemptDenialCause::SourceDisclosureParameters(cause) => Some(cause),
            _ => None,
        }
    }
}
