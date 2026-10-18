//! Native relation-only invariant ownership remains a pre-publication fence.
use super::{
    super::fixture::{basis, capture, installed_with},
    fixture::*,
};
use crate::identity::data::KindId;
use crate::validation::data::*;

#[test]
fn relation_only_custom_invariant_refuses_closed_retirement_before_effects() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let install = |version| {
        installed_with(
            registry(version),
            vec![CustomInvariantRegistration::new(RelationInvariant).unwrap()],
        )
    };
    let (runtime, receipt) = install(1);
    let world = world_with_installation(execution, runtime, receipt);
    let (mut target, _) = install(2);
    let (_, mut recovered) = target
        .durability_recovery()
        .restore_native_checkpoint_for_codec_transition_with_authority(
            execution,
            &capture(&world.runtime),
        )
        .unwrap();
    let before = capture(&target);
    let selected = basis(&target);
    let denied = candidate(
        execution,
        &mut target,
        &mut recovered,
        world.activation,
        crate::mvcc::RelationalOperationControl::uninterrupted(),
    )
    .unwrap_err();
    assert_eq!(
        denied.kind(),
        crate::runtime::RelationalSchemaTransitionAdmissionDenialKind::UnsupportedCodecConsequence
    );
    assert!(denied.detail().contains("custom invariant"));
    assert!(selected.is_current());
    assert!(recovered.codec_transition_pending());
    assert_eq!(capture(&target), before);
    assert_order(&target, &world.outputs, &world.relations);
}

struct RelationInvariant;
impl CustomInvariantRule for RelationInvariant {
    type Scope = ();
    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new("codec.closed.relation.invariant"),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: "Closed relation invariant".into(),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: std::num::NonZeroU64::new(1024).unwrap(),
                execution_point: InvariantExecutionPoint::CommitBoundary,
                groups: InvariantGroupSet::of(InvariantGroup::SchemaCompliance),
                cost_class: InvariantCostClass::Touched,
                failure_effect: InvariantFailureEffect::BlockCommit,
                access: CustomInvariantAccessContract {
                    read_relation_kinds: vec![KindId(4)],
                    affected_relation_kinds: vec![KindId(4)],
                    include_relation_endpoint_entity_touches: false,
                    ..Default::default()
                },
            },
        }
    }
    fn prepare_scope(
        &self,
        _: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<(), CustomInvariantPreparationError> {
        Ok(())
    }
    fn evaluate(
        &self,
        _: &CustomInvariantExecutionContext<'_>,
        _: &(),
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        Ok(CustomInvariantVerdict::Pass)
    }
}
