use crate::application::{
    worth_query_checked_declaration_progression, worth_query_declaration_progression_recipe,
    WorthQueryAdmittedDeclarationProgression, WorthQueryDeclarationAdmissionOrLegalityError,
    WorthQueryDeclarationInput, WorthQueryDeclarationLegalityEvidence,
    WorthQueryDeclarationProgressionChecked, WorthQueryDeclarationProgressionRecipe,
    WorthQueryDeclarationProgressionTerminalError, WorthQueryDomainEntryMarker,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::super::WorthQueryInstalledDomainDeclarationContext;
use crate::application::WorthQueryDomainOperatingContext;

impl<D: WorthQueryDomainEntryMarker, C: WorthQueryDomainOperatingContext<D>>
    WorthQueryInstalledDomainDeclarationContext<D, C>
{
    pub(crate) fn declaration_progression_recipe<I>(
        &self,
        legal: WorthQueryDeclarationLegalityEvidence<D, I>,
    ) -> Result<WorthQueryDeclarationProgressionRecipe<D, I>, IdentityDerivationDenial>
    where
        I: WorthQueryDeclarationInput<D>,
    {
        Ok(worth_query_declaration_progression_recipe(
            legal,
            self.retained_world_basis()?,
        ))
    }

    pub fn progress_declaration<I>(
        &self,
        legal: WorthQueryDeclarationLegalityEvidence<D, I>,
    ) -> Result<
        WorthQueryAdmittedDeclarationProgression<D, I>,
        WorthQueryDeclarationProgressionTerminalError<D, I>,
    >
    where
        I: WorthQueryDeclarationInput<D>,
    {
        match self.progress_declaration_checked(legal)? {
            WorthQueryDeclarationProgressionChecked::Admitted(admitted) => Ok(admitted),
            WorthQueryDeclarationProgressionChecked::Deferred(progress) => Err(
                WorthQueryDeclarationProgressionTerminalError::Deferred(progress),
            ),
            WorthQueryDeclarationProgressionChecked::Denied(progress) => Err(
                WorthQueryDeclarationProgressionTerminalError::Denied(progress),
            ),
            WorthQueryDeclarationProgressionChecked::Stale(progress) => Err(
                WorthQueryDeclarationProgressionTerminalError::Stale(progress),
            ),
            WorthQueryDeclarationProgressionChecked::RebindRequired(progress) => {
                Err(WorthQueryDeclarationProgressionTerminalError::RebindRequired(progress))
            }
            WorthQueryDeclarationProgressionChecked::Failed(progress) => Err(
                WorthQueryDeclarationProgressionTerminalError::Failed(progress),
            ),
        }
    }

    #[cfg(test)]
    pub(crate) fn progress_declaration_recipe<I>(
        &self,
        recipe: WorthQueryDeclarationProgressionRecipe<D, I>,
    ) -> Result<
        WorthQueryAdmittedDeclarationProgression<D, I>,
        WorthQueryDeclarationProgressionTerminalError<D, I>,
    >
    where
        I: WorthQueryDeclarationInput<D>,
    {
        match worth_query_checked_declaration_progression(recipe) {
            WorthQueryDeclarationProgressionChecked::Admitted(admitted) => Ok(admitted),
            WorthQueryDeclarationProgressionChecked::Deferred(progress) => Err(
                WorthQueryDeclarationProgressionTerminalError::Deferred(progress),
            ),
            WorthQueryDeclarationProgressionChecked::Denied(progress) => Err(
                WorthQueryDeclarationProgressionTerminalError::Denied(progress),
            ),
            WorthQueryDeclarationProgressionChecked::Stale(progress) => Err(
                WorthQueryDeclarationProgressionTerminalError::Stale(progress),
            ),
            WorthQueryDeclarationProgressionChecked::RebindRequired(progress) => {
                Err(WorthQueryDeclarationProgressionTerminalError::RebindRequired(progress))
            }
            WorthQueryDeclarationProgressionChecked::Failed(progress) => Err(
                WorthQueryDeclarationProgressionTerminalError::Failed(progress),
            ),
        }
    }

    pub fn progress_declaration_checked<I>(
        &self,
        legal: WorthQueryDeclarationLegalityEvidence<D, I>,
    ) -> Result<WorthQueryDeclarationProgressionChecked<D, I>, IdentityDerivationDenial>
    where
        I: WorthQueryDeclarationInput<D>,
    {
        Ok(worth_query_checked_declaration_progression(
            self.declaration_progression_recipe(legal)?,
        ))
    }

    #[cfg(test)]
    pub(crate) fn progress_declaration_recipe_checked<I>(
        &self,
        recipe: WorthQueryDeclarationProgressionRecipe<D, I>,
    ) -> WorthQueryDeclarationProgressionChecked<D, I>
    where
        I: WorthQueryDeclarationInput<D>,
    {
        worth_query_checked_declaration_progression(recipe)
    }

    pub(crate) fn declare_review_and_progress<I>(
        &self,
        input: I,
    ) -> Result<
        WorthQueryAdmittedDeclarationProgression<D, I>,
        WorthQueryDeclarationEntryProgressionError<D, I>,
    >
    where
        I: WorthQueryDeclarationInput<D>,
    {
        let legal = self
            .declare_and_review(input)
            .map_err(WorthQueryDeclarationEntryProgressionError::Entry)?;
        self.progress_declaration(legal)
            .map_err(WorthQueryDeclarationEntryProgressionError::Progression)
    }
}

pub enum WorthQueryDeclarationEntryProgressionError<
    D: WorthQueryDomainEntryMarker,
    I: WorthQueryDeclarationInput<D>,
> {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    Entry(WorthQueryDeclarationAdmissionOrLegalityError<D, I>),
    Progression(WorthQueryDeclarationProgressionTerminalError<D, I>),
}

impl<D: WorthQueryDomainEntryMarker, I: WorthQueryDeclarationInput<D>>
    From<IdentityDerivationDenial> for WorthQueryDeclarationEntryProgressionError<D, I>
{
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self::EvidenceIdentityDerivationDenied(cause)
    }
}
