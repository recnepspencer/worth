//! Catalog identity includes executable routing, even with no managed Sources.
use worth_execution::ExecutionRequest;
use worth_relational::facade::runtime::*;
use worth_testkit_relational::initial_input_source;
use worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request;

#[derive(Clone, Copy)]
struct Rule {
    endpoint_touches: bool,
}
impl CustomInvariantRule for Rule {
    type Scope = ();
    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new("catalog.endpoint-touch"),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: "Endpoint touch routing".into(),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: std::num::NonZeroU64::new(100),
                access: CustomInvariantAccessContract {
                    include_relation_endpoint_entity_touches: self.endpoint_touches,
                    ..Default::default()
                },
                execution_point: InvariantExecutionPoint::CommitBoundary,
                groups: InvariantGroupSet::of(InvariantGroup::SchemaCompliance),
                cost_class: InvariantCostClass::Touched,
                failure_effect: InvariantFailureEffect::BlockCommit,
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

#[test]
fn target_catalog_identity_changes_when_relation_endpoint_touch_routing_changes() {
    let request = serial_request();
    let execution = ExecutionRequest::serial(&request);
    let without = CustomInvariantRegistration::new(Rule {
        endpoint_touches: false,
    })
    .unwrap();
    let with = CustomInvariantRegistration::new(Rule {
        endpoint_touches: true,
    })
    .unwrap();
    let (runtime, _) = initial_input_source::initialized(&execution);
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let compile = |rule| {
        runtime
            .prepare_home_program_contracts(
                &execution,
                &basis,
                runtime.config().schema.registry.clone(),
                vec![rule],
                vec![],
            )
            .unwrap()
            .catalog_digest()
    };
    let without = compile(without);
    let with = compile(with);
    assert_ne!(without, with);
}
