use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_query_declaration::facade::application_schema::ApplicationOperationProgramTarget;
use worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{CreatedEntityRef, EntityReference};

use super::WorthQueryDelegationActivationBinding;
use crate::domain_computation::application_contract_admission::application_contract_contains_program_targets;

/// One complete effect minted by the delegation-authorization owner.
pub enum WorthQueryDelegationActivationEffect {
    CreateEntity {
        kind: KindId,
        key: String,
        fields: std::collections::BTreeMap<AspectFieldLocator, AspectValue>,
    },
    CreateRelation {
        kind: KindId,
        key: String,
        from: EntityReference,
        to: EntityReference,
    },
}

impl WorthQueryDelegationActivationBinding {
    /// Validate the installed program and lower the exact authorized activation.
    ///
    /// Callers receive complete effects, never the independent child, principal,
    /// resource, relationship, or context axes used to authorize them.
    pub fn materialize_program(
        &self,
        installed: &WorthQueryCompiledApplicationOperationContracts,
    ) -> Result<Vec<WorthQueryDelegationActivationEffect>, ()> {
        self.validate_installed_program(installed)?;
        let key = canonical_child_key(&self.child_key)?;
        let child = EntityReference::Created(CreatedEntityRef {
            partition_id: worth_relational::facade::identity::PartitionId::main(),
            kind_id: self.child_kind,
            client_key: ClientKey::raw(key.clone()),
        });
        let mut effects = Vec::with_capacity(
            5usize
                .saturating_add(usize::from(self.related.is_some()))
                .saturating_add(self.activation_context.len()),
        );
        effects.push(WorthQueryDelegationActivationEffect::CreateEntity {
            kind: self.child_kind,
            key: key.clone(),
            fields: self.fields.clone(),
        });
        effects.push(relation(
            self.parent_relation,
            &key,
            child.clone(),
            EntityReference::Existing(self.parent),
        ));
        effects.push(relation(
            self.grantor_relation,
            &key,
            EntityReference::Existing(self.grantor),
            child.clone(),
        ));
        effects.push(relation(
            self.grantee_relation,
            &key,
            EntityReference::Existing(self.grantee),
            child.clone(),
        ));
        effects.push(relation(
            self.resource_relation,
            &key,
            child.clone(),
            EntityReference::Existing(self.resource),
        ));
        match (self.related_relation, self.related) {
            (Some(kind), Some(entity)) => effects.push(relation(
                kind,
                &key,
                child.clone(),
                EntityReference::Existing(entity),
            )),
            (None, None) => {}
            _ => return Err(()),
        }
        effects.extend(self.activation_context.iter().map(|(kind, entity)| {
            relation(
                *kind,
                &key,
                child.clone(),
                EntityReference::Existing(*entity),
            )
        }));
        Ok(effects)
    }

    fn validate_installed_program(
        &self,
        installed: &WorthQueryCompiledApplicationOperationContracts,
    ) -> Result<(), ()> {
        let creates = self
            .required_program_targets
            .iter()
            .filter(|target| matches!(target, ApplicationOperationProgramTarget::Create { .. }))
            .count();
        let writes = self
            .required_program_targets
            .iter()
            .filter(|target| matches!(target, ApplicationOperationProgramTarget::Write { .. }))
            .count();
        let links = self
            .required_program_targets
            .iter()
            .filter(|target| matches!(target, ApplicationOperationProgramTarget::Link { .. }))
            .count();
        let expected_links = 4usize
            .saturating_add(usize::from(self.related.is_some()))
            .saturating_add(self.activation_context.len());
        (application_contract_contains_program_targets(installed, &self.required_program_targets)
            && creates == 1
            && writes == self.fields.len()
            && links == expected_links
            && self.required_program_targets.len()
                == creates.saturating_add(writes).saturating_add(links))
        .then_some(())
        .ok_or(())
    }
}

fn relation(
    kind: KindId,
    key: &str,
    from: EntityReference,
    to: EntityReference,
) -> WorthQueryDelegationActivationEffect {
    WorthQueryDelegationActivationEffect::CreateRelation {
        kind,
        key: key.to_owned(),
        from,
        to,
    }
}

fn canonical_child_key(value: &str) -> Result<String, ()> {
    if value.is_empty()
        || value.trim() != value
        || value.len() > 512
        || value.chars().any(char::is_control)
    {
        Err(())
    } else {
        Ok(value.to_owned())
    }
}
