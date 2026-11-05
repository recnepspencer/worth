//! Sealed test doors over this module's private members.
use super::*;

pub trait InstalledReadmissionDoorsOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>
{
    #[doc(hidden)]
    fn readmit_elevation_request_installation_for_test<Operation, Input, Scope>(
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

impl<Schema: ApplicationSchema, HostRuntime: crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema> + ?Sized> InstalledReadmissionDoorsOps<Schema> for HostRuntime {}
