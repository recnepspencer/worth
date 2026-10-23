//! Reify one source's installed association for a non-authorizing compatibility report.
use super::*;

pub(super) fn inspect_source<Schema: ApplicationSchema>(
    target: &WorthQuerySelectedProductOperation<'_, Schema>,
    phase: &WorthQueryAdvancementPhase<'_>,
    principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    source: &WorthQuerySelectedCorrectionSource<'_, Schema>,
    maximum_history: Option<NonZeroUsize>,
    maximum_results: NonZeroUsize,
) -> Result<(), WorthQueryCorrectionDerivationDenial> {
    let binding = source
        .occurrence()
        .commit_evidence()
        .original_authority()
        .original_mutation_binding()
        .ok_or(WorthQueryCorrectionDerivationDenial::UnsupportedSourceMeaning)?;
    Schema::visit_correction(
        binding,
        InspectSource {
            target,
            phase,
            principal,
            request,
            source,
            maximum_history,
            maximum_results,
        },
    )
}

struct InspectSource<'a, 'runtime, 'phase, Schema> {
    target: &'a WorthQuerySelectedProductOperation<'runtime, Schema>,
    phase: &'a WorthQueryAdvancementPhase<'phase>,
    principal: &'a WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &'a WorthQueryRequestScope,
    source: &'a WorthQuerySelectedCorrectionSource<'runtime, Schema>,
    maximum_history: Option<NonZeroUsize>,
    maximum_results: NonZeroUsize,
}
impl<Schema: ApplicationSchema> ApplicationCorrectionVisitor<Schema>
    for InspectSource<'_, '_, '_, Schema>
{
    type Output = Result<(), WorthQueryCorrectionDerivationDenial>;
    fn unavailable(self) -> Self::Output {
        Err(WorthQueryCorrectionDerivationDenial::UnsupportedSourceMeaning)
    }
    fn declared<C: ApplicationCorrectionBinding<Schema>>(self) -> Self::Output {
        disclosure::disclose_correction_source::<Schema, C>(
            self.target,
            self.phase,
            self.principal,
            self.request,
            self.source,
            self.maximum_history,
            self.maximum_results,
        )?;
        compatibility::require_compatible_source::<Schema, C>(
            self.target,
            self.phase,
            self.source,
            None,
        )
        .map(|_| ())
    }
}
