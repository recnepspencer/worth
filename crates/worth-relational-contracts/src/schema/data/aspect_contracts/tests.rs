use super::{
    AspectBinding, AspectContractPlanRevision, LoweredAspectContractBinding,
    LoweredAspectContractPlan,
};
use crate::identity::data::KindId;
use worth_foundational::FieldKey;

#[test]
fn lowered_plan_admits_only_lowered_entity_scalar_fields() {
    let lowered = LoweredAspectContractPlan {
        kind_id: KindId(1),
        plan_revision: AspectContractPlanRevision(1),
        executable_bindings: smallvec::smallvec![
            LoweredAspectContractBinding {
                contract: worth_foundational::AspectContract::scalar(
                    worth_foundational::AspectKey::new("name").expect("valid key"),
                    worth_foundational::AspectIdentity(1),
                    worth_foundational::AspectContractRevision(1),
                    worth_foundational::ScalarAspectType::String,
                ),
                target: AspectBinding::EntityField {
                    field: FieldKey::new("name").expect("valid field"),
                },
            },
            LoweredAspectContractBinding {
                contract: worth_foundational::AspectContract::scalar(
                    worth_foundational::AspectKey::new("lifecycle").expect("valid key"),
                    worth_foundational::AspectIdentity(2),
                    worth_foundational::AspectContractRevision(1),
                    worth_foundational::ScalarAspectType::String,
                ),
                target: AspectBinding::LifecycleTransition,
            }
        ],
    };

    assert!(lowered.admits_entity_scalar_field(&FieldKey::new("name").expect("valid field")));
    assert!(!lowered.admits_entity_scalar_field(&FieldKey::new("lifecycle").expect("valid field")));
    assert!(!lowered.admits_entity_scalar_field(&FieldKey::new("replicas").expect("valid field")));
}
