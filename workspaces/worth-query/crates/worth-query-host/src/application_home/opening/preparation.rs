//! Admission returns its original owners before origin-specific composition.
use super::*;

pub(super) struct DirectoryOpening<'authoring, Schema: ApplicationSchemaComposition, Program> {
    pub(super) query: PreparedHomeNativeMeaning<Schema, Program>,
    pub(super) home: UnsharedReadmittedApplicationHome,
    pub(super) checkpoint_limits: ApplicationHomeCheckpointLimits,
    pub(super) observation: ApplicationHomeOpeningObservation,
    pub(super) initial_state: Box<dyn FnOnce(
        &WorthQueryBootstrapAdvancementPhase<'_>,
        &mut worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphBootstrap<Schema>,
        &worth_query_installation::facade::WorthQueryInstalledApplicationSchema<Schema>,
    ) -> Result<(), worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial> + 'authoring>,
}
#[inline(never)]
pub(super) fn prepare_directory<'authoring, Schema, Program>(
    request_inputs: WorthQueryApplicationHomeRequest<'authoring, Schema, Program>,
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
) -> Result<
    DirectoryOpening<'authoring, Schema, Program>,
    WorthQueryApplicationHomeOpenRefused<Schema, Program>,
>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema> + 'static,
    Program::Outputs:
        ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    use WorthQueryApplicationHomeOpenRefused as Refused;
    let WorthQueryApplicationHomeLocation::Directory(location) = request_inputs.location else {
        unreachable!("directory preparation follows the closed location choice")
    };
    let query = prepare_home_program(
        request_inputs.program,
        request_inputs.roster,
        request_inputs.declaration,
        request_inputs.configuration,
        request_inputs.build,
        request_inputs.limits,
    )
    .map_err(Refused::Preparation)?;
    let binding = match query.admitted_home_binding() {
        Ok(binding) => binding,
        Err(cause) => return Err(Refused::Binding { query, cause }),
    };
    let request = phase.execution_request();
    let mut physical = ApplicationHomeRequest::new(
        location,
        query.home_relational_definition(),
        binding,
        request_inputs.build,
    );
    let observation = request_inputs
        .opening_control
        .as_ref()
        .map(|control| control.observation())
        .unwrap_or_default();
    if let Some(control) = request_inputs.opening_control {
        physical = physical.with_opening_control_for_test(control);
    }
    let home = match recover_home(physical, &request).and_then(|home| admit_home(home, &request)) {
        Ok(home) => home,
        Err(cause) => return Err(Refused::Physical { query, cause }),
    };
    observation.arrive_for_test(
        ApplicationHomeOpeningStage::NativeReadmission,
        ApplicationHomeOpeningEdge::Before,
    );
    let (query, home) = readmit_native(phase, query, home)?;
    observation.arrive_for_test(
        ApplicationHomeOpeningStage::NativeReadmission,
        ApplicationHomeOpeningEdge::After,
    );
    Ok(DirectoryOpening {
        query,
        home,
        checkpoint_limits: request_inputs.checkpoint_limits,
        observation,
        initial_state: request_inputs.initial_state,
    })
}

#[inline(never)]
fn readmit_native<Schema, Program>(
    phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    query: PreparedHomeApplication<Schema, Program>,
    home: worth_proof::HomeAdmitted<AdmittedApplicationHome>,
) -> Result<
    (
        PreparedHomeNativeMeaning<Schema, Program>,
        UnsharedReadmittedApplicationHome,
    ),
    WorthQueryApplicationHomeOpenRefused<Schema, Program>,
>
where
    Schema: ApplicationSchemaComposition,
    Program: ApplicationProgramDefinition<Schema> + 'static,
    Program::Outputs:
        ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
    Program::Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    use WorthQueryApplicationHomeOpenRefused as Refused;
    let request = phase.execution_request();
    let mut home = match attach_relational(home, &request) {
        Ok(home) => home,
        Err(home) => return Err(Refused::RelationalAttachment { query, home }),
    };
    let installed = match home.prepare_native_schema_for_home(&request) {
        Ok(ApplicationHomeNativeSchemaPreparation::Fresh {
            native,
            initialization,
        }) => query.initialize_home_native_schema(phase, native, initialization),
        Ok(ApplicationHomeNativeSchemaPreparation::Existing { native }) => {
            query.prepare_home_native_receiver_schema(phase, native)
        }
        Err(cause) => Err(query.refuse_home_native_schema(cause)),
    };
    let query = match installed {
        Ok(query) => query,
        Err(query) => return Err(Refused::NativeInstallation { query, home }),
    };
    let home = match home.readmit(query.home_initial_schema_receipt(), &request) {
        Ok(home) => home.consume(|home| home),
        Err(home) => return Err(Refused::NativeReadmission { query, home }),
    };
    Ok((query, home))
}
