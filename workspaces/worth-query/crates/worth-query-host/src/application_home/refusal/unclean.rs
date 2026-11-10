//! Read-only unclean-tail diagnostics grant no runtime or recovery authority.
use super::WorthQueryApplicationHomeOpenRefused;
use worth_foundational::facade::UncleanRecoveryUnsupported;
use worth_query_declaration::facade::application_schema::ApplicationSchemaComposition;
use worth_store_application::ApplicationHomeOpenRefusal;

impl<Schema: ApplicationSchemaComposition, Program>
    WorthQueryApplicationHomeOpenRefused<Schema, Program>
{
    /// Names the actual owner effect that prevents happy-track service.
    /// Cleanup track C5 removes this diagnostic with the D18 refusal.
    pub fn unclean_recovery_unsupported(&self) -> Option<UncleanRecoveryUnsupported> {
        let (Self::WorldReadmission { home, .. } | Self::ExistingWorldReadmission { home, .. }) =
            self
        else {
            return None;
        };
        let ApplicationHomeOpenRefusal::UncleanRecoveryUnsupported(cause) = home.reason() else {
            return None;
        };
        Some(*cause)
    }
}
