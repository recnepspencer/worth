//! The original installed callback used by source-recovery authority courts.
use crate::durable_medium::TestDurableMedium;
use std::sync::{Arc, Mutex};
use worth_relational::facade::{
    identity::{KindId, VersionId},
    runtime::*,
    schema::RelationalSchemaRegistry,
};
struct ObservedInitialInvariant {
    observed: Arc<Mutex<Vec<(VersionId, VersionId)>>>,
}
impl CustomInvariantRule for ObservedInitialInvariant {
    type Scope = ();
    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new("ordinary.original-enforcement-version"),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("One actual planned entity"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: None,
                access: CustomInvariantAccessContract {
                    read_entity_kinds: vec![KindId(1)],
                    read_relation_kinds: Vec::new(),
                    affected_entity_kinds: vec![KindId(1)],
                    affected_relation_kinds: Vec::new(),
                    include_relation_endpoint_entity_touches: false,
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
        context: &CustomInvariantExecutionContext<'_>,
        _: &(),
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        self.observed
            .lock()
            .unwrap()
            .push((context.version_id(), context.current_version_id()));
        Ok(if context.touched().planned_entity_creates().len() == 1 {
            CustomInvariantVerdict::Pass
        } else {
            CustomInvariantVerdict::Violation(CustomInvariantRefusal::Empty)
        })
    }
}

pub fn installed_runtime(
    medium: &Arc<TestDurableMedium>,
    schema: RelationalSchemaRegistry,
    observed: Arc<Mutex<Vec<(VersionId, VersionId)>>>,
) -> (
    RelationalRuntime,
    RelationalInitialSchemaInstallationReceipt,
) {
    let mut runtime = RelationalRuntimeApi::builder()
        .profile(worth_relational::facade::config::RelationalRuntimeProfile::CertificationCore)
        .build_durable(medium.attachment())
        .unwrap();
    let receipt = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install_with_custom_invariants(
            schema,
            vec![CustomInvariantRegistration::new(ObservedInitialInvariant { observed }).unwrap()],
        )
        .unwrap();
    (runtime, receipt)
}
