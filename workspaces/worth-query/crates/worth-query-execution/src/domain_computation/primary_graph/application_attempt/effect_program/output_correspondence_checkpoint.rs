//! Reconstruct correspondence only after admitting its independent owned payload.
use crate::equality_type_id::EqualityTypeId as TypeId;
use worth_query_declaration::facade::application_operation::WorthQueryApplicationOutputRoleNameDenial;
use {
    crate::domain_computation::primary_graph::vocabulary::application_attempt::output_correspondence::CommittedOutputBinding,
    crate::domain_computation::primary_graph::vocabulary::application_attempt::output_correspondence::CorrespondenceStorage,
    crate::domain_computation::primary_graph::vocabulary::application_attempt::output_correspondence::WorthQueryApplicationOutputCorrespondence,
    crate::domain_computation::primary_graph::vocabulary::application_attempt::output_correspondence::WorthQueryCheckpointOutputRole,
};
use crate::domain_computation::primary_graph::application_checkpoint::{
    CheckpointDecodeDenial, CheckpointMetadataFunding,
};
use worth_execution::ExecutionRequest;

/// The attempt's restoration of a committed output correspondence from the
/// roles an application checkpoint recorded, funded before it allocates.
pub(in crate::domain_computation::primary_graph) trait OutputCorrespondenceRestoration:
    Sized
{
    fn from_checkpoint_roles<'a>(
        binding_type: TypeId,
        contract_type: TypeId,
        optional_roles: impl Iterator<Item = &'a str> + Clone,
        roles: &[WorthQueryCheckpointOutputRole],
        entity_type: impl FnMut(&str) -> Option<TypeId>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, CheckpointDecodeDenial>;
}

impl OutputCorrespondenceRestoration for WorthQueryApplicationOutputCorrespondence {
    fn from_checkpoint_roles<'a>(
        binding_type: TypeId,
        contract_type: TypeId,
        optional_roles: impl Iterator<Item = &'a str> + Clone,
        roles: &[WorthQueryCheckpointOutputRole],
        mut entity_type: impl FnMut(&str) -> Option<TypeId>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, CheckpointDecodeDenial> {
        let mut funding = CheckpointMetadataFunding::new(execution)?;
        let mut optional = funding.vec::<String>(optional_roles.clone().count(), execution)?;
        for role in optional_roles {
            optional.push(funding.copy_text(role, execution)?);
        }
        optional.sort_unstable();
        if optional.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err("checkpoint output optional role is duplicated"
                .to_owned()
                .into());
        }
        let mut rebound =
            funding.vec::<(String, CommittedOutputBinding)>(roles.len(), execution)?;
        for role in roles {
            funding.check_live(execution)?;
            WorthQueryApplicationOutputRoleNameDenial::validate(&role.role)
                .map_err(|denial| format!("checkpoint output role {}: {denial}", role.role))?;
            let marker = entity_type(&role.entity_name).ok_or_else(|| {
                format!(
                    "checkpoint output role {} names an uninstalled entity {}",
                    role.role, role.entity_name
                )
            })?;
            rebound.push((
                funding.copy_text(&role.role, execution)?,
                CommittedOutputBinding::new(
                    role.posture,
                    funding.copy_text(&role.entity_name, execution)?,
                    marker,
                    role.entity,
                ),
            ));
        }
        rebound.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));
        if let Some(pair) = rebound.windows(2).find(|pair| pair[0].0 == pair[1].0) {
            return Err(format!("checkpoint output role {} is duplicated", pair[0].0).into());
        }
        // Both shared payload objects are admitted before Arc construction. Clones
        // share this store, so they cannot duplicate heaps or abandon their payer.
        funding.slots::<CorrespondenceStorage>(1, execution)?;
        funding.slots::<Self>(1, execution)?;
        Ok(Self::from_restored(
            binding_type,
            contract_type,
            optional,
            rebound,
            funding.into_reservation(),
        ))
    }
}
#[cfg(test)]
mod tests;

pub(crate) mod court_doors_for_test;
