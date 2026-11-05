//! Sealed test doors over this module's private members.
use super::*;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn readmit_elevation_request_installation_for_test<Operation, Input, Scope>(
        &self,
        program: WorthQueryElevationRequestProgram<Schema, Operation, Input, Scope>,
    ) -> Result<
        ReadmittedCommitProgram<
            '_,
            Schema,
            WorthQueryElevationRequestProgram<Schema, Operation, Input, Scope>,
        >,
        WorthQueryElevationRequestOutcome,
    > {
        Self::readmit_elevation_request_installation::<Operation, Input, Scope>(self, program)
    }
}
