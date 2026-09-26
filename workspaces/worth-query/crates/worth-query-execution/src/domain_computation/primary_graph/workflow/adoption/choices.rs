//! The caller's decision for every inventoried workflow fact, bound to the
//! inventory digest it was made against.

use std::collections::BTreeMap;

use worth_relational::facade::identity::EntityId;

use super::{
    WorthQueryWorkflowAdoptionInventory, WorthQueryWorkflowDefinitionDisposition,
    WorthQueryWorkflowDefinitionOccurrence, WorthQueryWorkflowInstanceDisposition,
    WorthQueryWorkflowInstanceOccurrence,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowDispositionDenial {
    /// The law does not permit this disposition for the definition.
    IllegalDefinitionDisposition {
        definition: EntityId,
        disposition: WorthQueryWorkflowDefinitionDisposition,
    },
    /// The law does not permit this disposition for the instance.
    IllegalInstanceDisposition {
        instance: EntityId,
        disposition: WorthQueryWorkflowInstanceDisposition,
    },
    /// The instance has no legal disposition; settle or recover it under the
    /// source program, or migrate it, then take a fresh inventory.
    MigrationRequired { instance: EntityId },
    /// The inventory holds no definition with this identity.
    UnknownDefinition { definition: EntityId },
    /// The inventory holds no instance with this identity.
    UnknownInstance { instance: EntityId },
    /// A definition was left undecided.
    DefinitionUndecided { definition: EntityId },
    /// An instance was left undecided.
    InstanceUndecided { instance: EntityId },
}

impl std::fmt::Display for WorthQueryWorkflowDispositionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "workflow disposition refused: {self:?}")
    }
}

impl std::error::Error for WorthQueryWorkflowDispositionDenial {}

/// One disposition per inventoried definition and instance.
///
/// Built from one inventory and bound to its digest, so preparation refuses
/// the choices if owner truth moved after the caller decided.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryWorkflowDispositions {
    pub(super) digest: [u8; 32],
    pub(super) definitions: BTreeMap<EntityId, WorthQueryWorkflowDefinitionDisposition>,
    pub(super) instances: BTreeMap<EntityId, WorthQueryWorkflowInstanceDisposition>,
}

impl WorthQueryWorkflowDispositions {
    /// The digest of the inventory these choices decide, not of the choices.
    pub const fn inventory_digest(&self) -> &[u8; 32] {
        &self.digest
    }

    pub fn definition(
        mut self,
        occurrence: &WorthQueryWorkflowDefinitionOccurrence,
        disposition: WorthQueryWorkflowDefinitionDisposition,
    ) -> Result<Self, WorthQueryWorkflowDispositionDenial> {
        if !occurrence.legal_dispositions().contains(&disposition) {
            return Err(
                WorthQueryWorkflowDispositionDenial::IllegalDefinitionDisposition {
                    definition: occurrence.entity_id(),
                    disposition,
                },
            );
        }
        self.definitions.insert(occurrence.entity_id(), disposition);
        Ok(self)
    }

    pub fn instance(
        mut self,
        occurrence: &WorthQueryWorkflowInstanceOccurrence,
        disposition: WorthQueryWorkflowInstanceDisposition,
    ) -> Result<Self, WorthQueryWorkflowDispositionDenial> {
        if occurrence.requires_migration() {
            return Err(WorthQueryWorkflowDispositionDenial::MigrationRequired {
                instance: occurrence.entity_id(),
            });
        }
        if !occurrence.legal_dispositions().contains(&disposition) {
            return Err(
                WorthQueryWorkflowDispositionDenial::IllegalInstanceDisposition {
                    instance: occurrence.entity_id(),
                    disposition,
                },
            );
        }
        self.instances.insert(occurrence.entity_id(), disposition);
        Ok(self)
    }

    pub fn definition_disposition(
        &self,
        definition: EntityId,
    ) -> Option<WorthQueryWorkflowDefinitionDisposition> {
        self.definitions.get(&definition).copied()
    }

    pub fn instance_disposition(
        &self,
        instance: EntityId,
    ) -> Option<WorthQueryWorkflowInstanceDisposition> {
        self.instances.get(&instance).copied()
    }
}

impl WorthQueryWorkflowAdoptionInventory {
    /// Empty choices bound to this inventory's digest.
    pub fn dispositions(&self) -> WorthQueryWorkflowDispositions {
        WorthQueryWorkflowDispositions {
            digest: self.digest,
            definitions: BTreeMap::new(),
            instances: BTreeMap::new(),
        }
    }

    /// Carries every definition and instance, refusing at the first one the
    /// target cannot carry.
    pub fn carry_compatible(
        &self,
    ) -> Result<WorthQueryWorkflowDispositions, WorthQueryWorkflowDispositionDenial> {
        let mut choices = self.dispositions();
        for definition in self.definitions() {
            choices =
                choices.definition(definition, WorthQueryWorkflowDefinitionDisposition::Carry)?;
        }
        for instance in self.instances() {
            choices = choices.instance(instance, WorthQueryWorkflowInstanceDisposition::Carry)?;
        }
        Ok(choices)
    }

    /// Checks that `choices` decide exactly this inventory, each legally.
    pub(in crate::domain_computation::primary_graph) fn admit(
        &self,
        choices: &WorthQueryWorkflowDispositions,
    ) -> Result<(), WorthQueryWorkflowDispositionDenial> {
        for (&definition, &disposition) in &choices.definitions {
            let occurrence = self
                .definition(definition)
                .ok_or(WorthQueryWorkflowDispositionDenial::UnknownDefinition { definition })?;
            self.dispositions().definition(occurrence, disposition)?;
        }
        for (&instance, &disposition) in &choices.instances {
            let occurrence = self
                .instance(instance)
                .ok_or(WorthQueryWorkflowDispositionDenial::UnknownInstance { instance })?;
            self.dispositions().instance(occurrence, disposition)?;
        }
        if let Some(occurrence) = self
            .instances()
            .iter()
            .find(|occurrence| occurrence.requires_migration())
        {
            return Err(WorthQueryWorkflowDispositionDenial::MigrationRequired {
                instance: occurrence.entity_id(),
            });
        }
        if let Some(occurrence) = self
            .definitions()
            .iter()
            .find(|occurrence| !choices.definitions.contains_key(&occurrence.entity_id()))
        {
            return Err(WorthQueryWorkflowDispositionDenial::DefinitionUndecided {
                definition: occurrence.entity_id(),
            });
        }
        if let Some(occurrence) = self
            .instances()
            .iter()
            .find(|occurrence| !choices.instances.contains_key(&occurrence.entity_id()))
        {
            return Err(WorthQueryWorkflowDispositionDenial::InstanceUndecided {
                instance: occurrence.entity_id(),
            });
        }
        Ok(())
    }
}
