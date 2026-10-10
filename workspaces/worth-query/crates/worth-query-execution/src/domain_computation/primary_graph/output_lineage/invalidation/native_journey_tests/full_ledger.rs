//! A full ledger refuses new derived rows, never a legal writer. Source
//! publication evicts to its paid empty image; fresh registration recovers
//! after pinned predecessor custody and the held capacity actually end.
use crate::domain_computation::primary_graph::output_binding_identity::OutputBindingIdentity;

use crate::domain_computation::primary_graph::output_lineage::RetainedSourceFacts;
#[path = "full_ledger/fork_consumption.rs"]
mod fork_consumption;
#[path = "full_ledger/stale_preflight.rs"]
mod stale_preflight;

use super::marking_ceiling::world_installing;
use super::*;
use crate::domain_computation::execution_runtime::WorthQueryInvalidationResourceInstallation;
use crate::domain_computation::primary_graph::output_lineage::invalidation::SettlementRegistrationStop;
use worth_relational::facade::mvcc::CompanionPreflightStop;

/// Small enough that a few rows fill it.
const MAXIMUM_RETAINED_BYTES: u64 = 256 * 1_024;
const WINDOW: usize = 4;

#[test]
fn a_full_ledger_still_publishes_retires_and_registers_again() {
    let world = world_installing(|defaults| WorthQueryInvalidationResourceInstallation {
        maximum_retained_bytes: MAXIMUM_RETAINED_BYTES,
        maximum_retained_positions: WINDOW,
        ..defaults
    });
    let selected = world.selected_product();
    let entity = selected
        .resolve_entity(
            AccountStatus::reference(),
            "open".to_owned(),
            &live_scope(),
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
        .entity_id();
    let (_, product, _) = selected.into_parts();
    let coordinate = ProductCoordinate {
        occurrence: product.observation().lifecycle_incarnation(),
        generation: product.observation().reference_generation().get(),
    };
    let source = SemanticSource {
        runtime_authority: world.application.runtime.authority_identity().as_u64(),
        schema: world.application.installed_schema.binding_identity(),
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(entity),
        output_binding: OutputBindingIdentity::declared("StatusOutput"),
    };
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let status_ref = AccountStatus::reference();
    let status = graph
        .layout()
        .field_locator(status_ref.entity(), status_ref.aspect(), status_ref.field())
        .unwrap()
        .clone();
    let label_ref = AccountLabel::reference();
    let label = graph
        .layout()
        .field_locator(label_ref.entity(), label_ref.aspect(), label_ref.field())
        .unwrap()
        .clone();
    let retained = || owner.resources.retained_capacity_bytes();
    let try_register = |runtime: &RelationalRuntime, slot: usize| {
        let row = RecordedSettlementIdentity::retain(&source, coordinate, slot);
        let (basis_handle, basis) = snapshot(runtime);
        let fact = field_fact(runtime, &basis_handle, entity, status.clone());
        let registered = owner.register_settlement(
            SettlementRegistration {
                work_membership: None,
                identity: Arc::clone(&row),
                facts: RetainedSourceFacts::for_test(false, Arc::from([fact])),
                output_facts: None,
                read_basis: basis,
                stale_at_read_basis: OrdSet::new(),
                requirement: None,
                upstream: OrdSet::new(),
            },
            &mut owner.edit_admission(),
        );
        runtime.snapshots().release_snapshot(&basis_handle).unwrap();
        registered.map(|()| row)
    };

    handle.with_runtime_mut(|runtime| {
        // Each delivery touches the label, which no row reads.
        let mut flip = false;
        let mut deliver = |runtime: &mut RelationalRuntime| {
            flip = !flip;
            write_field(
                runtime,
                entity,
                label.clone(),
                if flip { "aa" } else { "bb" },
            );
            assert!(retained() <= MAXIMUM_RETAINED_BYTES);
        };
        for _ in 0..=WINDOW {
            deliver(runtime);
        }
        assert!(retained() <= MAXIMUM_RETAINED_BYTES);

        let mut rows = Vec::new();
        let stop = loop {
            match try_register(runtime, rows.len()) {
                Ok(row) => rows.push(row),
                Err(stop) => break stop,
            }
        };
        assert!(
            matches!(
                stop,
                SettlementRegistrationStop::Admission(
                    CompanionPreflightStop::RetainedCompanionCapacityExhausted { .. }
                )
            ),
            "only the full ledger refuses a row: {stop:?}"
        );
        assert!(rows.len() > 1, "the ledger holds rows before it is full");
        // What the last row did not need is held too: the ledger is full to
        // the byte, so publication must use the pre-admitted empty image.
        let slack = MAXIMUM_RETAINED_BYTES - retained();
        let held = owner.resources.reserve_retained_capacity(slack).unwrap();
        assert_eq!(retained(), MAXIMUM_RETAINED_BYTES);

        // A legal writer is never refused, though its delivery marks every
        // row and its install replaces the root that holds them; nor are the
        // writes after it, which move the window.
        write_field(runtime, entity, status.clone(), "closed");
        let (discard_handle, discarded_at) = snapshot(runtime);
        for identity in &rows {
            assert!(
                matches!(
                    owner
                        .currentness(&discarded_at, identity, &mut owner.edit_admission())
                        .unwrap(),
                    SourceSettlementCurrentness::FullVerificationRequired(_)
                ),
                "discarded rows must require full comparison"
            );
        }
        runtime
            .snapshots()
            .release_snapshot(&discard_handle)
            .unwrap();
        assert!(retained() <= MAXIMUM_RETAINED_BYTES);
        for _ in 0..=WINDOW {
            deliver(runtime);
        }
        assert_eq!(
            owner.retire_settlements(&rows, &mut owner.edit_admission()),
            rows,
            "retirement frees what it replaces, so a full ledger retires"
        );
        for _ in 0..=WINDOW {
            deliver(runtime);
        }
        drop(held);
        assert_eq!(
            retained(),
            empty_index_custody_bytes(owner),
            "every retained byte has live empty-index custody"
        );
        let again = try_register(runtime, rows.len());
        assert!(again.is_ok(), "the recovered ledger registers a row again");
    });
}

#[test]
fn a_full_ledger_does_not_refuse_the_first_source_write_on_a_new_branch() {
    fork_consumption::restored_root_consumption_and_later_writes();
}

/// Derive the exact live capacity from ticket custody, not an observed footprint.
fn empty_index_custody_bytes(owner: &SourceInvalidationOwner) -> u64 {
    use crate::domain_computation::execution_runtime::source_invalidation::RetainedInvalidationCapacity;
    let mut tickets = BTreeMap::<usize, u64>::new();
    let mut add = |ticket: &Arc<RetainedInvalidationCapacity>| {
        tickets.insert(Arc::as_ptr(ticket) as usize, ticket.bytes());
    };
    let branches = owner.branches.lock().unwrap();
    if let Some(ticket) = &branches.retained_capacity {
        add(ticket);
    }
    let mut roots = vec![Arc::clone(branches.vacant.as_ref().unwrap())];
    roots.extend(
        branches
            .cells
            .values()
            .filter_map(|slot| slot.admitted())
            .map(|cell| Arc::clone(cell.read_image().payload())),
    );
    for root in roots {
        if let Some(history) = &root.history_capacity {
            use super::super::source_alignment::HistoricalMarkState;
            use worth_relational::facade::publication::PatchStreamPosition;
            // Each live history map owns its declared node bound and shared ticket.
            // retention::admit_edited_history funds this before replacing the map.
            use std::mem::{align_of, size_of};
            type Entry = (Option<PatchStreamPosition>, HistoricalMarkState);
            // im 15.1 has 64 entries, 65 child pointers, six usize bounds,
            // four alignment regions, and at least 31 keys per nonroot.
            let alignment = align_of::<Entry>().max(align_of::<usize>());
            let node = 64 * size_of::<Entry>()
                + 65 * size_of::<Option<Arc<()>>>()
                + 6 * size_of::<usize>()
                + 4 * alignment;
            let nodes = 1 + root.past.len().saturating_sub(1) / 31;
            let ticket_alignment =
                align_of::<RetainedInvalidationCapacity>().max(align_of::<usize>());
            let offset = (2 * size_of::<usize>()).div_ceil(ticket_alignment) * ticket_alignment;
            let ticket = (offset + size_of::<RetainedInvalidationCapacity>())
                .div_ceil(ticket_alignment)
                * ticket_alignment;
            let declared = (nodes * node + ticket) as u64;
            assert_eq!(history.bytes(), declared);
            add(history);
        }
        for ticket in [&root.retained_capacity, &root.inherited_capacity]
            .into_iter()
            .flatten()
        {
            add(ticket);
        }
        for state in
            std::iter::once(&root.current).chain(root.past.values().map(|past| &past.state))
        {
            assert!(
                state.settlements.is_empty(),
                "expired settlements retain no byte custody"
            );
            assert!(state.equal_links.is_empty());
            if let Some(ticket) = &state.retained_capacity {
                add(ticket);
            }
        }
        for past in root.past.values() {
            add(&past.next_delivery._capacity);
        }
    }
    tickets.values().sum()
}
