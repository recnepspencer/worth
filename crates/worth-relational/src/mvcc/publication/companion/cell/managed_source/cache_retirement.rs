//! Cache cleanup protocol only; public Source semantics have their own court.
use super::*;
use crate::mvcc::{
    PreparedPublicationCompanionEffect, PublicationCompanionPreflight,
    PublicationCompanionRegistrationPort, RelationalPublicationCompanion,
};
use crate::tests::support::{create_entity, managed_cache_schema, snapshot_for_owner_identity};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Mutex,
};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
struct Probe {
    core: Arc<Mutex<Weak<CompanionBranchCellCore>>>,
    port: PublicationCompanionRegistrationPort,
    drops: Arc<AtomicUsize>,
}
impl Drop for Probe {
    fn drop(&mut self) {
        let core = self.core.lock().unwrap().upgrade().unwrap();
        assert!(
            core.current.try_lock().is_ok(),
            "opaque Drop is outside current-image coordination"
        );
        let _new_registration = self
            .port
            .begin_required_registration()
            .expect("opaque Drop is outside the completed registration epoch");
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}
struct Owner {
    source: ManagedIndexSourceProgram,
    scope: EntityId,
    core: Arc<Mutex<Weak<CompanionBranchCellCore>>>,
    port: PublicationCompanionRegistrationPort,
    drops: Arc<AtomicUsize>,
    slots: Mutex<Vec<super::super::CompanionBranchCellSlot<ManagedIndexBranchImage>>>,
}
impl std::fmt::Debug for Owner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CacheRetirementProtocolOwner")
    }
}
impl RelationalPublicationCompanion for Owner {
    fn prepare(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop> {
        let mark = context.mint_selected_branch_cell(Arc::new(()))?;
        let mark = mark.reserve_preflight(context)?;
        let source = context.mint_managed_index_source_cell()?;
        self.slots.lock().unwrap().push(source.lookup_slot());
        let source = source.reserve_preflight(context)?;
        // This owner-local protocol fixture installs opaque cache data at a
        // real publication. It claims no projected-row/invariant READ proof.
        let payload: Arc<dyn Any + Send + Sync> = Arc::new(Probe {
            core: Arc::clone(&self.core),
            port: self.port.clone(),
            drops: Arc::clone(&self.drops),
        });
        let execution = context.execution_request();
        let mut image = ManagedIndexBranchImage::default();
        image.prepare_source(
            &execution,
            &self.source,
            self.scope,
            ManagedIndexSourceAvailability::Available,
            context.canonical_commit().commit.commit_id,
            true,
            Some(&payload),
        )?;
        let image = ManagedIndexBranchImage::retain_header(&execution, image)?;
        context.seal_managed_source_replacement(mark, Arc::new(()), source, image)
    }
}
#[test]
fn managed_cache_retirement_defers_last_opaque_drop_beyond_native_guards() {
    let (schema, program) = managed_cache_schema();
    let runtime = crate::facade::runtime::RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .build()
        .unwrap();
    let startup = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&startup);
    let scope = create_entity(&execution, &runtime, "cache-scope");
    let port = runtime.publication_companion_port();
    let core = Arc::new(Mutex::new(Weak::new()));
    let drops = Arc::new(AtomicUsize::new(0));
    let owner = Arc::new(Owner {
        source: program.clone(),
        scope,
        core: core.clone(),
        port: port.clone(),
        drops: drops.clone(),
        slots: Mutex::new(Vec::new()),
    });
    let registration = port
        .begin_required_registration()
        .unwrap()
        .activate(owner.clone())
        .unwrap();
    create_entity(&execution, &runtime, "real-publication");
    let cell = owner.slots.lock().unwrap()[0].admitted().unwrap();
    *core.lock().unwrap() = Arc::downgrade(&cell.core);
    let snapshot = snapshot_for_owner_identity(&runtime, &runtime.main_branch_identity());
    let retired = runtime
        .read_truth()
        .evict_managed_index_source(&execution, &snapshot, &program, scope)
        .unwrap();
    assert_eq!(
        drops.load(Ordering::SeqCst),
        0,
        "retirement carrier owns the last opaque share"
    );
    assert!(matches!(
        runtime
            .read_truth()
            .observe_managed_index_source(&execution, &snapshot, &program, scope),
        Err(Denial::RootUnavailable)
    ));
    drop(retired);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    drop((registration, owner, cell, runtime));
}
