use crate::application_schema::{
    ApplicationSchemaDeclarationDenial as Denial, ApplicationSchemaMember as Member,
};
use std::collections::BTreeSet;

pub(super) fn validate(members: &[Member]) -> Result<(), Denial> {
    let mut bindings = BTreeSet::new();
    for member in members {
        let Member::ApplicationQueryInspection { definition } = member else {
            continue;
        };
        definition
            .validate()
            .map_err(|_| Denial::InvalidApplicationQueryInspection)?;
        let parts = definition.parts();
        let binding = parts.binding.parts();
        if !bindings.insert(&binding.binding) {
            return Err(Denial::DuplicateApplicationQueryInspection);
        }
        // Portable closure can check declared query meaning, never Rust binding provenance.
        if !members.iter().any(|member| {
            matches!(member, Member::ApplicationQuery { definition }
            if definition.name() == binding.query_name
            && definition.query_type() == binding.query.as_str()
            && definition.parameter_type() == binding.parameters.as_str()
            && definition.result_type() == binding.result.as_str()
            && definition.scope_type() == binding.scope.as_str()
            && definition.scope_entity() == binding.scope_entity)
        }) {
            return Err(Denial::MissingApplicationQueryInspectionDependency);
        }
        let disclosure = &parts.metadata_disclosure;
        if let (Some(name), Some(identity)) =
            (disclosure.capability_name(), disclosure.capability_type())
        {
            if !members.iter().any(|member| {
                matches!(member, Member::ApplicationCapability { contract }
                if contract.name() == name && contract.capability_type() == identity)
            }) {
                return Err(Denial::MissingApplicationQueryInspectionDependency);
            }
        }
    }
    Ok(())
}
