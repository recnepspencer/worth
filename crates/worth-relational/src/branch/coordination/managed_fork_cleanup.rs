//! Native fork protocol evidence; public geometry and Query guard have separate courts.
use super::*;
use crate::mvcc::{
    CompanionBranchCellSlot, CompanionPreflightStop, ManagedIndexSourceForkPreparation,
    PreparedPublicationCompanionEffect, PublicationCompanionPreflight,
    RelationalPublicationCompanion,
};
use crate::tests::support::{
    attached_test_medium, create_entity, durable_runtime_with_test_schema, ScriptedPublication,
};
use crate::validation::candidate_source::ManagedIndexBranchImage;
use std::collections::BTreeMap;
use std::sync::atomic::AtomicUsize;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};

type Slot = CompanionBranchCellSlot<ManagedIndexBranchImage>;
struct Owner {
    coordination: Arc<RelationalBranchCoordinationCell>,
    published: Mutex<Vec<Slot>>,
    targets: Arc<Mutex<BTreeMap<BranchId, Slot>>>,
    drops: Arc<AtomicUsize>,
}
impl std::fmt::Debug for Owner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ManagedForkOwnerCourt")
    }
}
struct Lookup {
    coordination: Arc<RelationalBranchCoordinationCell>,
    targets: Arc<Mutex<BTreeMap<BranchId, Slot>>>,
    branch: BranchId,
    slot: Slot,
    drops: Arc<AtomicUsize>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl Drop for Lookup {
    fn drop(&mut self) {
        assert!(
            self.coordination.gate.try_lock().is_ok(),
            "opaque lookup cleanup runs outside source coordination"
        );
        if self.slot.admitted().is_none() {
            let mut targets = self.targets.lock().unwrap();
            let retired = targets.remove(&self.branch);
            let empty = targets.is_empty().then(|| std::mem::take(&mut *targets));
            drop(targets);
            drop(empty);
            drop(retired);
        }
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}
impl RelationalPublicationCompanion for Owner {
    fn prepare_managed_source_fork(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        target: &ManagedIndexSourceForkPreparation<'_>,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>, CompanionPreflightStop> {
        let memory = execution
            .reserve_retained_memory(4096)
            .map_err(|cause| CompanionPreflightStop::Execution(cause.into()))?;
        let slot = target.lookup_slot();
        assert!(slot.admitted().is_none());
        self.targets
            .lock()
            .unwrap()
            .insert(target.target_branch().clone(), slot.clone());
        Ok(Box::new(Lookup {
            coordination: Arc::clone(&self.coordination),
            targets: Arc::clone(&self.targets),
            branch: target.target_branch().clone(),
            slot,
            drops: Arc::clone(&self.drops),
            _memory: memory,
        }))
    }
    fn prepare(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop> {
        let mark = context.mint_selected_branch_cell(Arc::new(()))?;
        let mark = mark.reserve_preflight(context)?;
        let source =
            context.mint_selected_branch_cell(Arc::new(ManagedIndexBranchImage::default()))?;
        self.published.lock().unwrap().push(source.lookup_slot());
        let source = source.reserve_preflight(context)?;
        let image = context.prepare_managed_source_image(&source)?;
        context.seal_managed_source_replacement(mark, Arc::new(()), source, image)
    }
}
#[test]
fn managed_fork_refusal_and_success_drop_lookup_outside_native_coordination() {
    let runtime = durable_runtime_with_test_schema();
    let main = BranchId("main".into());
    let port = runtime.fork_port();
    let cell = port.owner.branch_cell(&main).unwrap().publication_cell();
    let targets = Arc::new(Mutex::new(BTreeMap::new()));
    let drops = Arc::new(AtomicUsize::new(0));
    let owner = Arc::new(Owner {
        coordination: Arc::clone(cell.coordination()),
        published: Mutex::new(Vec::new()),
        targets: Arc::clone(&targets),
        drops: Arc::clone(&drops),
    });
    let registration = runtime
        .publication_companion_port()
        .begin_required_registration()
        .unwrap()
        .activate(owner.clone())
        .unwrap();
    let startup = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    create_entity(&ExecutionRequest::serial(&startup), &runtime, "fork-source");
    let memory = SerialMemoryBudget::new(Some(64 * 1024 * 1024));
    let request =
        SerialRequest::from_memory(memory.clone(), CancellationSource::new().token(), None);
    let execution = ExecutionRequest::serial(&request);
    let target = BranchId("managed-refused-fork".into());
    let medium = attached_test_medium(&runtime);
    medium.script_next_transition(ScriptedPublication::ProvenNoEffect);
    let (_, source) = port.observe_fork_source(&main).unwrap();
    let stopped = port.fork_branch(&execution, target.clone(), source);
    assert!(matches!(
        stopped,
        Err(crate::branch::RelationalBranchTransitionStop::Denied(
            crate::branch::RelationalForkDenial::DurablePublicationRefused(
                crate::durability::medium::RelationalDurablePublicationRefusal::ProvenNoEffect { .. }
            )
        ))
    ));
    drop(stopped);
    assert_eq!(medium.offered_transitions().len(), 1);
    assert!(medium.transition_sequences().is_empty());
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    assert!(targets.lock().unwrap().is_empty());
    assert!(runtime.branch_identity(&target).is_err());
    // The real medium records the offered wire even on refusal. Its immutable
    // control admission remains charged until that recorder is released.
    assert!(memory.charged_memory_bytes() > 0);
    let (_, source) = port.observe_fork_source(&main).unwrap();
    port.fork_branch(&execution, target.clone(), source)
        .unwrap();
    assert_eq!(drops.load(Ordering::SeqCst), 2);
    assert!(targets.lock().unwrap()[&target].admitted().is_some());
    assert!(memory.charged_memory_bytes() > 0);
    drop((runtime, port, registration, owner));
    targets.lock().unwrap().clear();
    drop(medium);
    assert_eq!(memory.charged_memory_bytes(), 0);
}
