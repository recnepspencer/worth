//! A cold observation reads the same immutable causal row parser as a live original.
use super::*;
impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn observe_correction_original_fact(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &worth_runtime_world::facade::WorldOriginalStateObservation<'_>,
        commit: &worth_relational::facade::history::RelationalCommitReceipt,
    ) -> Result<Option<super::super::WorthQueryCanonicalCorrectionFact>, Denial> {
        if original.fate() != worth_foundational::facade::WorldOriginalFate::Published {
            return Err(Denial::OriginalMismatch);
        }
        self.graph.with_runtime(request, |native| {
            let subjects = native
                .durability_authority()
                .discover_all_associated_effects(request)
                .map_err(super::native_source::denial)?;
            let mut matching = subjects
                .iter()
                .filter(|subject| subject.receipt() == commit);
            let subject = matching.next().ok_or(Denial::NativeSourceUnavailable)?;
            if matching.next().is_some() {
                return Err(Denial::OriginalMismatch);
            }
            let effect = super::native_source::read_subject(request, native, original, subject)?;
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
