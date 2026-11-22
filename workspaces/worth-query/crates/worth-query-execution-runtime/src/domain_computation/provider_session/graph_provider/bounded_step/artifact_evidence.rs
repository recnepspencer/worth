use std::sync::Arc;

use crate::domain_computation::artifact_owner::{
    WorthQueryArtifactOccurrenceLedger, WorthQueryArtifactOccurrenceScope,
    WorthQueryArtifactOccurrenceSnapshot,
};
use crate::domain_computation::{
    WorthQueryArtifactDenial, WorthQueryArtifactProductionAuthority,
    WorthQueryArtifactProductionEvidence, WorthQueryArtifactProviderResource,
    WorthQueryMoveOnlyArtifactHandle,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryGraphProviderStepArtifactEvidence {
    produced_artifact_count: usize,
    retained_artifact_count: usize,
    disposed_artifact_count: usize,
    retained_bytes: usize,
}

#[derive(Clone)]
pub struct WorthQueryGraphProviderStepArtifactContext {
    authority: Arc<WorthQueryArtifactProductionAuthority>,
    occurrences: WorthQueryArtifactOccurrenceScope,
}

impl WorthQueryGraphProviderStepArtifactEvidence {
    pub const fn produced_artifact_count(self) -> usize {
        self.produced_artifact_count
    }

    pub const fn retained_artifact_count(self) -> usize {
        self.retained_artifact_count
    }

    pub const fn disposed_artifact_count(self) -> usize {
        self.disposed_artifact_count
    }

    pub const fn retained_bytes(self) -> usize {
        self.retained_bytes
    }
}

pub(super) struct WorthQueryGraphProviderStepArtifacts {
    context: Option<WorthQueryGraphProviderStepArtifactContext>,
    before: WorthQueryArtifactOccurrenceSnapshot,
}

impl WorthQueryGraphProviderStepArtifactContext {
    pub fn new(
        authority: Arc<WorthQueryArtifactProductionAuthority>,
        managed_run_occurrences: Arc<WorthQueryArtifactOccurrenceLedger>,
    ) -> Self {
        Self {
            authority,
            occurrences: WorthQueryArtifactOccurrenceScope::for_managed_run(
                managed_run_occurrences,
            ),
        }
    }
}

impl WorthQueryGraphProviderStepArtifacts {
    pub(super) fn new(context: Option<WorthQueryGraphProviderStepArtifactContext>) -> Self {
        let before = context
            .as_ref()
            .map_or_else(WorthQueryArtifactOccurrenceSnapshot::default, |context| {
                context.occurrences.call_snapshot()
            });
        Self { context, before }
    }

    pub(super) fn produce<R: WorthQueryArtifactProviderResource>(
        &self,
        evidence: WorthQueryArtifactProductionEvidence,
        resource: R,
    ) -> Result<WorthQueryMoveOnlyArtifactHandle, WorthQueryArtifactDenial> {
        let context = self.context.as_ref().ok_or_else(|| {
            WorthQueryArtifactDenial::new(
                crate::domain_computation::WorthQueryArtifactDenialKind::ArtifactContractNotInstalled,
                None,
                "bounded provider step has no installed artifact production contract",
            )
        })?;
        let admission = WorthQueryArtifactProductionAuthority::admit(&context.authority, evidence);
        WorthQueryArtifactProductionAuthority::register_tracked(
            &context.authority,
            admission,
            resource,
            context.occurrences.clone(),
        )
    }

    pub(super) fn finish(&self) -> WorthQueryGraphProviderStepArtifactEvidence {
        let Some(context) = &self.context else {
            return WorthQueryGraphProviderStepArtifactEvidence::default();
        };
        let after = context.occurrences.call_snapshot();
        WorthQueryGraphProviderStepArtifactEvidence {
            produced_artifact_count: after
                .produced_artifact_count()
                .saturating_sub(self.before.produced_artifact_count()),
            retained_artifact_count: after.retained_artifact_count(),
            disposed_artifact_count: after
                .disposed_artifact_count()
                .saturating_sub(self.before.disposed_artifact_count()),
            retained_bytes: after.retained_bytes(),
        }
    }
}
