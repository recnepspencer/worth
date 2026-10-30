//! Real selected view gates, not fabricated proof or scalar cache stand-ins.
use super::super::StructuralRelationView;
use crate::derived_relation::{evaluation::SourceRead, *};
use crate::facade::config::{CrossContextPolicy, RelationalRuntimeProfile};
use crate::facade::runtime::RelationalRuntimeApi;
use crate::graph_member::NativeMemberDomain;
use crate::identity::data::KindId;
use crate::schema::data::{
    EndpointKindContractDeclaration, RelationIntegrityDeclarations, RelationalSchemaRegistry,
    SchemaId, SchemaVersionId,
};
use crate::tests::support::{create_entity, create_relation, RelationIntegritySchemaFixture};
use crate::validation::custom_rule::{CustomInvariantInterruption, CustomInvariantWorkMeter};
use crate::validation::data::{
    CustomInvariantAccessContract, StructuralReadError, StructuralWalkDenialKind,
};
use crate::validation::engine::{
    state_view::InvariantStateView, InvariantObservation, InvariantRuntimeView,
};
use std::sync::Arc;

fn catalog() -> (RelationalSchemaRegistry, Vec<DerivedRelationHandle>) {
    let parts = (0..10)
        .map(|index| DerivedRelationDefinitionParts {
            identity: DerivedRelationIdentity {
                schema: SchemaId("test".into()),
                version: SchemaVersionId(1),
                name: format!("admission-{index}"),
            },
            from: NativeMemberDomain::Entity(KindId(1)),
            to: NativeMemberDomain::Entity(KindId(1)),
            nodes: vec![DerivedRelationNode::Stored {
                relation: KindId(2),
                from: KindId(1),
                to: KindId(1),
            }],
            laws: None,
        })
        .collect::<Vec<_>>();
    let identities = parts
        .iter()
        .map(|parts| parts.identity.clone())
        .collect::<Vec<_>>();
    let registry = RelationIntegritySchemaFixture {
        relation_integrity: RelationIntegrityDeclarations::new(
            vec![EndpointKindContractDeclaration {
                contract_id: "admission-endpoints".into(),
                allowed_source_kinds: vec![KindId(1)],
                allowed_target_kinds: vec![KindId(1)],
                self_edges_allowed: true,
                cross_context_policy: CrossContextPolicy::AllowExplicit,
            }],
            vec![],
            vec![],
            vec![],
            vec![],
        ),
        ..Default::default()
    }
    .build_registry()
    .register_derived_relations(parts)
    .unwrap();
    let handles = identities
        .iter()
        .map(|id| registry.derived_relation(id).unwrap())
        .collect();
    (registry, handles)
}
fn new_runtime(registry: RelationalSchemaRegistry) -> crate::runtime::RelationalRuntime {
    RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(registry)
        .build()
        .unwrap()
}
fn access() -> CustomInvariantAccessContract {
    CustomInvariantAccessContract {
        read_entity_kinds: vec![KindId(1)],
        read_relation_kinds: vec![KindId(2)],
        ..Default::default()
    }
}

