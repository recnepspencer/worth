//! Domain write refusals preserve the owner cause or original execution cause.
use super::{
    denial_cause::DenialCause as Cause, WorthQueryApplicationCommitDenial as Denial,
    WorthQueryApplicationCommitDenialKind as Kind, WorthQueryApplicationCommitDenialStage as Stage,
};
use crate::domain_computation::primary_graph::WorthQueryValueDecodeDenial as Admission;
use worth_query_installation::facade::ApplicationValueDecodeDenial;
impl Denial {
    pub(in crate::domain_computation::primary_graph) fn domain_value_rejected(
        cause: ApplicationValueDecodeDenial,
    ) -> Self {
        Self {
            kind: Kind::DomainValueRejected,
            stage: Stage::EffectLowering,
            detail: None,
            cause: Some(Box::new(Cause::DomainValue(cause))),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn domain_value_admission_rejected(
        cause: Admission,
    ) -> Self {
        if let Some(request) = cause.request_cause() {
            return Self::provider_execution_denied(Stage::EffectLowering,
                crate::domain_computation::primary_graph::provider::relational_execution_denial::relational_execution_kind(request.into(), None),
                "domain value write admission refused the original request");
        }
        Self {
            kind: Kind::DomainValueRejected,
            stage: Stage::EffectLowering,
            detail: None,
            cause: Some(Box::new(Cause::DomainValueAdmission(cause))),
        }
    }
    /// The domain owner's exact codec refusal, including its binding identity.
    pub fn domain_value_decode_denial(&self) -> Option<&ApplicationValueDecodeDenial> {
        match self.cause.as_deref() {
            Some(Cause::DomainValue(cause)) => Some(cause),
            _ => None,
        }
    }
    /// An internal admission-contract refusal; resource stops use execution_denial_cause.
    pub fn domain_value_admission_denial(&self) -> Option<Admission> {
        match self.cause.as_deref() {
            Some(Cause::DomainValueAdmission(cause)) => Some(*cause),
            _ => None,
        }
    }
}
