//! Transport retained producer roles through Native's admitted fresh-ID correction.
use super::*;
use crate::domain_computation::primary_graph::application_checkpoint::{
    CheckpointDecodeDenial, CheckpointMetadataFunding,
};
impl WorthQueryApplicationOutputCorrespondence {
    pub(in crate::domain_computation::primary_graph) fn transport_native_restoration(
        &self,
        publication: &worth_relational::facade::mvcc::RelationalRequiredOutputPublication,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, CheckpointDecodeDenial> {
        let mut funding = CheckpointMetadataFunding::new(execution)?;
        let mut optional = funding.vec::<String>(self.storage.optional_roles.len(), execution)?;
        for role in &self.storage.optional_roles {
            optional.push(funding.copy_text(role, execution)?);
        }
        let mut roles = funding
            .vec::<(String, CommittedOutputBinding)>(publication.members().len(), execution)?;
        let live = self
            .storage
            .roles
            .iter()
            .filter(|(_, binding)| binding.posture != WorthQueryApplicationOutputPosture::Retire)
            .count();
        if live != publication.members().len() {
            return Err("Native correction differs from retained producer roles"
                .to_owned()
                .into());
        }
        for (role, entity, posture, fresh) in publication.members() {
            funding.check_live(execution)?;
            let original = self
                .storage
                .binding(role)
                .filter(|original| {
                    original.entity_name == entity
                        && original.posture != WorthQueryApplicationOutputPosture::Retire
                        && posture
                            == worth_relational::facade::mvcc::RequiredOutputRolePosture::Create
                })
                .ok_or_else(|| {
                    CheckpointDecodeDenial::from(format!(
                        "Native correction has no retained producer role {role}"
                    ))
                })?;
            roles.push((
                funding.copy_text(role, execution)?,
                CommittedOutputBinding {
                    posture: original.posture,
                    entity_name: funding.copy_text(entity, execution)?,
                    entity_type: original.entity_type,
                    entity: fresh,
                },
            ));
        }
        roles.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));
        funding.slots::<CorrespondenceStorage>(1, execution)?;
        funding.slots::<Self>(1, execution)?;
        Ok(Self {
            binding_type: self.binding_type,
            contract_type: self.contract_type,
            storage: std::sync::Arc::new(CorrespondenceStorage {
                optional_roles: optional,
                roles,
                _funding: Some(funding),
            }),
        })
    }
}
