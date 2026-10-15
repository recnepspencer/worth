//! Lifetime-free transfer of pre-effect capture refusal; never an effect/recovery engine.
use super::descriptor::{Conditions, SourceQuery};
use super::*;
use crate::application_entry::mutation::request::WorthQueryMutationExpectedSource;
/// Keeps the complete uncaptured original when an adapter's initiating scope ends.
/// It carries no request, principal, grant, runtime handle or active phase.
#[must_use]
pub struct WorthQueryOperationInputCaptureRefusal<
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
> {
    intent: I,
    key: <I::Binding as ApplicationMutationBinding<S>>::IdempotencyKey,
    preconditions: Conditions<S, I>,
    source: Option<WorthQueryMutationExpectedSource<SourceQuery<S, I>>>,
    branch: worth_query_execution::facade::product::WorthQueryProductBranch,
    denial: WorthQueryOperationInputCaptureDenial,
}
#[derive(Debug)]
pub enum WorthQueryOperationInputCaptureRetryDenial {
    FreshRequestMismatch,
    Capture,
}
impl<S: ApplicationSchema, I: ApplicationMutationIntent<S>>
    WorthQueryOperationInputCaptureRefusal<S, I>
{
    /// Last capture diagnostic only; it grants neither retry nor business authority.
    pub fn denial(&self) -> &WorthQueryOperationInputCaptureDenial {
        &self.denial
    }
    pub fn branch(&self) -> worth_query_execution::facade::product::WorthQueryProductBranch {
        self.branch
    }
    /// One fresh native capture, preserving all original fields on every refusal.
    pub fn retry_capture(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        WorthQueryOperationDescription<S, I>,
        (Self, WorthQueryOperationInputCaptureRetryDenial),
    >
    where
        <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
    {
        if self.branch != request.branch
            || !request
                .application
                .product_runtime()
                .owns_branch_selector(self.branch)
        {
            return Err((
                self,
                WorthQueryOperationInputCaptureRetryDenial::FreshRequestMismatch,
            ));
        }
        let original = crate::application_entry::WorthQueryOperationBuilder {
            native: crate::application_entry::mutation::WorthQueryApplicationMutationRequest {
                application: request.application,
                principal: request.principal,
                scope: request.scope,
                branch: self.branch,
                intent: self.intent,
                preconditions: self.preconditions,
                source: self.source,
                source_preparation: std::marker::PhantomData::<WorthQueryMutationSourcePrepared>,
            },
        };
        original
            .idempotency_owned(self.key)
            .map_err(|(builder, key, denial)| {
                (
                    builder.into_capture_refusal(key, denial),
                    WorthQueryOperationInputCaptureRetryDenial::Capture,
                )
            })
    }
}
impl<S: ApplicationSchema, I: ApplicationMutationIntent<S>>
    crate::application_entry::WorthQueryOperationBuilder<
        '_,
        '_,
        '_,
        S,
        I,
        WorthQueryMutationSourcePrepared,
    >
{
    /// Transfers the returned actual pre-effect builder without retaining its borrowed authority.
    /// No fields, identity or completed description are supplied by the caller.
    /// Safety comes from consuming the actual unexecuted builder, not from the
    /// descriptive denial value. Every retry still performs fresh native capture.
    pub fn into_capture_refusal(
        self,
        key: <I::Binding as ApplicationMutationBinding<S>>::IdempotencyKey,
        denial: WorthQueryOperationInputCaptureDenial,
    ) -> WorthQueryOperationInputCaptureRefusal<S, I> {
        WorthQueryOperationInputCaptureRefusal {
            intent: self.native.intent,
            key,
            preconditions: self.native.preconditions,
            source: self.native.source,
            branch: self.native.branch,
            denial,
        }
    }
}

impl<S: ApplicationSchema, I: ApplicationMutationIntent<S>> std::fmt::Debug
    for WorthQueryOperationInputCaptureRefusal<S, I>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OperationInputCaptureRefusal")
            .field("denial", &self.denial)
            .finish_non_exhaustive()
    }
}
