//! A cold observation reads the same immutable causal row parser as a live original.
use super::*;
impl WorthQueryPrimaryGraphProvider {
    pub fn observe_correction_original_fact(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &worth_runtime_world::facade::WorldOriginalStateObservation<'_>,
        commit: &worth_relational::facade::history::RelationalCommitReceipt,
    ) -> Result<Option<super::super::WorthQueryCanonicalCorrectionFact>, Denial> {
        if original.fate() != worth_foundational::facade::WorldOriginalFate::Published {
            return Err(Denial::OriginalMismatch);
        }
        self.graph.with_runtime(request, |native| {
            let subject = native
                .durability_authority()
                .discover_associated_effect(request, commit)
                .map_err(super::native_source::denial)?
                .ok_or(Denial::NativeSourceUnavailable)?;
            let effect = super::native_source::read_subject(request, native, original, &subject)?;
            self.readmitted_original_causality(
                request,
                native,
                &effect,
                worth_foundational::facade::WorldOwnerCoordinate::Home {
                    owner: original.attempt().owner(),
                },
            )
            .map_err(|_| Denial::OriginalMismatch)
        })
    }
}
