//! Refusal retains the exact completed prefix and all untouched configurations.
use super::*;
pub struct RefusedConditionalConfiguration {
    pub cause: WorthQueryConditionalRuntimeInstallationDenial,
    pub installed: Option<InstalledConditionalBinding>,
}
pub struct RefusedConditionalBatch<
    Schema: ApplicationSchema + 'static,
> {
    _pending: PendingConditionalRegistry<Schema>,
    _completed: BTreeMap<String, InstalledConditionalBinding>,
    _refused_installation: Option<InstalledConditionalBinding>,
    cause: WorthQueryConditionalRuntimeInstallationDenial,
}
impl<Schema: ApplicationSchema + 'static> RefusedConditionalBatch<Schema> {
    pub fn reason(
        &self,
    ) -> &WorthQueryConditionalRuntimeInstallationDenial {
        &self.cause
    }
    pub fn into_cause(
        self,
    ) -> WorthQueryConditionalRuntimeInstallationDenial {
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
        let mut completed = BTreeMap::new();
        while let Some((identity, configuration)) = self.configured.pop_first() {
            let declaration = self
                .declared
                .get(&identity)
                .expect("configured identity came from the declaration");
            match configuration.install(producers, declaration, installation) {
                Ok(installed) => {
                    completed.insert(identity, installed);
                }
                Err(refused) => {
                    return Err(RefusedConditionalBatch {
                        _pending: self,
                        _completed: completed,
                        _refused_installation: refused.installed,
                        cause: refused.cause,
                    })
                }
            }
        }
        Ok(WorthQueryInstalledApplicationConditionalRegistry::from_entries(
            worth_query_execution_query::facade::ports::worth_query_query_crate_seal(),
            completed,
        ))
    }
}
