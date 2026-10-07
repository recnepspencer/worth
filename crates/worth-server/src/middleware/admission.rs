use super::WorthServerPipelineIntent;
use crate::WorthServerResolvedRequestContext;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerAdmission {
    resolved_request_context: WorthServerResolvedRequestContext,
    prepared_query_handoff_intent: WorthServerPreparedQueryHandoffIntent,
}

impl WorthServerAdmission {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        self.resolved_request_context
            .owned_allocation_capacity_bytes()
            .saturating_add(
                self.prepared_query_handoff_intent
                    .owned_allocation_capacity_bytes(),
            )
    }

    pub(crate) fn new(
        resolved_request_context: WorthServerResolvedRequestContext,
        prepared_query_handoff_intent: WorthServerPreparedQueryHandoffIntent,
    ) -> Self {
        Self {
            resolved_request_context,
            prepared_query_handoff_intent,
        }
    }

    pub fn request_context(&self) -> &crate::WorthServerRequestContext {
        self.resolved_request_context.request_context()
    }

    pub fn resolved_request_context(&self) -> &WorthServerResolvedRequestContext {
        &self.resolved_request_context
    }

    pub fn query_handoff_intent(&self) -> &WorthServerPreparedQueryHandoffIntent {
        &self.prepared_query_handoff_intent
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthServerPreparedQueryHandoffKind {
    WorthNativeSession,
    QueryRead,
    QueryMutation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerPreparedQueryHandoffIntent {
    kind: WorthServerPreparedQueryHandoffKind,
    operation_name: String,
}

impl WorthServerPreparedQueryHandoffIntent {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        crate::product_adapter::execution_pipeline::read_batch_accounting::string(
            &self.operation_name,
        )
    }

    pub(crate) fn from_pipeline_intent(intent: WorthServerPipelineIntent) -> Self {
        match intent {
            WorthServerPipelineIntent::WorthNativeSession { operation_name } => Self {
                kind: WorthServerPreparedQueryHandoffKind::WorthNativeSession,
                operation_name,
            },
            WorthServerPipelineIntent::QueryRead { operation_name } => Self {
                kind: WorthServerPreparedQueryHandoffKind::QueryRead,
                operation_name,
            },
            WorthServerPipelineIntent::QueryMutation { operation_name } => Self {
                kind: WorthServerPreparedQueryHandoffKind::QueryMutation,
                operation_name,
            },
        }
    }

    pub fn kind(&self) -> WorthServerPreparedQueryHandoffKind {
        self.kind
    }

    pub fn operation_name(&self) -> &str {
        &self.operation_name
    }
}
