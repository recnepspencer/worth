//! Installation checks correlate the exact fresh programme with its admitted owner.
use super::BridgeReadmittedAuthoredAssembly;
use crate::facade::BridgeAuthoritativeSourceProfile;
impl BridgeReadmittedAuthoredAssembly {
    pub fn admit_programme_binding(
        &self,
        candidate: &worth_proof::BridgeProgramInstallationBinding,
    ) -> Result<(), worth_proof::BridgeProgramInstallationDenial> {
        self.program.installation.admit(candidate)
    }
    pub fn readmits_authoritative_source_profile(
        &self,
        candidate: &BridgeAuthoritativeSourceProfile,
    ) -> bool {
        self.program
            .bridge
            .authoritative_source_profile()
            .is_some_and(|installed| installed == candidate)
    }
}
