//! A schema's immutable descriptive carrier shares its original admitted backing.
use super::RelationalBranchRootSchemaAuthority;
use crate::durability::data::recovery_stop::RelationalRecoveryStop;
use crate::history::data::canonical_commit_envelope::schema_carrier::{
    RecordedBranchSchemaImage, SchemaImageSource,
};
use worth_execution::ExecutionRequest;

impl RelationalBranchRootSchemaAuthority {
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
        match self.recorded_image.get() {
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
        if let Some(image) = self.recorded_image.get() {
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
