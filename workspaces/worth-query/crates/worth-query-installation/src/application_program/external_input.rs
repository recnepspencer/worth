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
        self.admit_preserving(provider)
            .map_err(|(_, denial)| denial)
    }

    /// Validates the original capture without discarding it on refusal.
    /// Retry with this returned capture; resolving again is a different observation.
    /// This admits data correspondence, not permission to execute an operation.
    /// # Errors
    /// Returns the unchanged selection, values, revision and provenance alongside
    /// the provider's actual validation denial.
    pub fn admit_preserving(
        self,
        provider: &Provider,
    ) -> Result<
        WorthQueryAdmittedExternalInput<Schema, Operation, Provider>,
        (Self, Provider::Denial),
    > {
        if let Err(denial) = provider.validate_revision(&self.selection, self.resolution.revision())
        {
            return Err((self, denial));
        }
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

#[cfg(test)]
mod tests {
    use super::super::program_support_fixture::{SupportSchema, UnknownOperation};
    use super::*;
    struct Provider(u64);
    impl ApplicationExternalInputProvider<SupportSchema, UnknownOperation> for Provider {
        const IDENTITY: &'static str = "external-capture-boundary";
        type Selection = String;
        type Values = Vec<u8>;
        type Revision = u64;
        type Provenance = String;
        type Denial = (u64, u64);
        fn resolve(
            &self,
            selection: &String,
        ) -> Result<ApplicationExternalInputResolution<Vec<u8>, u64, String>, Self::Denial>
        {
            Ok(ApplicationExternalInputResolution::new(
                selection.as_bytes().to_vec(),
                self.0,
                "original external source".to_owned(),
            ))
        }
        fn validate_revision(&self, _: &String, revision: &u64) -> Result<(), Self::Denial> {
            if *revision == self.0 {
                Ok(())
            } else {
                Err((*revision, self.0))
            }
        }
    }
    #[test]
    fn external_input_capture_refusal_preserves_selection_values_revision_and_provenance() {
        // Pure caller-owned DATA, not a Home record or operation capability.
        let selection = "original selection".to_owned();
        let resolution = Provider(7).resolve(&selection).unwrap();
        let capture = WorthQueryCapturedExternalInput::<SupportSchema, UnknownOperation, Provider> {
            selection: selection.clone(),
            resolution,
            marker: PhantomData,
        };
        let (capture, denial) = capture
            .admit_preserving(&Provider(8))
            .err()
            .expect("changed original revision refuses");
        assert_eq!(denial, (7, 8));
        assert_eq!(capture.selection(), &selection);
        assert_eq!(capture.resolution().values(), selection.as_bytes());
        assert_eq!(capture.resolution().revision(), &7);
        assert_eq!(
            capture.resolution().provenance(),
            "original external source"
        );
        let admitted = capture
            .admit_preserving(&Provider(7))
            .unwrap_or_else(|_| panic!("fresh provider must validate the retained original"));
        assert_eq!(admitted.selection(), &selection);
        assert_eq!(admitted.into_resolution().values(), selection.as_bytes());
    }
}
