//! Closed installed policy meaning, captured once beside its vocabulary.
use super::*;
use worth_query_declaration::facade::{
    application_program::{
        ApplicationWorkflowActorPolicy, ApplicationWorkflowAuditPolicy,
        ApplicationWorkflowObservationPolicy,
    },
    application_schema::ApplicationSchemaMember,
};

impl<'installed, Schema, Spec, Program>
    WorthQueryApplicationWorkflowSpecInstallation<'installed, Schema, Spec, Program>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
    pub(super) fn validate_policies(
        &self,
        actors: &ApplicationWorkflowActorPolicy,
        reads: &ApplicationWorkflowObservationPolicy,
    ) -> Result<(), WorthQueryApplicationWorkflowInstallationDenial> {
        let invalid = || {
            denial(
                WorthQueryApplicationWorkflowInstallationDenialKind::InvalidPolicy,
                Spec::IDENTITY.as_str(),
            )
        };
        if !reads.is_valid()
            || matches!(actors, ApplicationWorkflowActorPolicy::Bound(entries) if entries.is_empty())
        {
            return Err(invalid());
        }
        let members = self.schema.installed_declaration().members();
        let capabilities = || {
            members.iter().filter_map(|member| match member {
                ApplicationSchemaMember::ApplicationCapability { contract } => Some(contract),
                _ => None,
            })
        };
        let mut seen = std::collections::BTreeSet::new();
        for entry in actors.bindings() {
            if !seen.insert(entry.binding())
                || !self
                    .program
                    .actions()
                    .iter()
                    .any(|action| action.binding() == entry.binding())
            {
                return Err(invalid());
            }
            let descriptor = self
                .schema
                .installed_mutation_binding_inventory()
                .find(|descriptor| {
                    descriptor.identity() == entry.binding()
                        && descriptor.operation_name() == entry.operation()
                })
                .ok_or_else(invalid)?;
            let is_operation = self.operations.iter().any(|operation| {
                operation.identifier == entry.operation()
                    && operation.binding_identity == entry.binding()
                    && operation
                        .capability
                        .as_ref()
                        .is_some_and(|(name, identity)| {
                            (*name, identity.as_str()) == entry.capability()
                        })
            });
            let is_approval = self.approvals.iter().any(|approval| {
                approval.binding.operation_identifier == entry.operation()
                    && approval.binding.identifier == entry.capability().0
                    && approval.binding.capability_type.as_str() == entry.capability().1
            });
            if !is_operation && !is_approval {
                return Err(invalid());
            }
            let mut requirements = capabilities()
                .filter(|contract| {
                    contract.operation() == entry.operation()
                        && contract.name() == entry.capability().0
                        && contract.capability_type() == entry.capability().1
                })
                .filter_map(|contract| {
                    contract
                        .composition()
                        .actors()
                        .recorded_actor()
                        .requirement()
                });
            let Some(requirement) = requirements.next() else {
                return Err(invalid());
            };
            if !requirement.is_valid()
                || requirement.source() != entry.source()
                || requirements.any(|other| other != requirement)
                || descriptor.workflow_control() && !is_approval
            {
                return Err(invalid());
            }
        }
        // A positive rule used by this vocabulary must have an exact typed
        // binding. A declared but unenforced annotation cannot install here.
        for contract in capabilities().filter(|contract| {
            contract
                .composition()
                .actors()
                .recorded_actor()
                .requirement()
                .is_some()
        }) {
            let used = self.operations.iter().any(|operation| {
                operation.identifier == contract.operation()
                    && operation
                        .capability
                        .as_ref()
                        .is_some_and(|(name, identity)| {
                            *name == contract.name()
                                && identity.as_str() == contract.capability_type()
                        })
            }) || self.approvals.iter().any(|approval| {
                approval.binding.identifier == contract.name()
                    && approval.binding.capability_type.as_str() == contract.capability_type()
                    && approval.binding.operation_identifier == contract.operation()
            });
            if used
                && !actors.bindings().iter().any(|entry| {
                    entry.operation() == contract.operation()
                        && entry.capability() == (contract.name(), contract.capability_type())
                })
            {
                return Err(invalid());
            }
        }
        if let ApplicationWorkflowObservationPolicy::Governed {
            instance,
            definition,
            definition_metadata,
            audit,
        } = reads
        {
            for rule in [
                Some(instance),
                Some(definition),
                definition_metadata.as_ref(),
                match audit {
                    ApplicationWorkflowAuditPolicy::NotApplicable => None,
                    ApplicationWorkflowAuditPolicy::Governed(rule) => Some(rule),
                },
            ]
            .into_iter()
            .flatten()
            {
                let (name, identity) = rule.capability();
                if !capabilities().any(|contract| {
                    contract.name() == name
                        && contract.capability_type() == identity
                        && contract.operation() == rule.binding().operation()
                        && contract.input_type() == rule.binding().input()
                }) {
                    return Err(invalid());
                }
                if !members
                    .iter()
                    .any(|member| rule.binding().matches_principal_member(member))
                {
                    return Err(invalid());
                }
            }
        }
        Ok(())
    }
}
