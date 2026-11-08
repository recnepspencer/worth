//! Pre-service capture of the actual program's Signal and Bridge declarations.
use super::BridgeConditionalRuntimeBuilder;
use crate::conditional_execution::authored_section::{
    capture::Capture, BridgeAuthoredSectionDenial as D,
};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    DurableSignalBranchIdentity, DurableSignalConditionalDefinitionOccurrence,
    SignalDefinitionTransitionIdentity,
};
use worth_signal::facade::durability::{
    SignalDefinitionTransitionArtifact, SignalDurableTierVocabulary,
};

impl BridgeConditionalRuntimeBuilder {
    /// Capture the initial authored definition body before installation is sealed.
    /// Identities describe the enclosing Home; this body grants no publication authority.
    pub fn capture_authored_definition(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        identity: SignalDefinitionTransitionIdentity,
        branch: DurableSignalBranchIdentity,
        occurrences: &[DurableSignalConditionalDefinitionOccurrence],
    ) -> Result<SignalDefinitionTransitionArtifact, D> {
        let runtime = self.runtime.lock_signal_runtime();
        let lowerings = self
            .runtime
            .conditional_lowerings
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let bytes = lowerings
            .len()
            .checked_mul(std::mem::size_of::<
                &crate::conditional_execution::BridgeInstalledConditionalLowering,
            >())
            .and_then(|size| u64::try_from(size).ok())
            .ok_or(D::SizeLimit)?;
        let _preparation = execution
            .reserve_retained_memory(bytes)
            .map_err(D::Request)?;
        let mut definitions: Vec<_> = lowerings.values().map(std::sync::Arc::as_ref).collect();
        definitions.sort_by_key(|lowering| lowering.signal_node());
        if definitions
            .windows(2)
            .any(|pair| pair[0].signal_node() == pair[1].signal_node())
            || definitions.len() != occurrences.len()
        {
            return Err(D::SignalDefinitionMismatch);
        }
        let section = Capture {
            bridge: &self.runtime.bridge,
            graph: runtime.graph(),
            branch,
            transition: identity,
            occurrences,
        }
        .capture(execution, &definitions)?;
        let image = runtime
            .graph()
            .capture_authored_image(execution, branch)
            .map_err(D::SignalImage)?;
        let vocabulary =
            SignalDurableTierVocabulary::declare([((), "bridge")]).map_err(|_| D::Malformed)?;
        let config = runtime
            .config()
            .capture_authored_image(runtime.graph(), &vocabulary, branch, execution)
            .map_err(D::SignalImage)?;
        SignalDefinitionTransitionArtifact::capture(
            execution,
            identity,
            branch,
            None,
            &image,
            &config,
            occurrences,
            section.as_signal_section(),
        )
        .map_err(D::SignalCarrier)
    }
}
