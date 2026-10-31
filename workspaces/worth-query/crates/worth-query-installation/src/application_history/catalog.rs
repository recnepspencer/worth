use super::{
    WorthQueryDocumentHistoryInstallationDenial as Denial,
    WorthQueryDocumentHistoryInstallationDenialKind as Kind,
};
use crate::application_schema::WorthQueryInstalledApplicationSchema;
use std::{collections::BTreeMap, sync::Arc};
use worth_query_declaration::facade::{
    application_history::{
        ApplicationHistoryBindingDescriptor, ErasedApplicationHistoryDefinition,
    },
    application_schema::{
        ApplicationSchema, ApplicationSchemaBindingIdentity, ApplicationSchemaMember,
    },
};

pub(super) struct CompiledApplicationHistory {
    pub(super) schema: ApplicationSchemaBindingIdentity,
    pub(super) definition: ErasedApplicationHistoryDefinition,
    pub(super) descriptor: ApplicationHistoryBindingDescriptor,
    pub(super) has_external_operation: bool,
}

#[derive(Default)]
pub(crate) struct WorthQueryInstalledApplicationHistoryCatalog {
    histories: BTreeMap<String, Arc<CompiledApplicationHistory>>,
}

impl WorthQueryInstalledApplicationHistoryCatalog {
    pub(super) fn get(&self, identity: &str) -> Option<Arc<CompiledApplicationHistory>> {
        self.histories.get(identity).map(Arc::clone)
    }

    pub(crate) fn compile<S: ApplicationSchema>(
        schema: &WorthQueryInstalledApplicationSchema<S>,
    ) -> Result<Self, Denial> {
        let mut histories = BTreeMap::new();
        for member in schema.installed_declaration().members() {
            let ApplicationSchemaMember::ApplicationHistory { definition } = member else {
                continue;
            };
            let parts = definition.parts();
            definition
                .validate()
                .map_err(|_| Denial::new(Kind::BindingMeaningChanged, &parts.identity))?;
            let mut matches = schema
                .member_provenance
                .document_history_bindings()
                .iter()
                .filter(|descriptor| descriptor.identity() == parts.identity);
            let descriptor = matches
                .next()
                .ok_or_else(|| Denial::new(Kind::MissingTypedBinding, &parts.identity))?;
            if matches.next().is_some()
                || descriptor.history_type() != &parts.history_type
                || !parts.access.matches_descriptor(descriptor.access())
                || descriptor.operations().len() != parts.operations.len()
                || descriptor
                    .navigation()
                    .map(|navigation| navigation.declaration())
                    != parts.navigation.as_ref()
            {
                return Err(Denial::new(Kind::BindingMeaningChanged, &parts.identity));
            }
            let access = schema
                .query_catalog
                .get_binding(descriptor.access().identity())
                .filter(|binding| binding.descriptor() == descriptor.access())
                .ok_or_else(|| Denial::new(Kind::AccessBindingMismatch, &parts.identity))?;
            for (declared, operation) in parts.operations.iter().zip(descriptor.operations()) {
                let installed = schema
                    .mutation_catalog
                    .get_binding(operation.identity())
                    .filter(|binding| binding.descriptor() == operation)
                    .ok_or_else(|| {
                        Denial::new(Kind::OperationBindingMismatch, &declared.binding)
                    })?;
                if declared.binding != operation.identity()
                    || installed.descriptor().scope_entity()
                        != access.descriptor().scope().field().locus().entity()
                {
                    return Err(Denial::new(
                        Kind::OperationBindingMismatch,
                        &declared.binding,
                    ));
                }
            }
            let compiled = Arc::new(CompiledApplicationHistory {
                schema: schema.binding_identity(),
                definition: definition.clone(),
                descriptor: descriptor.clone(),
                has_external_operation: schema.installed_declaration().members().iter().any(|member| {
                    matches!(member, ApplicationSchemaMember::OperationExternalEffect { operation, .. }
                        if descriptor.operations().iter().any(|binding| binding.operation_name() == operation)
                            || parts.navigation.as_ref().is_some_and(|navigation| navigation.parts().operation == *operation))
                }),
            });
            if histories.insert(parts.identity.clone(), compiled).is_some() {
                return Err(Denial::new(Kind::BindingMeaningChanged, &parts.identity));
            }
        }
        if schema
            .member_provenance
            .document_history_bindings()
            .iter()
            .any(|descriptor| !histories.contains_key(descriptor.identity()))
        {
            return Err(Denial::new(
                Kind::BindingMeaningChanged,
                schema.schema_name(),
            ));
        }
        Ok(Self { histories })
    }
}
