use std::marker::PhantomData;

use worth_query_declaration::facade::{
    application_program::{ApplicationExternalInputProvider, ApplicationExternalInputResolution},
    application_schema::{
        ApplicationOperationMarkerIdentity, ApplicationSchema, ApplicationSchemaBindingIdentity,
    },
};

pub struct WorthQueryInstalledExternalInputProvider<Schema, Operation, Provider> {
    schema_binding: ApplicationSchemaBindingIdentity,
    marker: PhantomData<fn() -> (Schema, Operation, Provider)>,
}

impl<Schema, Operation, Provider>
    WorthQueryInstalledExternalInputProvider<Schema, Operation, Provider>
where
    Schema: ApplicationSchema,
    Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
    Provider: ApplicationExternalInputProvider<Schema, Operation>,
{
    pub(super) const fn new(schema_binding: ApplicationSchemaBindingIdentity) -> Self {
        Self {
            schema_binding,
            marker: PhantomData,
        }
    }

    pub const fn schema_binding(&self) -> &ApplicationSchemaBindingIdentity {
        &self.schema_binding
    }

    pub fn resolve(
        &self,
        provider: &Provider,
        selection: Provider::Selection,
    ) -> Result<WorthQueryCapturedExternalInput<Schema, Operation, Provider>, Provider::Denial>
    {
        let resolution = provider.resolve(&selection)?;
        Ok(WorthQueryCapturedExternalInput {
            selection,
            resolution,
            marker: PhantomData,
        })
    }
}

pub struct WorthQueryCapturedExternalInput<Schema, Operation, Provider>
where
    Schema: ApplicationSchema,
    Operation: ApplicationOperationMarkerIdentity<Schema>,
    Provider: ApplicationExternalInputProvider<Schema, Operation>,
{
    selection: Provider::Selection,
    resolution: ApplicationExternalInputResolution<
        Provider::Values,
        Provider::Revision,
        Provider::Provenance,
    >,
    marker: PhantomData<fn() -> (Schema, Operation)>,
}

impl<Schema, Operation, Provider> WorthQueryCapturedExternalInput<Schema, Operation, Provider>
where
    Schema: ApplicationSchema,
    Operation: ApplicationOperationMarkerIdentity<Schema>,
    Provider: ApplicationExternalInputProvider<Schema, Operation>,
{
    pub const fn resolution(
        &self,
    ) -> &ApplicationExternalInputResolution<
        Provider::Values,
        Provider::Revision,
        Provider::Provenance,
    > {
        &self.resolution
    }

    pub const fn selection(&self) -> &Provider::Selection {
        &self.selection
    }

    pub fn admit(
        self,
        provider: &Provider,
    ) -> Result<WorthQueryAdmittedExternalInput<Schema, Operation, Provider>, Provider::Denial>
    {
        provider.validate_revision(&self.selection, self.resolution.revision())?;
        Ok(WorthQueryAdmittedExternalInput {
            selection: self.selection,
            resolution: self.resolution,
            marker: PhantomData,
        })
    }
}

pub struct WorthQueryAdmittedExternalInput<Schema, Operation, Provider>
where
    Schema: ApplicationSchema,
    Operation: ApplicationOperationMarkerIdentity<Schema>,
    Provider: ApplicationExternalInputProvider<Schema, Operation>,
{
    selection: Provider::Selection,
    resolution: ApplicationExternalInputResolution<
        Provider::Values,
        Provider::Revision,
        Provider::Provenance,
    >,
    marker: PhantomData<fn() -> (Schema, Operation)>,
}

impl<Schema, Operation, Provider> WorthQueryAdmittedExternalInput<Schema, Operation, Provider>
where
    Schema: ApplicationSchema,
    Operation: ApplicationOperationMarkerIdentity<Schema>,
    Provider: ApplicationExternalInputProvider<Schema, Operation>,
{
    pub const fn selection(&self) -> &Provider::Selection {
        &self.selection
    }

    pub fn into_resolution(
        self,
    ) -> ApplicationExternalInputResolution<
        Provider::Values,
        Provider::Revision,
        Provider::Provenance,
    > {
        self.resolution
    }
}
