//! Read-only unclean-tail diagnostics grant no runtime or recovery authority.
use super::WorthQueryApplicationHomeOpenRefused;
use worth_foundational::facade::{DurableOwnerFamily, UncleanRecoveryUnsupported};
use worth_query_declaration::facade::application_schema::ApplicationSchemaComposition;
use worth_query_execution::facade::application_home_integration::{
    QueryDispatchJournalDenial, QueryHomeCustodyReadmissionDenial,
};
use worth_store_application::ApplicationHomeOpenRefusal;

impl<Schema: ApplicationSchemaComposition, Program>
    WorthQueryApplicationHomeOpenRefused<Schema, Program>
{
    /// Names the actual owner effect that prevents happy-track service.
    /// Cleanup track C5 removes this diagnostic with the D18 refusal.
    pub fn unclean_recovery_unsupported(&self) -> Option<UncleanRecoveryUnsupported> {
        // A recorded dispatch attempt with no recorded observation: the
        // process stopped inside that effect's external contact.
        if let Self::CustodyReadmission {
            cause:
                super::HomeCustodyReadmissionDenial::Query(QueryHomeCustodyReadmissionDenial::Dispatch(
                    QueryDispatchJournalDenial::UnobservedAttempt { effect, .. },
                )),
            ..
        } = self
        {
            return Some(UncleanRecoveryUnsupported {
                owner: DurableOwnerFamily::Dispatch,
                effect: *effect,
            });
        }
        let (Self::WorldReadmission { home, .. }
        | Self::ExistingWorldReadmission { home, .. }
        | Self::WorldSourcePreparation { home, .. }
        | Self::ExistingWorldSourcePreparation { home, .. }) = self
        else {
            return None;
        };
        let ApplicationHomeOpenRefusal::UncleanRecoveryUnsupported(cause) = home.reason() else {
            return None;
        };
        Some(*cause)
    }
}
