//! A schema's immutable descriptive carrier shares its original admitted backing.
use super::RelationalBranchRootSchemaAuthority;
use crate::durability::data::recovery_stop::RelationalRecoveryStop;
use crate::history::commit::{CapturedOriginalCanonicalInputs, OriginalCanonicalInputs};
use crate::history::data::canonical_commit_envelope::schema_carrier::{
    RecordedBranchSchemaImage, SchemaImageSource,
};
use worth_execution::ExecutionRequest;

impl OriginalCanonicalInputs<RelationalBranchRootSchemaAuthority> {
    pub(crate) fn capture(
        self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<CapturedOriginalCanonicalInputs, RelationalRecoveryStop> {
        Ok(CapturedOriginalCanonicalInputs {
            prestate_schema: self.prestate_schema.recorded_image(request)?,
            result_schema: self.result_schema.recorded_image(request)?,
            mutation: self.mutation,
        })
    }
}

impl TryFrom<OriginalCanonicalInputs<RelationalBranchRootSchemaAuthority>>
    for CapturedOriginalCanonicalInputs
{
    type Error = crate::execution::denial_cause::RelationalExecutionDenialCause;

    fn try_from(
        inputs: OriginalCanonicalInputs<RelationalBranchRootSchemaAuthority>,
    ) -> Result<Self, Self::Error> {
        let carrier = |schema: &RelationalBranchRootSchemaAuthority| {
            schema
                .retained_recorded_image()
                .cloned()
                .ok_or(Self::Error::ContentInconsistency)
        };
        Ok(Self {
            prestate_schema: carrier(&inputs.prestate_schema)?,
            result_schema: carrier(&inputs.result_schema)?,
            mutation: inputs.mutation,
        })
    }
}

impl RelationalBranchRootSchemaAuthority {
    pub(crate) fn retained_recorded_image(&self) -> Option<&RecordedBranchSchemaImage> {
        self.recorded_image.get()
    }

    pub(crate) fn schema_image_source(&self) -> SchemaImageSource<'_> {
        SchemaImageSource {
            registry: self.registry(),
            retained_aspect_contracts: self.retained_aspect_contracts(),
            schema_root: self.authority_digest(),
            descriptor_semantics_version: self.descriptor_semantics_version(),
        }
    }

    /// Reconstructive capture may share an existing producer image; it cannot
    /// install a new live cache owned by the capture's request.
    pub(crate) fn descriptive_image_for_request(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<RecordedBranchSchemaImage, RelationalRecoveryStop> {
        crate::execution::request_custody::check_request(request)
            .map_err(RelationalRecoveryStop::Request)?;
        match self.retained_recorded_image() {
            Some(image) => Ok(image.clone()),
            None => {
                RecordedBranchSchemaImage::capture_for_request(self.schema_image_source(), request)
            }
        }
    }
    pub(crate) fn retain_readmitted_image(
        &self,
        image: RecordedBranchSchemaImage,
    ) -> Result<(), &'static str> {
        if self.authority_digest() != image.schema_root()
            || self.descriptor_semantics_version() != image.descriptor_semantics_version()
        {
            return Err("readmitted schema carrier differs from its authority");
        }
        self.recorded_image
            .set(image)
            .map_err(|_| "readmitted schema image already installed")
    }
    pub(crate) fn recorded_image(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<RecordedBranchSchemaImage, RelationalRecoveryStop> {
        crate::execution::request_custody::check_request(request)
            .map_err(RelationalRecoveryStop::Request)?;
        if let Some(image) = self.retained_recorded_image() {
            return Ok(image.clone());
        }
        let image =
            RecordedBranchSchemaImage::capture_for_request(self.schema_image_source(), request)?;
        // Concurrent preparations may independently admit a carrier. Losing
        // copies release their own backing; no caller's request is replaced.
        let _ = self.recorded_image.set(image.clone());
        Ok(image)
    }
}
