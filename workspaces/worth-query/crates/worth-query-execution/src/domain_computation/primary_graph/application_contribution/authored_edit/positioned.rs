//! Ordered authoring transfers the original C admission, never re-admitting at D.
use super::capture::TypedStage;
use super::*;
use worth_query_declaration::facade::application_schema::ApplicationReadableScalarValueBinding;

impl<S: ApplicationSchema, B: ApplicationMutationBinding<S>> TypedStage<S, B> {
    pub(super) fn bind_corrective_identity(
        &self,
        binding: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryApplicationIdempotencyBinding {
        self.correction
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .map_or(binding, |intent| intent.bind_idempotency(binding))
    }
    pub(super) fn execution_admission(
        &self,
        context: &AuthoredEditStageContext<'_, '_, '_, S>,
    ) -> Result<AuthoredEditStageAdmission<S, B>, WorthQueryOutputDemandDenial> {
        if !self.ordered.load(std::sync::atomic::Ordering::Relaxed) {
            return (self.admit)(context);
        }
        let refuse = |detail: String| {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                format!("stage {} {}: {detail}", self.ordinal, B::IDENTITY),
            )
        };
        // This scalar is data retained beside the genuine original admission.
        let principal = B::PrincipalIdentityBinding::decode(&self.principal_value)
            .map_err(|cause| refuse(format!("retained principal identity: {cause:?}")))?;
        let operation = self
            .original_admission
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
            .ok_or_else(|| refuse("the positioned candidate was already consumed".into()))?;
        Ok(AuthoredEditStageAdmission {
            operation,
            principal,
        })
    }
}
