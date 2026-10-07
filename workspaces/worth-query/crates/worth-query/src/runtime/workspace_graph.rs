use super::{
    WorthQueryBatchWriteReceipt, WorthQueryExistingTruthAssertionDenialKind,
    WorthQueryExistingTruthBindingDenialKind, WorthQueryGraphCompositionBuilder,
    WorthQueryGraphCompositionDenialKind, WorthQueryRuntimeError,
    WorthQuerySymbolicTargetReferenceDenialKind, WorthQueryWorkspace,
};
use crate::runtime::mutation::graph_composition_error;

impl WorthQueryWorkspace {
    pub fn compose_graph(
        &mut self,
        declaration: impl FnOnce(
            &mut WorthQueryGraphCompositionBuilder,
        ) -> Result<(), WorthQueryRuntimeError>,
    ) -> Result<WorthQueryBatchWriteReceipt, WorthQueryRuntimeError> {
        let mut builder = WorthQueryGraphCompositionBuilder::new();
        declaration(&mut builder)?;
        let (commands, breadth, program) = builder.finish()?;
        match self.runtime.write_graph_batch(commands, breadth, program) {
            Err(WorthQueryRuntimeError::ExistingTruthAssertionDenied(denial)) => {
                let kind = match denial.kind() {
                    WorthQueryExistingTruthAssertionDenialKind::Handle(handle) => return Err(handle.into()),
                    WorthQueryExistingTruthAssertionDenialKind::BackendVerificationUnsupported => {
                        WorthQueryGraphCompositionDenialKind::ExistingTargetBackendVerificationUnsupported
                    }
                    WorthQueryExistingTruthAssertionDenialKind::ClearAssertionUnsupported => {
                        WorthQueryGraphCompositionDenialKind::ExistingTargetClearAssertionUnsupported
                    }
                    WorthQueryExistingTruthAssertionDenialKind::MissingAssertedAspect => {
                        WorthQueryGraphCompositionDenialKind::ExistingTargetMissingAssertedAspect
                    }
                    WorthQueryExistingTruthAssertionDenialKind::AssertedValueMismatch => {
                        WorthQueryGraphCompositionDenialKind::ExistingTargetAssertedValueMismatch
                    }
                };
                Err(graph_composition_error(
                    kind,
                    None,
                    denial.binding().target_collection_identity().cloned(),
                    denial.message().to_string(),
                ))
            }
            Err(WorthQueryRuntimeError::MutationBindingDenied(denial)) => {
                let kind = match denial.kind() {
                    WorthQueryExistingTruthBindingDenialKind::UnsupportedFamily => {
                        WorthQueryGraphCompositionDenialKind::ExistingTargetBindingUnsupported
                    }
                    WorthQueryExistingTruthBindingDenialKind::ResolvedTargetMissing => {
                        WorthQueryGraphCompositionDenialKind::ExistingTargetResolvedTargetMissing
                    }
                    WorthQueryExistingTruthBindingDenialKind::CollectionMismatch => {
                        WorthQueryGraphCompositionDenialKind::ExistingTargetCollectionMismatch
                    }
                };
                Err(graph_composition_error(
                    kind,
                    None,
                    denial.binding().target_collection_identity().cloned(),
                    denial.message().to_string(),
                ))
            }
            Err(WorthQueryRuntimeError::MutationTargetReferenceDenied(denial)) => {
                let kind = match denial.kind() {
                    WorthQuerySymbolicTargetReferenceDenialKind::UnresolvedSameBatchTarget => {
                        WorthQueryGraphCompositionDenialKind::UnresolvedSymbolicReference
                    }
                    WorthQuerySymbolicTargetReferenceDenialKind::CollectionMismatch => {
                        WorthQueryGraphCompositionDenialKind::SymbolicCollectionMismatch
                    }
                    WorthQuerySymbolicTargetReferenceDenialKind::RequiresBatchContext => {
                        return Err(WorthQueryRuntimeError::MutationTargetReferenceDenied(
                            denial,
                        ));
                    }
                    WorthQuerySymbolicTargetReferenceDenialKind::NonEntityReferenceTarget => {
                        return Err(WorthQueryRuntimeError::MutationTargetReferenceDenied(
                            denial,
                        ));
                    }
                };
                Err(graph_composition_error(
                    kind,
                    Some(denial.reference().symbol().to_string()),
                    denial.reference().target_collection_identity().cloned(),
                    denial.message().to_string(),
                ))
            }
            other => other,
        }
    }
}
