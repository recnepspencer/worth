//! Refusal retains the exact completed prefix and all untouched configurations.
use super::*;
pub struct RefusedConditionalConfiguration {
    pub cause: WorthQueryConditionalRuntimeInstallationDenial,
    pub installed: Option<InstalledConditionalBinding>,
}
pub struct RefusedConditionalBatch<Schema: ApplicationSchema + 'static> {
    _pending: PendingConditionalRegistry<Schema>,
    _completed: BTreeMap<String, InstalledConditionalBinding>,
    _refused_installation: Option<InstalledConditionalBinding>,
    cause: WorthQueryConditionalRuntimeInstallationDenial,
}
impl<Schema: ApplicationSchema + 'static> RefusedConditionalBatch<Schema> {
    pub fn reason(&self) -> &WorthQueryConditionalRuntimeInstallationDenial {
        &self.cause
    }
    pub fn into_cause(self) -> WorthQueryConditionalRuntimeInstallationDenial {
        self.cause
    }
}
impl<Schema: ApplicationSchema + 'static> PendingConditionalRegistry<Schema> {
    pub fn install_all(
        mut self,
        producers: &WorthQueryInstalledApplicationProducerRegistry<Schema>,
        installation: &mut WorthQueryConditionalApplicationRuntimeInstallation<Schema>,
    ) -> Result<
        WorthQueryInstalledApplicationConditionalRegistry<Schema>,
        RefusedConditionalBatch<Schema>,
    > {
        let declared = self.declared;
        let configured = self.configured;
        WorthQueryInstalledApplicationConditionalRegistry::install_declared_entries(
            &declared,
            configured,
            |declaration, configuration| {
                configuration.install(producers, declaration, installation)
            },
        )
        .map_err(|(remaining, completed, refused)| RefusedConditionalBatch {
            _pending: PendingConditionalRegistry {
                declared,
                configured: remaining,
            },
            _completed: completed,
            _refused_installation: refused.installed,
            cause: refused.cause,
        })
    }
}
