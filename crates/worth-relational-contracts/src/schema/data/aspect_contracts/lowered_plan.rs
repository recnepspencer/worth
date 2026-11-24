use std::collections::BTreeMap;

use smallvec::SmallVec;
use worth_foundational::facade::AspectKey;
use worth_foundational::{AspectContract, FieldKey};

use crate::identity::data::KindId;

use super::{AspectBinding, AspectContractPlanRevision};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AspectContractPlanCatalog {
    pub entity_plans: BTreeMap<KindId, LoweredAspectContractPlan>,
    pub relation_plans: BTreeMap<KindId, LoweredAspectContractPlan>,
}

impl AspectContractPlanCatalog {
    pub fn empty() -> Self {
        Self {
            entity_plans: BTreeMap::new(),
            relation_plans: BTreeMap::new(),
        }
    }
}

impl Default for AspectContractPlanCatalog {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredAspectContractPlan {
    pub kind_id: KindId,
    pub plan_revision: AspectContractPlanRevision,
    pub executable_bindings: SmallVec<[LoweredAspectContractBinding; 8]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredAspectContractBinding {
    pub contract: AspectContract,
    pub target: AspectBinding,
}

impl LoweredAspectContractBinding {
    pub fn aspect_key(&self) -> &AspectKey {
        self.contract.key()
    }

    pub fn aspect_shape(&self) -> worth_foundational::AspectShape {
        self.contract.shape().clone()
    }

    pub fn targets_entity_scalar_field(&self, target: &FieldKey) -> bool {
        matches!(&self.target, AspectBinding::EntityField { field } if field == target)
            && matches!(
                self.contract.shape(),
                worth_foundational::AspectShape::Scalar(_)
            )
    }

    pub fn targets_entity_struct_field(&self, target: &FieldKey) -> bool {
        matches!(&self.target, AspectBinding::EntityField { .. })
            && self.struct_contract_declares_field(target)
    }

    pub fn targets_relation_scalar_field(&self, target: &FieldKey) -> bool {
        matches!(&self.target, AspectBinding::RelationField { field } if field == target)
            && matches!(
                self.contract.shape(),
                worth_foundational::AspectShape::Scalar(_)
            )
    }

    pub fn targets_relation_struct_field(&self, target: &FieldKey) -> bool {
        matches!(&self.target, AspectBinding::RelationField { .. })
            && self.struct_contract_declares_field(target)
    }

    pub fn struct_contract_declares_field(&self, target: &FieldKey) -> bool {
        let worth_foundational::AspectShape::Struct(shape) = self.contract.shape() else {
            return false;
        };
        shape.field(target).is_some()
    }
}

impl LoweredAspectContractPlan {
    pub fn contract_for(&self, key: &AspectKey) -> Option<AspectContract> {
        self.executable_bindings
            .iter()
            .find(|binding| binding.aspect_key() == key)
            .map(|binding| binding.contract.clone())
    }

    pub fn admits_entity_scalar_field(&self, target: &FieldKey) -> bool {
        self.entity_scalar_field_aspect_key(target).is_some()
    }

    pub fn entity_scalar_field_aspect_key(&self, target: &FieldKey) -> Option<AspectKey> {
        self.executable_bindings
            .iter()
            .find(|binding| binding.targets_entity_scalar_field(target))
            .map(|binding| binding.aspect_key().clone())
    }

    pub fn admits_entity_field_update_target(&self, field: &FieldKey) -> bool {
        self.executable_bindings.iter().any(|binding| {
            binding.targets_entity_scalar_field(field) || binding.targets_entity_struct_field(field)
        })
    }
}

impl worth_foundational::facade::PortableAspectContractLookup for LoweredAspectContractPlan {
    fn contract_for(&self, key: &AspectKey) -> Option<AspectContract> {
        self.contract_for(key)
    }
}
