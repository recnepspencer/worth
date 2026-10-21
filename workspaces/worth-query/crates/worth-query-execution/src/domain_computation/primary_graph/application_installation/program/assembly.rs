//! Assemble the one admitted program roster and native runtime into its installed owner.
use super::super::program_admission::WorthQueryAdmittedProgramSupport;
use super::super::WorthQueryInMemoryApplicationDenial;
use super::supported_program::WorthQuerySupportedProgramRecord;
use super::{
    WorthQueryApplicationProgramRoots, WorthQueryApplicationProgramRoster,
    WorthQueryProgramApplicationRuntime,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use crate::equality_type_id::EqualityTypeId as TypeId;
use worth_query_declaration::facade::application_program::{
    ApplicationProgramDefinition, ApplicationProgramOutputsShape, ValidatedApplicationProgram,
};
use worth_query_installation::facade::{
    install_rostered_application_program, WorthQueryInstalledApplicationProgram,
    WorthQueryInstalledApplicationSchema, WorthQueryProgramSupportAdmission,
};

pub(super) fn assemble<Schema, Program>(
    mut runtime: WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    admitted: Option<(
        WorthQueryInstalledApplicationProgram<Schema, Program>,
        Vec<WorthQuerySupportedProgramRecord>,
    )>,
) -> Result<WorthQueryProgramApplicationRuntime<Schema, Program>, WorthQueryInMemoryApplicationDenial>
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema> + 'static,
    Program::Outputs:
        ApplicationProgramOutputsShape<Schema> + WorthQueryApplicationProgramRoots<Schema>,
{
    let (installed, supported) =
        admitted.ok_or(WorthQueryInMemoryApplicationDenial::ProgramAdmissionIncomplete)?;
    if let Some(support) = runtime.program_support.as_mut() {
        use crate::domain_computation::primary_graph::product_operation::program_adoption::authorization::admit_program;
        support
            .adoption_admissions
            .insert(*installed.revision(), admit_program::<Schema, Program>);
        for record in &supported {
            support
                .adoption_admissions
                .insert(*record.revision(), record.adoption_admission::<Schema>());
        }
    }
    super::conditional::validate_conditional_actions(
        installed.actions(),
        runtime.installed_conditionals.operation_types(),
    )?;
    let mut seen_action_bindings = std::collections::HashSet::new();
    let action_bindings = installed
        .actions()
        .iter()
        .filter_map(|action| action.ordinary_mutation_binding_type())
        .map(TypeId::from)
        .filter(|binding| seen_action_bindings.insert(*binding))
        .collect::<Vec<_>>();
    runtime.program_required_operations.extend(
        installed
            .actions()
            .iter()
            .map(|action| TypeId::from(action.operation_type())),
    );
    runtime.program_required_operations.extend(
        runtime
            .installed_schema
            .installed_mutation_binding_inventory()
            .filter(|binding| action_bindings.contains(&TypeId::from(binding.binding_type())))
            .map(|binding| TypeId::from(binding.operation_type())),
    );
    runtime
        .program_required_bindings
        .extend(action_bindings.iter().copied());
    let output_source_bindings = super::output_roots::required_output_bindings::<
        Schema,
        Program::Outputs,
    >(installed.actions());
    require_rostered_binding_membership(&runtime, &installed, &output_source_bindings)?;
    runtime
        .program_required_bindings
        .extend(output_source_bindings.iter().copied());
    Ok(WorthQueryProgramApplicationRuntime {
        backing: std::sync::Arc::new(super::program_runtime::WorthQueryProgramRuntimeBacking {
            runtime,
            program: installed,
            connection_types: Program::Outputs::connection_types()
                .into_iter()
                .map(TypeId::from)
                .collect(),
            root_graph_types: Program::Outputs::root_graph_types()
                .into_iter()
                .map(TypeId::from)
                .collect(),
            output_source_bindings: output_source_bindings.into_boxed_slice(),
            action_bindings: action_bindings.into_iter().collect(),
            supported: supported.into_boxed_slice(),
        }),
    })
}

/// Admits the complete roster, then installs every rostered program against it.
///
/// Admission closes before any program is installed, so nothing is ever
/// installed against a roster that is still growing.
pub(super) fn admit_program_roster<Schema, Program>(
    program: ValidatedApplicationProgram<Schema, Program>,
    roster: WorthQueryApplicationProgramRoster<'_, Schema>,
    installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
) -> Result<
    (
        WorthQueryAdmittedProgramSupport<Schema>,
        WorthQueryInstalledApplicationProgram<Schema, Program>,
        Vec<WorthQuerySupportedProgramRecord>,
    ),
    WorthQueryInMemoryApplicationDenial,
>
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema> + 'static,
{
    let admission = WorthQueryProgramSupportAdmission::for_installed_schema(installed_schema)
        .support(&program)
        .map_err(program_support_denied)?;
    let closed = roster
        .admit_all(admission)
        .map_err(program_support_denied)?
        .close()
        .map_err(program_support_denied)?;
    let supported = roster
        .install_all(installed_schema, &closed)
        .map_err(program_support_denied)?;
    let initial_revision = *program.revision();
    let installed = install_rostered_application_program(program, installed_schema, &closed)
        .map_err(program_support_denied)?;
    Ok((
        WorthQueryAdmittedProgramSupport {
            roster: std::sync::Arc::new(closed),
            initial_revision,
            features: installed
                .features()
                .iter()
                .chain(
                    supported
                        .iter()
                        .flat_map(|record| record.installed.features()),
                )
                .cloned()
                .collect(),
        },
        installed,
        supported,
    ))
}

/// Requires that every installed binding demanding a program is answered by the
/// roster as a whole: an action of some rostered program, or an output source.
fn require_rostered_binding_membership<Schema, Program>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    installed: &WorthQueryInstalledApplicationProgram<Schema, Program>,
    output_source_bindings: &[TypeId],
) -> Result<(), WorthQueryInMemoryApplicationDenial>
where
    Schema: worth_query_declaration::facade::application_schema::ApplicationSchema,
{
    let mut answered = output_source_bindings
        .iter()
        .copied()
        .map(TypeId::into_external)
        .collect::<std::collections::HashSet<_>>();
    if let Some(support) = runtime.installed_program_support() {
        answered.extend(
            runtime
                .installed_schema()
                .installed_mutation_binding_inventory()
                .filter(|binding| support.declares_mutation_binding(binding.binding_type().into()))
                .map(|binding| binding.binding_type()),
        );
    }
    worth_query_installation::facade::require_complete_program_binding_membership(
        installed,
        runtime.installed_schema(),
        &answered,
    )
    .map_err(WorthQueryInMemoryApplicationDenial::Program)
}

fn program_support_denied(
    denial: worth_query_installation::facade::WorthQueryProgramSupportDenial,
) -> WorthQueryInMemoryApplicationDenial {
    WorthQueryInMemoryApplicationDenial::Program(denial.into())
}
