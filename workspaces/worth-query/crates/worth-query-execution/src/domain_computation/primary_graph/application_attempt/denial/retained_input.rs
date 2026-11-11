//! Original canonical retention preserves the codec's exact typed refusal.
use super::{
    AttemptDenialCause, WorthQueryApplicationAttemptDenial as Denial,
    WorthQueryApplicationAttemptDenialKind as Kind,
};
use worth_query_declaration::facade::application_operation::ApplicationRetainedInputDenial;
impl Denial {
    pub(in crate::domain_computation::primary_graph) fn retained_input(
        cause: ApplicationRetainedInputDenial,
    ) -> Self {
        Self::from_cause(
            Kind::CorrectionCaptureDenied,
            "original canonical input retention".into(),
            AttemptDenialCause::RetainedInput(Box::new(cause)),
        )
    }
    pub fn retained_input_denial(&self) -> Option<&ApplicationRetainedInputDenial> {
        match self.cause() {
            Some(AttemptDenialCause::RetainedInput(cause)) => Some(cause),
            _ => None,
        }
    }
}
