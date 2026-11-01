use super::*;
use worth_query_execution::{
    facade::product::{
        WorthQueryApplicationProductBranchCleanupFailure,
        WorthQueryApplicationProductBranchCloseDenial, WorthQueryProductBranch,
    },
    publication_boundary::document_history::WorthQueryDocumentHistoryCloseFailure,
};

#[must_use = "retain the live document or retry its actual retired cleanup owner"]
pub enum WorthQueryApplicationHistoryCloseFailure<
    'runtime,
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H> = OrdinaryHistoryPayload,
    Outputs = WorthQueryHistoryNoOutputs,
> {
    Retained {
        document: WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
        denial: WorthQueryApplicationProductBranchCloseDenial,
    },
    Retired {
        branch: WorthQueryProductBranch,
        failure: WorthQueryApplicationProductBranchCleanupFailure,
    },
    Closing(WorthQueryDocumentHistoryClosing<'runtime, S>),
    ClosingOutputs(WorthQueryHistoryClosingOutputs<'runtime, S, H, Payload, Outputs>),
}

/// Only disposal is available while SDK output custody still owns a live document.
/// A close remainder cannot read the retained document.
/// ```compile_fail,E0599
/// use worth_query_publication::facade::application_entry::*;
/// use worth_query_declaration::facade::{application_history::*, application_query::*, application_schema::*};
/// use worth_query_execution::facade::primary_graph::WorthQueryApplicationProjection;
/// type B<S, H> = ApplicationHistoryAccessBinding<S, H>;
/// type Q<S, H> = <B<S, H> as ApplicationQueryBinding<S>>::Query;
/// type V<S, H> = <<B<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
/// fn read<S, H, O>(owner: &WorthQueryHistoryClosingOutputs<'_, S, H, OrdinaryHistoryPayload, O>,
///     request: &WorthQueryApplicationRequest<'_, '_, '_, S>)
/// where S: ApplicationSchema, H: ApplicationHistoryBinding<S>,
///     <B<S, H> as ApplicationQueryBinding<S>>::ScopeBinding: ApplicationQueryScopeResolution<S,
///         <B<S, H> as ApplicationQueryBinding<S>>::PrincipalIdentity>,
///     V<S, H>: WorthQueryApplicationProjection<S, Q<S, H>>,
/// { let _ = owner.read(request); }
/// ```
/// Read a live document, or retry disposal using the retained close owner.
/// ```
/// use worth_query_publication::facade::application_entry::*;
/// use worth_query_declaration::facade::{application_history::*, application_query::*, application_schema::*};
/// use worth_query_execution::facade::{primary_graph::WorthQueryApplicationProjection,
///     application_contribution::WorthQueryAdvancementPhase};
/// type B<S, H> = ApplicationHistoryAccessBinding<S, H>;
/// type Q<S, H> = <B<S, H> as ApplicationQueryBinding<S>>::Query;
/// type V<S, H> = <<B<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
/// fn read_live<S, H, O>(owner: &WorthQueryApplicationHistoryDocument<'_, S, H, OrdinaryHistoryPayload, O>,
///     request: &WorthQueryApplicationRequest<'_, '_, '_, S>)
/// where S: ApplicationSchema, H: ApplicationHistoryBinding<S>,
///     <B<S, H> as ApplicationQueryBinding<S>>::ScopeBinding: ApplicationQueryScopeResolution<S,
///         <B<S, H> as ApplicationQueryBinding<S>>::PrincipalIdentity>,
///     V<S, H>: WorthQueryApplicationProjection<S, Q<S, H>>,
/// { let _ = owner.read(request); }
/// fn retry<'r, S, H, O>(owner: WorthQueryHistoryClosingOutputs<'r, S, H, OrdinaryHistoryPayload, O>,
///     phase: &WorthQueryAdvancementPhase<'_>)
///     -> Result<WorthQueryDocumentHistoryCloseReceipt,
///         WorthQueryApplicationHistoryCloseFailure<'r, S, H, OrdinaryHistoryPayload, O>>
/// where S: ApplicationSchema, H: ApplicationHistoryBinding<S>, O: WorthQueryHistoryOutputsMode<S, H>
/// { owner.retry(phase) }
/// ```
#[must_use = "retry disposal before releasing the retained document"]
pub struct WorthQueryHistoryClosingOutputs<
    'runtime,
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H> = OrdinaryHistoryPayload,
    Outputs = WorthQueryHistoryNoOutputs,
> {
    document: WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
    denial: WorthQueryHistoryOutputDenial,
}

impl<
        'runtime,
        S: ApplicationSchema,
        H: ApplicationHistoryBinding<S>,
        Payload: WorthQueryHistoryPayloadMode<S, H>,
        Outputs: WorthQueryHistoryOutputsMode<S, H>,
    > WorthQueryHistoryClosingOutputs<'runtime, S, H, Payload, Outputs>
{
    pub fn denial(&self) -> &WorthQueryHistoryOutputDenial {
        &self.denial
    }
    pub fn retry(
        self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<
        WorthQueryDocumentHistoryCloseReceipt,
        WorthQueryApplicationHistoryCloseFailure<'runtime, S, H, Payload, Outputs>,
    > {
        self.document.close(phase)
    }
}

impl<
        'runtime,
        S: ApplicationSchema,
        H: ApplicationHistoryBinding<S>,
        Payload: WorthQueryHistoryPayloadMode<S, H>,
        Outputs: WorthQueryHistoryOutputsMode<S, H>,
    > WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>
{
    pub fn close(
        mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<
        WorthQueryDocumentHistoryCloseReceipt,
        WorthQueryApplicationHistoryCloseFailure<'runtime, S, H, Payload, Outputs>,
    > {
        if let Err(denial) = self.outputs.close(phase) {
            return Err(WorthQueryApplicationHistoryCloseFailure::ClosingOutputs(
                WorthQueryHistoryClosingOutputs {
                    document: self,
                    denial,
                },
            ));
        }
        let Self {
            owner,
            access,
            payload,
            outputs,
        } = self;
        match owner.close(phase) {
            Err(WorthQueryDocumentHistoryCloseFailure::Closing(closing)) => {
                Err(WorthQueryApplicationHistoryCloseFailure::Closing(closing))
            }
            Ok(receipt) => Ok(receipt),
            Err(WorthQueryDocumentHistoryCloseFailure::Retained {
                document: owner,
                denial,
            }) => Err(WorthQueryApplicationHistoryCloseFailure::Retained {
                document: Self {
                    owner,
                    access,
                    payload,
                    outputs,
                },
                denial,
            }),
            Err(WorthQueryDocumentHistoryCloseFailure::Retired { branch, failure }) => {
                Err(WorthQueryApplicationHistoryCloseFailure::Retired { branch, failure })
            }
        }
    }
}

impl<
        S: ApplicationSchema,
        H: ApplicationHistoryBinding<S>,
        Payload: WorthQueryHistoryPayloadMode<S, H>,
        Outputs,
    > std::fmt::Debug for WorthQueryApplicationHistoryCloseFailure<'_, S, H, Payload, Outputs>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ClosingOutputs(_) => f.debug_tuple("ClosingOutputs").finish(),
            Self::Closing(closing) => f.debug_tuple("Closing").field(closing).finish(),
            Self::Retained { denial, .. } => f.debug_tuple("Retained").field(denial).finish(),
            Self::Retired { branch, failure } => f
                .debug_struct("Retired")
                .field("branch", branch)
                .field("failure", failure)
                .finish(),
        }
    }
}
