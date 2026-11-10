use crate::application::{
    review_declaration_legality, worth_query_checked_family_declaration,
    worth_query_checked_family_support, WorthQueryCanonicalDeclarationArtifact,
    WorthQueryDeclarationAdmissionError, WorthQueryDeclarationAdmissionOrLegalityError,
    WorthQueryDeclarationCanonicalizationVersion, WorthQueryDeclarationFamilyMarker,
    WorthQueryDeclarationFamilySupportChecked, WorthQueryDeclarationFamilySupportReport,
    WorthQueryDeclarationInput, WorthQueryDeclarationLegalityChecked,
    WorthQueryDeclarationLegalityDenial, WorthQueryDeclarationLegalityEvidence,
    WorthQueryDeclarationLegalityInput, WorthQueryDeclaredFamilyChecked,
    WorthQueryDomainEntryMarker,
};

use super::WorthQueryInstalledDomainDeclarationContext;
use crate::application::WorthQueryDomainOperatingContext;
use crate::runtime::WorthQueryRuntimeFacadeFamily;

impl<D: WorthQueryDomainEntryMarker, C: WorthQueryDomainOperatingContext<D>>
    WorthQueryInstalledDomainDeclarationContext<D, C>
{
    pub fn family_support<F>(&self) -> WorthQueryDeclarationFamilySupportReport<D, F>
    where
        F: WorthQueryDeclarationFamilyMarker<D>,
    {
        match self.family_support_checked::<F>() {
            WorthQueryDeclarationFamilySupportChecked::Admitted(report)
            | WorthQueryDeclarationFamilySupportChecked::Deferred(report)
            | WorthQueryDeclarationFamilySupportChecked::Unsupported(report)
            | WorthQueryDeclarationFamilySupportChecked::InvalidContext(report) => report,
        }
    }

    pub fn family_support_checked<F>(&self) -> WorthQueryDeclarationFamilySupportChecked<D, F>
    where
        F: WorthQueryDeclarationFamilyMarker<D>,
    {
        worth_query_checked_family_support::<D, C, F>(self)
    }

    pub fn declare<I>(
        &self,
        input: I,
    ) -> Result<
        WorthQueryCanonicalDeclarationArtifact<D, I>,
        WorthQueryDeclarationAdmissionError<D, I>,
    >
    where
        I: WorthQueryDeclarationInput<D>,
    {
        match self.declare_checked(input) {
            WorthQueryDeclaredFamilyChecked::Admitted(artifact) => Ok(artifact),
            WorthQueryDeclaredFamilyChecked::Deferred(denial) => {
                Err(WorthQueryDeclarationAdmissionError::Deferred(denial))
            }
            WorthQueryDeclaredFamilyChecked::AsyncDeferred(denial) => {
                Err(WorthQueryDeclarationAdmissionError::AsyncDeferred(denial))
            }
            WorthQueryDeclaredFamilyChecked::TemporalDeferred(denial) => Err(
                WorthQueryDeclarationAdmissionError::TemporalDeferred(denial),
            ),
            WorthQueryDeclaredFamilyChecked::Unsupported(denial) => {
                Err(WorthQueryDeclarationAdmissionError::Unsupported(denial))
            }
            WorthQueryDeclaredFamilyChecked::AsyncUnsupported(denial) => Err(
                WorthQueryDeclarationAdmissionError::AsyncUnsupported(denial),
            ),
            WorthQueryDeclaredFamilyChecked::TemporalUnsupported(denial) => Err(
                WorthQueryDeclarationAdmissionError::TemporalUnsupported(denial),
            ),
            WorthQueryDeclaredFamilyChecked::InvalidContext(denial) => {
                Err(WorthQueryDeclarationAdmissionError::InvalidContext(denial))
            }
            WorthQueryDeclaredFamilyChecked::Canonicalization(error) => {
                Err(WorthQueryDeclarationAdmissionError::Canonicalization(error))
            }
        }
    }

    pub fn declare_checked<I>(&self, input: I) -> WorthQueryDeclaredFamilyChecked<D, I>
    where
        I: WorthQueryDeclarationInput<D>,
    {
        worth_query_checked_family_declaration(
            self,
            input,
            WorthQueryDeclarationCanonicalizationVersion::default(),
        )
    }

    pub fn declare_with_version<I>(
        &self,
        input: I,
        version: WorthQueryDeclarationCanonicalizationVersion,
    ) -> Result<
        WorthQueryCanonicalDeclarationArtifact<D, I>,
        WorthQueryDeclarationAdmissionError<D, I>,
    >
    where
        I: WorthQueryDeclarationInput<D>,
    {
        match worth_query_checked_family_declaration(self, input, version) {
            WorthQueryDeclaredFamilyChecked::Admitted(artifact) => Ok(artifact),
            WorthQueryDeclaredFamilyChecked::Deferred(denial) => {
                Err(WorthQueryDeclarationAdmissionError::Deferred(denial))
            }
            WorthQueryDeclaredFamilyChecked::AsyncDeferred(denial) => {
                Err(WorthQueryDeclarationAdmissionError::AsyncDeferred(denial))
            }
            WorthQueryDeclaredFamilyChecked::TemporalDeferred(denial) => Err(
                WorthQueryDeclarationAdmissionError::TemporalDeferred(denial),
            ),
            WorthQueryDeclaredFamilyChecked::Unsupported(denial) => {
                Err(WorthQueryDeclarationAdmissionError::Unsupported(denial))
            }
            WorthQueryDeclaredFamilyChecked::AsyncUnsupported(denial) => Err(
                WorthQueryDeclarationAdmissionError::AsyncUnsupported(denial),
            ),
            WorthQueryDeclaredFamilyChecked::TemporalUnsupported(denial) => Err(
                WorthQueryDeclarationAdmissionError::TemporalUnsupported(denial),
            ),
            WorthQueryDeclaredFamilyChecked::InvalidContext(denial) => {
                Err(WorthQueryDeclarationAdmissionError::InvalidContext(denial))
            }
            WorthQueryDeclaredFamilyChecked::Canonicalization(error) => {
                Err(WorthQueryDeclarationAdmissionError::Canonicalization(error))
            }
        }
    }

    pub fn review_legality<I>(
        &self,
        declaration: WorthQueryCanonicalDeclarationArtifact<D, I>,
    ) -> Result<
        WorthQueryDeclarationLegalityEvidence<D, I>,
        WorthQueryDeclarationLegalityDenial<D, I>,
    >
    where
        I: WorthQueryDeclarationInput<D>,
    {
        match self.review_legality_checked(declaration) {
            WorthQueryDeclarationLegalityChecked::Legal(evidence) => Ok(evidence),
            WorthQueryDeclarationLegalityChecked::Illegal(denial) => Err(denial),
        }
    }

    pub fn review_legality_checked<I>(
        &self,
        declaration: WorthQueryCanonicalDeclarationArtifact<D, I>,
    ) -> WorthQueryDeclarationLegalityChecked<D, I>
    where
        I: WorthQueryDeclarationInput<D>,
    {
        let support_report = self.family_support::<I::Family>();
        let legality_contract = I::Family::legality_contract();
        let temporal_runtime_support_status = self
            .support_snapshot()
            .runtime_support_matrix()
            .row_for_family(WorthQueryRuntimeFacadeFamily::Temporal)
            .map(|row| row.status());
        let async_runtime_support_status = self
            .support_snapshot()
            .runtime_support_matrix()
            .row_for_family(WorthQueryRuntimeFacadeFamily::AsyncResource)
            .map(|row| row.status());
        let input = WorthQueryDeclarationLegalityInput::new(
            declaration,
            support_report,
            legality_contract,
            self.retained_world_basis(),
            temporal_runtime_support_status,
            async_runtime_support_status,
        );
        review_declaration_legality(self.handle_identity_digest(), input)
    }

    pub fn declare_and_review<I>(
        &self,
        input: I,
    ) -> Result<
        WorthQueryDeclarationLegalityEvidence<D, I>,
        WorthQueryDeclarationAdmissionOrLegalityError<D, I>,
    >
    where
        I: WorthQueryDeclarationInput<D>,
    {
        let declaration = self
            .declare(input)
            .map_err(WorthQueryDeclarationAdmissionOrLegalityError::Admission)?;
        self.review_legality(declaration)
            .map_err(WorthQueryDeclarationAdmissionOrLegalityError::Legality)
    }
}
