//! Certification-only doors: open a home on a Store the test opened itself
//! (with a media fault schedule), and reach the home's Store to park a
//! publication at a mutation seam. Both run the production admission path;
//! neither exists in a production build.

use worth_store::physical_runtime::ServingPhysicalRuntime;

use super::{RelationalStoreHome, RelationalStoreHomeOpenRefusal, RelationalStoreHomeRequest};

impl RelationalStoreHome {
    /// Admit the home on `serving` exactly as `open` would after its own
    /// physical open. The request's location is ignored.
    #[doc(hidden)]
    pub fn certification_open_on(
        serving: ServingPhysicalRuntime,
        request: RelationalStoreHomeRequest,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, RelationalStoreHomeOpenRefusal> {
        let (_, builder) = request.into_parts();
        Self::admit(serving, builder, execution)
    }

    /// The home's Store, for certification seams.
    #[doc(hidden)]
    pub fn certification_serving(&self) -> &ServingPhysicalRuntime {
        &self.serving
    }
}
