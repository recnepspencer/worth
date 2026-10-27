use worth_foundational::facade::AspectKey;
use worth_query_declaration::facade::application_schema::{
    ApplicationFieldBindingOwnerKind, ApplicationSchemaBindingIdentity,
};

/// Exact installed application locus to which one native aspect contract belongs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInstalledApplicationAspectLocus {
    schema: ApplicationSchemaBindingIdentity,
    owner_kind: ApplicationFieldBindingOwnerKind,
    entity: String,
    aspect: AspectKey,
}

impl WorthQueryInstalledApplicationAspectLocus {
    pub(crate) fn new(
        schema: ApplicationSchemaBindingIdentity,
        owner_kind: ApplicationFieldBindingOwnerKind,
        entity: String,
        aspect: AspectKey,
    ) -> Self {
        Self {
            schema,
            owner_kind,
            entity,
            aspect,
        }
    }

    pub fn schema(&self) -> &ApplicationSchemaBindingIdentity {
        &self.schema
    }

    pub fn entity(&self) -> &str {
        &self.entity
    }

    pub const fn owner_kind(&self) -> ApplicationFieldBindingOwnerKind {
        self.owner_kind
    }
    pub fn record_name(&self) -> &str {
        &self.entity
    }

    pub fn aspect(&self) -> &AspectKey {
        &self.aspect
    }
}
