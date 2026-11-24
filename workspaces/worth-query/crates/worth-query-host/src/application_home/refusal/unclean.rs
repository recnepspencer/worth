//! Read-only unclean-tail diagnostics grant no runtime or recovery authority.
use super::{WorthQueryApplicationHomeOpenRefused, WorthQueryApplicationHomeOpenRefusedPhase};
use worth_foundational::facade::UncleanRecoveryUnsupported;
use worth_query_declaration::facade::application_schema::ApplicationSchemaComposition;
use worth_runtime_world::facade::WorldReadmissionDenial;
use worth_store_application::{ApplicationHomeOpenRefusal, ApplicationHomePhysicalRefusal};

impl<Schema: ApplicationSchemaComposition, Program>
    WorthQueryApplicationHomeOpenRefused<Schema, Program>
{
    /// Store's physical refusal when the Home location itself could not be
    /// opened: contended by a live owner or tombstoned by a pending discard.
    pub fn physical_refusal(&self) -> Option<&ApplicationHomePhysicalRefusal> {
        match self.phase() {
            WorthQueryApplicationHomeOpenRefusedPhase::Physical {
                cause: ApplicationHomeOpenRefusal::Physical(cause),
                ..
            } => Some(cause),
            _ => None,
        }
    }

    /// Names the actual owner effect that prevents happy-track service.
    /// Cleanup track C5 removes this diagnostic with the D18 refusal.
    pub fn unclean_recovery_unsupported(&self) -> Option<UncleanRecoveryUnsupported> {
        let (WorthQueryApplicationHomeOpenRefusedPhase::WorldReadmission { home, .. }
        | WorthQueryApplicationHomeOpenRefusedPhase::ExistingWorldReadmission { home, .. }
        | WorthQueryApplicationHomeOpenRefusedPhase::WorldSourcePreparation { home, .. }
        | WorthQueryApplicationHomeOpenRefusedPhase::ExistingWorldSourcePreparation {
            home,
            ..
        }) = self.phase()
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
