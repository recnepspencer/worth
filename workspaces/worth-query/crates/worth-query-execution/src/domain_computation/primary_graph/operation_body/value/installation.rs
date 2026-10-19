//! Only exact installed destination codecs may admit a computed scalar.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryPrimaryGraphInstallationDenial as Denial,
    WorthQueryPrimaryGraphInstallationDenialKind as Kind,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationBodyCreatedField as Created, ApplicationOperationBodyProgram,
    ApplicationOperationBodyScopeField as Scope,
};
pub(in crate::domain_computation::primary_graph::operation_body) fn install<
    S: worth_query_installation::facade::ApplicationSchema,
>(
    schema: &worth_query_installation::facade::WorthQueryInstalledApplicationSchema<S>,
    body: &ApplicationOperationBodyProgram,
    operation: &str,
) -> Result<Vec<InstalledBodyValue>, Denial> {
    let denied = || Denial::new(Kind::OperationBodyUnavailable, operation);
    body.values()
        .iter()
        .map(|program| {
            let fields = body
                .scope_fields()
                .iter()
                .filter_map(|step| match step {
                    Scope::AssignValue { field, value, .. } if value == program.identity() => {
                        Some(field)
                    }
                    _ => None,
                })
                .chain(
                    body.creation()
                        .into_iter()
                        .flat_map(|creation| creation.initialization_sites())
                        .flat_map(|creation| creation.fields())
                        .filter_map(|step| match step {
                            Created::Computed { field, value, .. }
                                if value == program.identity() =>
                            {
                                Some(field)
                            }
                            _ => None,
                        }),
                )
                .chain(
                    body.path_value_assignments()
                        .iter()
                        .filter(|step| step.value == program.identity())
                        .map(|step| &step.target.field),
                );
            let bindings = fields
                .map(|field| {
                    let binding = schema
                        .value_bindings()
                        .field(field.entity(), field.aspect(), field.field())
                        .ok_or_else(denied)?;
                    if binding.identity() != program.binding()
                        || binding.scalar_family()
                            != worth_foundational::facade::ScalarAspectType::UInt64
                        || binding.unit().is_some()
                        || binding.frame().is_some()
                        || !binding.is_readable()
                        || !binding.admits_native_input()
                    {
                        return Err(denied());
                    }
                    Ok(binding.clone())
                })
                .collect::<Result<Vec<_>, Denial>>()?;
            if bindings.is_empty() {
                return Err(denied());
            }
            Ok(InstalledBodyValue {
                program: program.clone(),
                bindings,
            })
        })
        .collect()
}
