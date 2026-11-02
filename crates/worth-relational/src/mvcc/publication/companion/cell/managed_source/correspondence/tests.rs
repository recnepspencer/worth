//! Receiver protocol only. The public court proves actual correspondence issuance.
use super::*;
use crate::mvcc::{
    PreparedPublicationCompanionEffect, PublicationCompanionPreflight,
    RelationalPublicationCompanion,
};
use crate::tests::support::{create_entity, managed_cache_schema, snapshot_for_owner_identity};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};

struct Owner {
    program: ManagedIndexSourceProgram,
    scope: EntityId,
    payload: Arc<dyn Any + Send + Sync>,
    available: std::sync::atomic::AtomicBool,
    slots: std::sync::Mutex<Vec<crate::mvcc::CompanionBranchCellSlot<ManagedIndexBranchImage>>>,
}
impl std::fmt::Debug for Owner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CorrespondenceReceiverProtocolOwner")
    }
}
impl RelationalPublicationCompanion for Owner {
    fn prepare(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop> {
        let mark = context
            .mint_selected_branch_cell(Arc::new(()))?
            .reserve_preflight(context)?;
        let source = context.mint_managed_index_source_cell()?;
        self.slots.lock().unwrap().push(source.lookup_slot());
        let source = source.reserve_preflight(context)?;
        let execution = context.execution_request();
        let mut image = ManagedIndexBranchImage::default();
        let availability = if self.available.load(std::sync::atomic::Ordering::SeqCst) {
            ManagedIndexSourceAvailability::Available
        } else {
            ManagedIndexSourceAvailability::Unavailable
        };
        image.prepare_source(
            &execution,
            &self.program,
            self.scope,
            availability,
            context.canonical_commit().commit.commit_id,
            true,
            Some(&self.payload),
        )?;
        let image = ManagedIndexBranchImage::retain_header(&execution, image)?;
        context.seal_managed_source_replacement(mark, Arc::new(()), source, image)
    }
}

#[test]
fn managed_source_correspondence_receiver_binds_exact_selected_image_and_unavailable_payload() {
    let (schema, program) = managed_cache_schema();
    let runtime = crate::facade::runtime::RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .build()
        .unwrap();
    let host = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&host);
    let scope = create_entity(&execution, &runtime, "scope");
    let other_scope = create_entity(&execution, &runtime, "other-scope");
    let owner = Arc::new(Owner {
        program: program.clone(),
        scope,
        payload: Arc::new(7u64),
        available: std::sync::atomic::AtomicBool::new(false),
        slots: std::sync::Mutex::new(Vec::new()),
    });
    let registration = runtime
        .publication_companion_port()
        .begin_required_registration()
        .unwrap()
        .activate(owner.clone())
        .unwrap();
    create_entity(&execution, &runtime, "unavailable-cache-publication");
    let basis = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let image = basis.inner.root.managed_source_image.get().unwrap();
    let snapshot = snapshot_for_owner_identity(&runtime, &runtime.main_branch_identity());
    let selected = runtime.read_truth().positioned_snapshot(&snapshot).unwrap();
    let generation = image.core.upgrade().unwrap().registration_generation;
    let payload = image
        .corresponding_payload(&selected, generation, &program, scope)
        .unwrap()
        .unwrap();
    assert!(
        Arc::ptr_eq(&payload, &owner.payload),
        "receiver shares the exact immutable payload allocation"
    );
    assert!(
        matches!(
            image.observe(selected.clone(), &program, scope, None, generation),
            Err(Denial::RootUnavailable)
        ),
        "logical backing is never a committed physical read"
    );
    let (_, foreign_program) = managed_cache_schema();
    assert!(matches!(
        image.corresponding_payload(&selected, generation, &foreign_program, scope),
        Err(Denial::ForeignSource)
    ));
    assert!(image
        .corresponding_payload(&selected, generation, &program, other_scope)
        .unwrap()
        .is_none());
    assert!(matches!(
        image.corresponding_payload(&selected, generation + 1, &program, scope),
        Err(Denial::Publication(
            CompanionPreflightStop::RegistrationChanged
        ))
    ));
    for axis in 0..5 {
        let mut foreign = selected.clone();
        match axis {
            0 => foreign.runtime_instance_id += 1,
            1 => foreign.branch_id.0.push_str("-foreign"),
            2 => foreign.root_id += 1,
            3 => foreign.commit_id = None,
            4 => foreign.position = None,
            _ => unreachable!(),
        }
        assert!(
            matches!(
                image.corresponding_payload(&foreign, generation, &program, scope),
                Err(Denial::ForeignSource)
            ),
            "receiver rejects changed selected axis {axis}"
        );
    }
    owner
        .available
        .store(true, std::sync::atomic::Ordering::SeqCst);
    create_entity(&execution, &runtime, "available-cache-publication");
    let resumed = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let resumed_image = resumed.inner.root.managed_source_image.get().unwrap();
    let resumed_snapshot = snapshot_for_owner_identity(&runtime, &runtime.main_branch_identity());
    let resumed_selected = runtime
        .read_truth()
        .positioned_snapshot(&resumed_snapshot)
        .unwrap();
    assert!(
        matches!(
            resumed_image.corresponding_payload(&resumed_selected, generation, &program, scope),
            Err(Denial::ForeignSource)
        ),
        "an available association cannot masquerade as a suspension payload"
    );
    assert!(
        matches!(
            image.corresponding_payload(&resumed_selected, generation, &program, scope),
            Err(Denial::ForeignSource)
        ),
        "an old image cannot supply the successor payload"
    );
    assert!(
        matches!(
            image.evict_selected(&execution, &selected, &program, scope, generation),
            Err(Denial::Publication(
                CompanionPreflightStop::ManagedSourceNotPrepared
            ))
        ),
        "public cache eviction cannot edit an unavailable association"
    );
    drop((payload, registration));
}
