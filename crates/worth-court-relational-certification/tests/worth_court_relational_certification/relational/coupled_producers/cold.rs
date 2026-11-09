//! Mixed and all-ordinary Native groups retain exactly their real origins on disk.
use super::*;
use std::sync::atomic::{AtomicU64, Ordering};
use worth_relational::facade::{durability::RelationalNativeCheckpoint, schema::*};
use worth_testkit_relational::aspect_storage::AspectSchemaFixture;
static NEXT: AtomicU64 = AtomicU64::new(0);
pub(super) fn reopen(
    world: &World,
    execution: &ExecutionRequest<'_, '_>,
    mixed: bool,
) -> (
    RelationalRuntime,
    RelationalInstalledApplicationPublicationAuthority,
) {
    let checkpoint = world
        .runtime
        .durability_authority()
        .native_checkpoint(execution, execution.allocation_policy())
        .unwrap();
    let path = std::env::temp_dir().join(format!(
        "worth-ordinary-native-{}-{mixed}-{}.checkpoint",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::write(&path, checkpoint.bytes()).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    let fixture = AspectSchemaFixture::with_default_declared_aspects(
        worth_relational::facade::config::CascadeDeletePolicy::RetainDanglingForAudit,
    );
    let registry = register_current_publication_storage(
        fixture.build_registry(),
        &fixture.schema_id,
        fixture.schema_version_id,
    )
    .unwrap();
    let mut runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build()
        .unwrap();
    let receipt = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(RelationalSchemaRegistry::new())
        .unwrap();
    runtime
        .durability_recovery()
        .restore_native_checkpoint(
            execution,
            &RelationalNativeCheckpoint::from_untrusted_bytes(bytes.into_boxed_slice()),
        )
        .expect("the canonical event and current roster must reopen without a format change");
    let receipt = runtime
        .readmit_recovered_initial_schema_installation(receipt)
        .unwrap();
    let (_, installed) = receipt.issue_application_publication_owner();
    (
        runtime,
        installed.with_source_scope_owned_producers([("owned".into(), KindId(1))]),
    )
}
pub(super) fn facts(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    ids: &[EntityId],
) -> Vec<worth_relational::facade::runtime::EntityReadRecord> {
    let basis = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .1;
    let snapshot = runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();
    let read = runtime
        .read_truth()
        .read_snapshot(&snapshot, execution)
        .unwrap()
        .unwrap();
    let rows = ids
        .iter()
        .map(|id| {
            read.get_entity(*id)
                .expect("the same full-generation record is live")
                .clone()
        })
        .collect();
    runtime.snapshots().release_snapshot(&snapshot).unwrap();
    rows
}
#[test]
fn disk_reopen_keeps_ordinary_companions_and_only_the_real_mixed_origin() {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    for mixed in [false, true] {
        let world = World::new(execution, true, true);
        let mut parts = assembly(&world);
        parts.mixed = mixed;
        let proposal = world.prepare(execution, &parts).unwrap();
        let ordinary = proposal
            .coupled_contribution_records("ordinary", &world.meaning("ordinary"))
            .unwrap();
        let companion = ordinary
            .ordinary_output_members()
            .unwrap()
            .find(|(role, ..)| *role == "ordinary-companion")
            .expect("real ordinary allocation")
            .3;
        let mut ids = vec![world.source, world.carrier, companion];
        let result = world.publish(execution, proposal);
        let origins = result
            .required_output_publications(&world.installed, execution)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(origins.len(), usize::from(mixed));
        if mixed {
            ids.push(origins[0].members().next().unwrap().3);
        }
        release_test_commit_snapshot(&world.runtime, &result);
        let expected = facts(&world.runtime, execution, &ids);
        let (cold, installed) = reopen(&world, execution, mixed);
        let actual = facts(&cold, execution, &ids);
        for (a, b) in actual.iter().zip(&expected) {
            assert_eq!(a.authoritative_aspect_state, b.authoritative_aspect_state);
        }
        let basis = cold.observe_branch(&cold.main_branch_identity()).unwrap().1;
        assert!(
            cold.readmit_required_output_route_absence(
                execution,
                &installed,
                &basis,
                &world.meaning("ordinary")
            )
            .unwrap()
            .is_some(),
            "ordinary never gains route presence or an origin"
        );
        assert_eq!(
            cold.readmit_required_output_route_absence(
                execution,
                &installed,
                &basis,
                &world.meaning("owned")
            )
            .unwrap()
            .is_none(),
            mixed,
            "only the real owned route has a position"
        );
    }
}
