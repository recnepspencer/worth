use serde::{Deserialize, Serialize};

use super::execution_summary::StrategyExecutionSummary;
use super::mutation_program::StrategyMutationProgram;
use super::output_artifact::CanonicalStrategyOutputArtifact;
use crate::commit_strategies::data::descriptor::descriptor_record::CommitStrategyDescriptorDigest;
use crate::commit_strategies::data::request::{
    CanonicalStrategyCommitRequest, CanonicalStrategyInputDigest,
};
use crate::commit_strategies::data::strategy_id::CommitStrategyId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyRequestBinding {
    strategy_id: CommitStrategyId,
    descriptor_digest: CommitStrategyDescriptorDigest,
    input_digest: CanonicalStrategyInputDigest,
}

impl StrategyRequestBinding {
    fn from_request(request: &CanonicalStrategyCommitRequest) -> Self {
        Self {
            strategy_id: request.strategy_id(),
            descriptor_digest: request.descriptor_digest(),
            input_digest: request.canonical_input().digest(),
        }
    }

    pub fn strategy_id(&self) -> CommitStrategyId {
        self.strategy_id
    }

    pub fn descriptor_digest(&self) -> CommitStrategyDescriptorDigest {
        self.descriptor_digest
    }

    pub fn input_digest(&self) -> CanonicalStrategyInputDigest {
        self.input_digest
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyExecutionResult {
    output: CanonicalStrategyOutputArtifact,
    mutation_program: StrategyMutationProgram,
}

impl StrategyExecutionResult {
    pub fn new(
        output: CanonicalStrategyOutputArtifact,
        mutation_program: StrategyMutationProgram,
    ) -> Self {
        Self {
            output,
            mutation_program,
        }
    }

    pub fn output(&self) -> &CanonicalStrategyOutputArtifact {
        &self.output
    }

    pub fn mutation_program(&self) -> &StrategyMutationProgram {
        &self.mutation_program
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyExecutionDraft {
    request_binding: StrategyRequestBinding,
    output: CanonicalStrategyOutputArtifact,
    mutation_program: StrategyMutationProgram,
    summary: StrategyExecutionSummary,
}

impl StrategyExecutionDraft {
    pub fn request_binding(&self) -> &StrategyRequestBinding {
        &self.request_binding
    }

    pub fn output(&self) -> &CanonicalStrategyOutputArtifact {
        &self.output
    }

    pub fn mutation_program(&self) -> &StrategyMutationProgram {
        &self.mutation_program
    }

    pub fn summary(&self) -> StrategyExecutionSummary {
        self.summary
    }
}

/// Operations used by the Relational owners.
pub trait StrategyExecutionDraftOwnerPort: Sized {
    fn from_measured_result(
        request: &CanonicalStrategyCommitRequest,
        result: StrategyExecutionResult,
        summary: StrategyExecutionSummary,
    ) -> Self;
}

impl StrategyExecutionDraftOwnerPort for StrategyExecutionDraft {
    fn from_measured_result(
        request: &CanonicalStrategyCommitRequest,
        result: StrategyExecutionResult,
        summary: StrategyExecutionSummary,
    ) -> Self {
        Self {
            request_binding: StrategyRequestBinding::from_request(request),
            output: result.output,
            mutation_program: result.mutation_program,
            summary,
        }
    }
}
