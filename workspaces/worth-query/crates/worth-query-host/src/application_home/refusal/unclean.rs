//! Read-only unclean-tail diagnostics grant no runtime or recovery authority.
use super::WorthQueryApplicationHomeOpenRefused;
use worth_foundational::facade::{DurableOwnerFamily, UncleanRecoveryUnsupported};
use worth_query_declaration::facade::application_schema::ApplicationSchemaComposition;
use worth_query_execution::facade::application_home_integration::{
    QueryDispatchJournalDenial, QueryHomeCustodyReadmissionDenial,
};
use worth_runtime_world::facade::WorldReadmissionDenial;
use worth_store_application::{ApplicationHomeOpenRefusal, ApplicationHomePhysicalRefusal};

impl<Schema: ApplicationSchemaComposition, Program>
    WorthQueryApplicationHomeOpenRefused<Schema, Program>
{
    /// Store's physical refusal when the Home location itself could not be
    /// opened: contended by a live owner or tombstoned by a pending discard.
    pub fn physical_refusal(&self) -> Option<&ApplicationHomePhysicalRefusal> {
        match self {
            Self::Physical {
                cause: ApplicationHomeOpenRefusal::Physical(cause),
                ..
            } => Some(cause),
            _ => None,
        }
    }

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
        let ApplicationHomeOpenRefusal::World(WorldReadmissionDenial::UncleanRecoveryUnsupported(
            cause,
        )) = home.reason()
        else {
            return None;
        };
        Some(*cause)
    }
}
