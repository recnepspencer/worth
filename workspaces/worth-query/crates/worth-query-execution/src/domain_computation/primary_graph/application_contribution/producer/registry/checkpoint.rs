use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledApplicationSchema};

use super::WorthQueryInstalledApplicationProducerRegistry;

impl<Schema> WorthQueryInstalledApplicationProducerRegistry<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub(in crate::domain_computation::primary_graph) fn readmit_checkpoint_output(
        &self,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        checkpoint: &crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity,
    ) -> Result<
        std::sync::Arc<
            crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence,
        >,
        String,
    > {
        let installed = installed_checkpoint_producer(&self.entries, &checkpoint.producer)?;
        validate_checkpoint_output_meaning(&installed.declaration, checkpoint)?;
        crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence::from_checkpoint_roles(
            installed.declaration.operation_binding_type,
            checkpoint.roles.clone(),
            |entity| installed_schema.installed_entity_marker_type(entity),
        )
        .map(std::sync::Arc::new)
    }
}

pub(super) fn installed_checkpoint_producer<'entries, Schema>(
    entries: &'entries std::collections::BTreeMap<String, super::InstalledProducerProvider<Schema>>,
    producer: &str,
) -> Result<&'entries super::InstalledProducerProvider<Schema>, String> {
    entries
        .get(producer)
        .ok_or_else(|| format!("checkpoint output producer {producer} is not installed"))
}

pub(super) fn validate_checkpoint_output_meaning(
    installed: &super::DeclaredProducerBinding,
    checkpoint: &crate::domain_computation::primary_graph::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity,
) -> Result<(), String> {
    for role in &checkpoint.roles {
        let exact = installed
            .output_role_descriptors
            .iter()
            .find(|expected| expected.name() == role.role);
        let family = installed.output_role_families.iter().find(|expected| {
            role.role
                .strip_prefix(expected.prefix())
                .is_some_and(|member| !member.is_empty())
        });
        let meaning_matches = match (exact, family) {
            (Some(expected), None) => {
                expected.entity() == role.entity_name && expected.posture() == role.posture
            }
            (None, Some(expected)) => {
                expected.entity() == role.entity_name && expected.postures().allows(role.posture)
            }
            _ => false,
        };
        if !meaning_matches {
            return Err(format!(
                "checkpoint output role {} differs from installed producer {}",
                role.role, checkpoint.producer
            ));
        }
    }
    if let Some(missing) = installed.output_role_descriptors.iter().find(|expected| {
        !checkpoint
            .roles
            .iter()
            .any(|role| role.role == expected.name())
    }) {
        return Err(format!(
            "checkpoint output omits installed role {}",
            missing.name()
        ));
    }
    for family in &installed.output_role_families {
        let members = checkpoint
            .roles
            .iter()
            .filter(|role| {
                role.role
                    .strip_prefix(family.prefix())
                    .is_some_and(|member| !member.is_empty())
            })
            .count();
        if members < family.minimum() {
            return Err(format!(
                "checkpoint output family {} is incomplete",
                family.prefix()
            ));
        }
    }
    Ok(())
}
