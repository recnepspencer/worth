use worth_proof::{
    AssumptionBasis, AuthorityMarker, AuthorityWitness, CurrentValidity, ExecutedRecipe,
    ExecutionReadyRecipe, FreshnessScopedBasis, LoweredRecipeDxExt, Recipe, ResolvedRecipeDxExt,
    TransitionOutcome, UnresolvedRecipeDxExt,
};

use crate::data::comparator::VersionComparatorPolicy;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::node::EvaluationCondition;

use super::dependency_versions::SignalConditionalDependencyVersions;
use super::{SignalConditionalArtifactReusePolicy, SignalConditionalExecutionRequest};

pub(super) struct SignalConditionalExecutionPayload {
    _graph_instance_id: u64,
    _node: crate::data::handle::NodeId,
}

pub(super) struct SignalConditionalExecutionBasis {
    _graph_instance_id: u64,
    _node: crate::data::handle::NodeId,
    _snapshot_identity: String,
    _execution_identity: String,
    _attempt: u64,
    _condition: EvaluationCondition,
    _dependency_aspects: crate::data::aspect::AspectMask,
    _trigger_aspects: crate::data::aspect::AspectMask,
    _dependency_comparator: VersionComparatorPolicy,
    _output_comparator: VersionComparatorPolicy,
    _artifact_reuse: SignalConditionalArtifactReusePolicy,
    _dependency_versions: SignalConditionalDependencyVersions,
}

struct SignalConditionalResolutionAuthority {
    _private: (),
}

impl AuthorityMarker for SignalConditionalResolutionAuthority {}

struct SignalConditionalReadinessAuthority {
    _private: (),
}

impl AuthorityMarker for SignalConditionalReadinessAuthority {}

pub(super) type SignalConditionalReadyRecipe = ExecutionReadyRecipe<
    SignalConditionalExecutionPayload,
    FreshnessScopedBasis<CurrentValidity, AssumptionBasis<SignalConditionalExecutionBasis>>,
>;

pub(super) type SignalConditionalExecutedRecipe = ExecutedRecipe<
    SignalConditionalExecutionPayload,
    FreshnessScopedBasis<CurrentValidity, AssumptionBasis<SignalConditionalExecutionBasis>>,
>;

pub(super) fn prepare_execution_proof(
    graph: &SignalGraph,
    request: &SignalConditionalExecutionRequest<'_>,
    dependency_versions: &SignalConditionalDependencyVersions,
    work: &mut crate::data::retained_storage::RetainedStoragePreparation,
) -> Result<SignalConditionalReadyRecipe, SignalError> {
    let TransitionOutcome::Success(node) = graph.admit_installed_node(request.contract.node())
    else {
        return Err(SignalError::invalid_input(
            "conditional execution lost its installed Signal node capability",
        ));
    };
    // Installed conditions/comparators carry fixed enums or shared identities.
    // The request strings are the only variable-size copies in this basis.
    super::execution::work::reserve(
        work,
        request
            .snapshot_identity
            .len()
            .checked_add(request.execution_identity.len())
            .and_then(|bytes| bytes.checked_add(12)),
    )?;
    let payload = SignalConditionalExecutionPayload {
        _graph_instance_id: request.contract.graph_instance_id(),
        _node: request.contract.node(),
    };
    let basis = SignalConditionalExecutionBasis {
        _graph_instance_id: request.contract.graph_instance_id(),
        _node: request.contract.node(),
        _snapshot_identity: request.snapshot_identity.to_string(),
        _execution_identity: request.execution_identity.to_string(),
        _attempt: request.attempt,
        _condition: request.contract.condition().clone(),
        _dependency_aspects: request.contract.dependency_aspects(),
        _trigger_aspects: request.contract.trigger_aspects(),
        _dependency_comparator: request.contract.dependency_comparator().clone(),
        _output_comparator: request.contract.output_comparator().clone(),
        _artifact_reuse: request.contract.artifact_reuse().clone(),
        _dependency_versions: std::sync::Arc::clone(dependency_versions),
    };
    Ok(Recipe::new(payload)
        .resolve_with(
            AuthorityWitness::from_authority_marker(SignalConditionalResolutionAuthority {
                _private: (),
            }),
            basis,
        )
        .lower_with(node.lowering_witness())
        .ready_with(
            AuthorityWitness::from_authority_marker(SignalConditionalReadinessAuthority {
                _private: (),
            }),
            graph.runtime_instance_id(),
        ))
}
