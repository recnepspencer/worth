use worth_foundational::facade::{
    AspectValue, ContractValidationInput, PortableAspectContractBasis,
    PortableAspectPatchOperation, PortableRecordAspectPatch,
};

use crate::identity::data::EntityId;
use crate::schema::data::{AspectBinding, LoweredAspectContractPlan};

pub(crate) fn append_authoritative_endpoints(
    candidate: PortableRecordAspectPatch,
    plan: Option<&LoweredAspectContractPlan>,
    source: EntityId,
    target: EntityId,
) -> PortableRecordAspectPatch {
    let mut operations = candidate.operations().to_vec();
    let Some(plan) = plan else {
        return candidate;
    };
    for binding in &plan.executable_bindings {
        let endpoint = match binding.target {
            AspectBinding::RelationSourceEndpoint => source,
            AspectBinding::RelationTargetEndpoint => target,
            _ => continue,
        };
        operations.push(PortableAspectPatchOperation::SetWhole {
            basis: PortableAspectContractBasis::from_contract(&binding.contract),
            value: ContractValidationInput::Scalar(AspectValue::EntityRef(foundational_id(
                endpoint,
            ))),
        });
    }
    PortableRecordAspectPatch::new(operations)
}

fn foundational_id(entity_id: EntityId) -> worth_foundational::facade::EntityId {
    worth_foundational::facade::EntityId {
        partition_id: worth_foundational::facade::PartitionId(entity_id.partition_id.as_u32()),
        local_slot: worth_foundational::facade::LocalSlot(entity_id.local_slot_value()),
        generation: worth_foundational::facade::Generation(entity_id.generation_value()),
    }
}
