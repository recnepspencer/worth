use crate::identity::data::KindId;

use crate::schema::data::{
    AspectContractPlanRevision, DeclaredAspectContractBinding, LoweredAspectContractBinding,
};

pub(super) fn lower_binding(
    _kind_id: KindId,
    _plan_revision: AspectContractPlanRevision,
    aspect: &DeclaredAspectContractBinding,
) -> LoweredAspectContractBinding {
    LoweredAspectContractBinding {
        contract: aspect.contract.clone(),
        target: aspect.binding.clone(),
    }
}
