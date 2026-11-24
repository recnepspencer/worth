use worth_foundational::{
    FoundationalMergeIntent, FoundationalMergeScope, FoundationalMergeScopeFamily,
};

use crate::merge::data::requests::normalized::NormalizedRelationalMergeRequest;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalFoundationalMergeRequest {
    normalized_request: NormalizedRelationalMergeRequest,
    foundational_scope: FoundationalMergeScope,
    foundational_intent: FoundationalMergeIntent,
    lowering_digest: String,
}

impl RelationalFoundationalMergeRequest {
    pub(crate) fn new(
        normalized_request: NormalizedRelationalMergeRequest,
        foundational_scope: FoundationalMergeScope,
        foundational_intent: FoundationalMergeIntent,
    ) -> Self {
        let lowering_digest = format!(
            "{}::{:?}::{:?}",
            normalized_request.request_digest(),
            foundational_scope.family(),
            foundational_intent
        );
        Self {
            normalized_request,
            foundational_scope,
            foundational_intent,
            lowering_digest,
        }
    }

    pub fn normalized_request(&self) -> &NormalizedRelationalMergeRequest {
        &self.normalized_request
    }

    pub fn foundational_scope(&self) -> &FoundationalMergeScope {
        &self.foundational_scope
    }

    pub fn foundational_scope_family(&self) -> FoundationalMergeScopeFamily {
        self.foundational_scope.family()
    }

    pub fn foundational_intent(&self) -> FoundationalMergeIntent {
        self.foundational_intent
    }

    pub fn lowering_digest(&self) -> &str {
        &self.lowering_digest
    }
}
