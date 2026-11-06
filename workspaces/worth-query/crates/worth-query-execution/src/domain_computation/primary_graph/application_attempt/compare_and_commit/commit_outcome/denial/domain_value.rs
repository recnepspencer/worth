//! Domain write refusals preserve the owner cause or original execution cause.
use crate::domain_computation::vocabulary::primary_graph::value_decode::WorthQueryValueDecodeDenial as Admission;
use worth_query_installation::facade::ApplicationValueDecodeDenial;
use {
    super::denial_cause::DenialCause as Cause, super::WorthQueryApplicationCommitDenial as Denial,
    super::WorthQueryApplicationCommitDenialStage as Stage,
    crate::domain_computation::primary_graph::vocabulary::application_attempt::denial::WorthQueryApplicationCommitDenialKind as Kind,
};
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
                crate::domain_computation::primary_graph::vocabulary::provider::relational_execution_denial::relational_execution_kind(request.into(), None),
                "domain value write admission refused the original request");
        }
        Self {
            kind: Kind::DomainValueRejected,
            stage: Stage::EffectLowering,
            detail: None,
            cause: Some(Box::new(Cause::DomainValueAdmission(cause))),
        }
    }
    /// Exact Native content failure at the single pre-publication field seal.
    pub fn content_denial(
        &self,
    ) -> Option<worth_relational::facade::content::RelationalContentDenial> {
        match self.domain_value_admission_denial() {
            Some(Admission::Content(cause)) => Some(cause),
            _ => None,
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