#[test]
fn derived_admission_keeps_exact_registry_audience_rebind_and_liveness() {
    let cancellation = worth_execution::CancellationSource::new();
    let host =
        crate::test_execution_policy::serial_request().with_cancellation(cancellation.token());
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let (registry, handles) = catalog();
    let runtime = new_runtime(registry);
    let source = create_entity(&execution, &runtime, "source");
    let target = create_entity(&execution, &runtime, "target");
    create_relation(&execution, &runtime, source, target, "edge");
    let observation = InvariantObservation::committed(runtime.storage_access().current_edition());
    let context = InvariantRuntimeView::from_runtime(&runtime);
    let work = CustomInvariantWorkMeter::new(None);
    let view = |access| {
        StructuralRelationView::new(
            InvariantStateView::new(
                observation.committed_partition_access(),
                runtime.current_version_id(),
            ),
            execution,
            work.clone(),
            Arc::new(access),
        )
        .with_structural_source(&context, &observation, true)
    };
    let full = view(access());
    full.require_derived_relation(&handles[0]).unwrap();
    let before = work.consumed().get();
    full.clone().require_derived_relation(&handles[0]).unwrap();
    assert_eq!(
        work.consumed().get() - before,
        8,
        "real same-view hit pays bounded pointer work"
    );
    let prepared = full
        .prepare_derived_relations_from(&handles[0], source)
        .unwrap();
    assert_eq!(
        full.read_prepared_derived_relations(&prepared)
            .unwrap()
            .endpoints(),
        &[target.into()]
    );

    let (foreign_registry, foreign_handles) = catalog();
    assert_eq!(handles[0].parts(), foreign_handles[0].parts());
    assert_ne!(handles[0], foreign_handles[0]);
    assert!(matches!(
        full.require_derived_relation(&foreign_handles[0]),
        Err(DerivedRelationReadDenial::Definition(
            DerivedRelationDenial::ForeignDefinition
        ))
    ));
    let mut read = StructuralSource {
        view: &full,
        reads: DerivedRelationReads::default(),
    };
    assert!(
        matches!(
            read.admit_definition(&foreign_registry, &handles[0]),
            Err(DerivedRelationReadDenial::Definition(
                DerivedRelationDenial::ForeignDefinition
            ))
        ),
        "a warm view cannot admit the evaluator's different registry"
    );
    let actual_registry = context.selected_registry.registry();
    let before = work.consumed().get();
    read.admit_definition(actual_registry, &handles[0]).unwrap();
    assert_eq!(work.consumed().get() - before, 8);

    let narrow = view(CustomInvariantAccessContract {
        read_entity_kinds: vec![KindId(1)],
        ..Default::default()
    });
    assert!(matches!(
        narrow.read_prepared_derived_relations(&prepared),
        Err(DerivedRelationReadDenial::Structural(
            StructuralWalkDenialKind::Read(StructuralReadError::OutsideDeclaredAccess)
        ))
    ));
    let foreign_runtime = new_runtime(foreign_registry);
    let foreign_context = InvariantRuntimeView::from_runtime(&foreign_runtime);
    let rebound = full
        .clone()
        .with_structural_source(&foreign_context, &observation, true);
    assert!(
        matches!(
            rebound.require_derived_relation(&handles[0]),
            Err(DerivedRelationReadDenial::Definition(
                DerivedRelationDenial::ForeignDefinition
            ))
        ),
        "rebinding a genuine view must discard its previous registry admission"
    );
    cancellation.cancel();
    assert!(matches!(
        full.require_derived_relation(&handles[0]),
        Err(DerivedRelationReadDenial::Structural(
            StructuralWalkDenialKind::Interrupted(CustomInvariantInterruption::Cancelled)
        ))
    ));
}

#[test]
fn derived_admission_full_inline_cache_preserves_ordinary_admission() {
    let host = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let (registry, handles) = catalog();
    let runtime = new_runtime(registry);
    let observation = InvariantObservation::committed(runtime.storage_access().current_edition());
    let context = InvariantRuntimeView::from_runtime(&runtime);
    let work = CustomInvariantWorkMeter::new(None);
    let view = StructuralRelationView::new(
        InvariantStateView::new(
            observation.committed_partition_access(),
            runtime.current_version_id(),
        ),
        execution,
        work.clone(),
        Arc::new(access()),
    )
    .with_structural_source(&context, &observation, true);
    for handle in &handles {
        view.require_derived_relation(handle).unwrap();
    }
    let before = work.consumed().get();
    view.require_derived_relation(handles.last().unwrap())
        .unwrap();
    assert!(
        work.consumed().get() - before > 8,
        "a full acceleration cache falls back rather than refusing"
    );
}
