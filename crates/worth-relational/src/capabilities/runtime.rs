use crate::config::data::{RelationalRuntimeConfig, RelationalRuntimeProfile};
use crate::identity::data::VersionId;
use crate::runtime::{RelationalRuntime, RuntimeInstrumentation};

pub(crate) trait VersionSource {
    fn current_version_id(&self) -> VersionId;
}

impl VersionSource for RelationalRuntime {
    fn current_version_id(&self) -> VersionId {
        RelationalRuntime::current_version_id(self)
    }
}

pub(crate) trait RuntimeConfigSource {
    /// The configuration in force for this operation.
    fn runtime_config(&self) -> &RelationalRuntimeConfig;
}

impl RuntimeConfigSource for RelationalRuntime {
    fn runtime_config(&self) -> &RelationalRuntimeConfig {
        &self.config
    }
}

pub(crate) trait InstrumentationSource {
    fn runtime_instrumentation(&self) -> &RuntimeInstrumentation;
}

impl InstrumentationSource for RelationalRuntime {
    fn runtime_instrumentation(&self) -> &RuntimeInstrumentation {
        &self.services.instrumentation
    }
}

pub(crate) trait RuntimeIdentitySource {
    fn runtime_name(&self) -> &str;
    fn runtime_profile(&self) -> RelationalRuntimeProfile;
}

impl RuntimeIdentitySource for RelationalRuntime {
    fn runtime_name(&self) -> &str {
        &self.config.execution.runtime_name
    }

    fn runtime_profile(&self) -> RelationalRuntimeProfile {
        self.config.profile
    }
}

pub(crate) trait SchemaVersionSource {
    fn primary_schema_version_id(&self) -> crate::schema::data::SchemaVersionId;
}

impl SchemaVersionSource for RelationalRuntime {
    fn primary_schema_version_id(&self) -> crate::schema::data::SchemaVersionId {
        self.config.schema.registry.primary_schema_version_id()
    }
}

impl SchemaVersionSource for RelationalRuntimeConfig {
    fn primary_schema_version_id(&self) -> crate::schema::data::SchemaVersionId {
        self.schema.registry.primary_schema_version_id()
    }
}
