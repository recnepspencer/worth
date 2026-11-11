//! A schema's immutable descriptive carrier shares its original admitted backing.
use super::RelationalBranchRootSchemaAuthority;
use crate::durability::data::{DurableBranchRootSchemaImage, RelationalRecoveryStop};
use worth_execution::ExecutionRequest;

impl RelationalBranchRootSchemaAuthority {
    /// Reconstructive capture may share an existing producer image; it cannot
    /// install a new live cache owned by the capture's request.
    pub(crate) fn descriptive_image_for_request(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<DurableBranchRootSchemaImage, RelationalRecoveryStop> {
        crate::execution::check_request(request).map_err(RelationalRecoveryStop::Request)?;
        match self.recorded_image.get() {
            Some(image) => Ok(image.clone()),
            None => DurableBranchRootSchemaImage::capture_for_request(self, request),
        }
    }
    pub(crate) fn retain_readmitted_image(
        &self,
        image: DurableBranchRootSchemaImage,
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
    ) -> Result<DurableBranchRootSchemaImage, RelationalRecoveryStop> {
        crate::execution::check_request(request).map_err(RelationalRecoveryStop::Request)?;
        if let Some(image) = self.recorded_image.get() {
            return Ok(image.clone());
        }
        let image = DurableBranchRootSchemaImage::capture_for_request(self, request)?;
        // Concurrent preparations may independently admit a carrier. Losing
        // copies release their own backing; no caller's request is replaced.
        let _ = self.recorded_image.set(image.clone());
        Ok(image)
    }
}
