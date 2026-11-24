//! The same untouched custom rules leave real commit work and span unchanged between request postures.
use std::{
    num::{NonZeroU64, NonZeroUsize},
    sync::{Arc, OnceLock},
};
use worth_execution::{
    CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig, ExecutionRequest,
    ExecutionResourceLease, ExecutionWorkCeiling, LeaseRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::{
    identity::{KindId, PartitionId},
    runtime::{
        CustomInvariantAccessContract, CustomInvariantDescriptor, CustomInvariantExecutionContext,
        CustomInvariantExecutionError, CustomInvariantLeaseBudget,
        CustomInvariantOperationalMetadata, CustomInvariantPreparationError,
        CustomInvariantRegistration, CustomInvariantRule, CustomInvariantRuleId,
        CustomInvariantScopePlanner, CustomInvariantSemanticIdentity,
        CustomInvariantSemanticVersion, CustomInvariantVerdict, InvariantCatalog,
        InvariantCostClass, InvariantExecutionPoint, InvariantFailureEffect, InvariantGroup,
        InvariantGroupSet, RelationalRuntime, RelationalRuntimeApi,
    },
    schema::{
        EntityKindRegistration, KindAspectContractDeclarations, RelationalSchemaRegistry, SchemaId,
        SchemaVersionId,
    },
    symbols::ClientKey,
    transactions::{CreateIntent, EntitySpec, MutationIntent, WorkerIntentBatch},
};
use worth_testkit_relational::aspect_storage::publications::begin_main_transaction;

struct TinyRule(u8);
impl CustomInvariantRule for TinyRule {
    type Scope = ();
    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new(format!("narrowing.tiny.{}", self.0)),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("Untouched tiny rule"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: Some(NonZeroU64::new(1).unwrap()),
                access: CustomInvariantAccessContract {
                    read_entity_kinds: vec![KindId(2)],
                    read_relation_kinds: vec![],
                    affected_entity_kinds: vec![KindId(2)],
                    affected_relation_kinds: vec![],
                    include_relation_endpoint_entity_touches: false,
                },
                execution_point: InvariantExecutionPoint::CommitBoundary,
                groups: InvariantGroupSet::of(InvariantGroup::SchemaCompliance),
                cost_class: InvariantCostClass::Touched,
                failure_effect: InvariantFailureEffect::BlockCommit,
            },
        }
    }
    fn supports_checked_preparation(&self) -> bool {
        true
    }
    fn prepare_scope(
        &self,
        _: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<(), CustomInvariantPreparationError> {
        Ok(())
    }
    fn prepare_scope_checked(
        &self,
        _: &mut CustomInvariantScopePlanner<'_>,
        _: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<(), CustomInvariantPreparationError>> {
        Some(Ok(()))
    }
    fn evaluate(
        &self,
        _: &CustomInvariantExecutionContext<'_>,
        _: &(),
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        Ok(CustomInvariantVerdict::Pass)
    }
    fn evaluate_checked(
        &self,
        _: &CustomInvariantExecutionContext<'_>,
        _: &(),
        _: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<CustomInvariantVerdict, CustomInvariantExecutionError>> {
        Some(Ok(CustomInvariantVerdict::Pass))
    }
}
fn runtime() -> RelationalRuntime {
    let mut schema = RelationalSchemaRegistry::new();
    for (id, name) in [(1, "unrelated"), (2, "rule-kind")] {
        schema = schema
            .register_entity_kind(EntityKindRegistration {
                kind_id: KindId(id),
                kind_name: name.into(),
                schema_id: SchemaId("narrowing".into()),
                schema_version_id: SchemaVersionId(1),
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
            })
            .unwrap();
    }
    RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .invariant_catalog(InvariantCatalog {
            registrations: vec![],
        })
        .custom_invariant(CustomInvariantRegistration::new(TinyRule(0)).unwrap())
        .custom_invariant(CustomInvariantRegistration::new(TinyRule(1)).unwrap())
        .build()
        .expect("valid native runtime configuration")
}
fn request(serial: bool) -> ExecutionResourceLease<'static> {
    let request = LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, None, Some(1_000_000)),
        ),
        cancellation: CancellationToken::new(),
        deadline: None,
    };
    if serial {
        ExecutionResourceLease::serial(request).unwrap()
    } else {
        execution_authority().request_lease(request).unwrap()
    }
}
fn intent(kind: u32, key: &str) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: PartitionId::main(),
        kind_id: KindId(kind),
        client_key: ClientKey::raw(key),
        fields: Default::default(),
    }))
}
fn execution_authority() -> &'static ExecutionAuthority {
    static AUTHORITY: OnceLock<ExecutionAuthority> = OnceLock::new();
    AUTHORITY.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(8).unwrap(),
            charged_memory_bytes: Some(512 * 1024 * 1024),
        })
        .unwrap()
    })
}

#[test]
fn untouched_rules_keep_real_commit_work_and_span_equal_at_both_postures() {
    let mut totals = Vec::new();
    for serial in [false, true] {
        let runtime = runtime();
        let lease = request(serial);
        let mut tx = begin_main_transaction(&runtime);
        tx.push_batch(
            WorkerIntentBatch::new("untouched").push(intent(1, "committed")),
            ExecutionRequest::leased(&lease).allocation_policy(),
        )
        .unwrap();
        let (_, report) = ExecutionRequest::leased(&lease)
            .run(ExecutionWorkCeiling::new(Some(1_000_000)), |_| {
                tx.commit(&ExecutionRequest::leased(&lease), &runtime)
                    .expect("untouched tiny rules accept the real commit")
            })
            .unwrap();
        totals.push((report.charged_work(), report.charged_span()));
    }
    assert_eq!(totals[0], totals[1]);
}
