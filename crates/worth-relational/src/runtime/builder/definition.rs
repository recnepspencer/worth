use crate::runtime::{
    RelationalRuntime, RelationalRuntimeConfig, RelationalRuntimeConstructionDenial,
    RuntimeExtensions,
};

/// Resolved native construction inputs, before a runtime or medium exists.
///
/// Configuration is descriptive, not admission or publication authority. The
/// existing construction core still validates it when this value is built.
///
/// ```
/// use worth_relational::facade::runtime::RelationalRuntimeApi;
/// let definition = RelationalRuntimeApi::builder().runtime_name("declared-home").resolve();
/// assert_eq!(definition.configuration().execution.runtime_name, "declared-home");
/// let runtime = definition.build().unwrap();
/// assert_eq!(runtime.config().execution.runtime_name, "declared-home");
/// ```
///
/// A resolved definition is consumed by its one construction:
///
/// ```compile_fail,E0382
/// use worth_relational::facade::runtime::RelationalRuntimeApi;
/// let definition = RelationalRuntimeApi::builder().resolve();
/// let first = definition.build().unwrap();
/// let second = definition.build().unwrap();
/// ```
#[derive(Debug)]
pub struct RelationalRuntimeDefinition {
    configuration: RelationalRuntimeConfig,
    extensions: RuntimeExtensions,
}

impl RelationalRuntimeDefinition {
    pub(super) fn new(
        configuration: RelationalRuntimeConfig,
        extensions: RuntimeExtensions,
    ) -> Self {
        Self {
            configuration,
            extensions,
        }
    }

    pub fn configuration(&self) -> &RelationalRuntimeConfig {
        &self.configuration
    }

    pub(in crate::runtime) fn into_home_configuration(
        self,
    ) -> Result<RelationalRuntimeConfig, crate::runtime::RelationalHomeProgramContractDenial> {
        if !self.extensions.is_home_configuration_only() {
            return Err(
                crate::runtime::RelationalHomeProgramContractDenial::UnsupportedConfigurationChange,
            );
        }
        Ok(self.configuration)
    }

    pub fn build(self) -> Result<RelationalRuntime, RelationalRuntimeConstructionDenial> {
        self.build_with(None)
    }

    pub fn build_durable(
        self,
        medium: Box<dyn crate::durability::medium::RelationalDurableMedium>,
    ) -> Result<RelationalRuntime, RelationalRuntimeConstructionDenial> {
        self.build_with(Some(medium))
    }

    pub(super) fn build_with(
        self,
        medium: Option<Box<dyn crate::durability::medium::RelationalDurableMedium>>,
    ) -> Result<RelationalRuntime, RelationalRuntimeConstructionDenial> {
        RelationalRuntime::build_from_extensions(
            self.configuration,
            self.extensions.with_durable_medium(medium),
        )
    }
}
