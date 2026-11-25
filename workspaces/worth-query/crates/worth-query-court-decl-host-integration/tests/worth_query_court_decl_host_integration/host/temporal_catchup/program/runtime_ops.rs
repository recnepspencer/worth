//! Graph seeding, runtime read/write operations, and temp dirs for desktop temporal courts.

use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

use worth_query_decl::facade::authentication::{
    WorthQueryExternalPrincipalIdentity, WorthQueryPrincipalMappingStatus,
};
use worth_query_host::facade::{
    application_entry::{WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt},
    domain, primary_graph, runtime,
};

use super::ReminderHome;
use crate::host::temporal_catchup::{
    adapters::{admit_local_adapter, fresh_request_scope, MutateReminderIntent},
    schema::{
        due_field, fired_count_field, gate_field, identity_field, interval_field, lifecycle_field,
        max_fires_field, revision_field, DesktopReminderSchema, ExternalMapping,
        MutateReminderInput, Principal, ReminderIntent, ReminderPrincipalBinding, ReminderRow,
        ReminderTarget, ReminderTargetsReadRequest,
    },
};

pub(super) fn seed_reminder_graph(
    graph: &mut primary_graph::WorthQueryPrimaryGraphBootstrap<DesktopReminderSchema>,
    principal_binding: &domain::WorthQueryInstalledPrincipalBinding<
        DesktopReminderSchema,
        ReminderPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        worth_query_decl::facade::application_schema::U64ApplicationValueBinding,
    >,
) {
    graph
        .bind_principal(
            principal_binding,
            primary_graph::WorthQueryApplicationPrincipalKey::new("daymark-user").unwrap(),
            1_u64,
            WorthQueryExternalPrincipalIdentity::new("https://local.desktop", "daymark-user")
                .unwrap(),
            WorthQueryPrincipalMappingStatus::Enabled,
        )
        .unwrap();
    for (row_key, id) in [("row-hub", "hub"), ("row-i1", "i1"), ("row-i2", "i2")] {
        graph
            .bind_entity(
                primary_graph::WorthQueryApplicationEntitySeed::new(
                    ReminderIntent::reference(),
                    primary_graph::WorthQueryApplicationEntityKey::new(row_key).unwrap(),
                )
                .field(identity_field(), id.to_owned())
                .field(revision_field(), 0_u64)
                .field(due_field(), 0_u64)
                .field(interval_field(), 0_u64)
                .field(max_fires_field(), 0_u64)
                .field(fired_count_field(), 0_u64)
                .field(lifecycle_field(), "absent".to_owned())
                .field(gate_field(), "ready".to_owned()),
            )
            .unwrap();
    }
    for (rel_id, target_key) in [("hub-to-i1", "row-i1"), ("hub-to-i2", "row-i2")] {
        graph
            .bind_relation(primary_graph::WorthQueryApplicationRelationSeed::new(
                ReminderTarget::reference(),
                rel_id,
                primary_graph::WorthQueryApplicationEntityKey::new("row-hub").unwrap(),
                primary_graph::WorthQueryApplicationEntityKey::new(target_key).unwrap(),
            ))
            .unwrap();
    }
}

pub fn mutate_reminder(home: &ReminderHome, input: MutateReminderInput, command_key: u64) {
    let _ = mutate_reminder_returning_change(home, input, command_key);
}

pub fn mutate_reminder_returning_change(
    home: &ReminderHome,
    input: MutateReminderInput,
    command_key: u64,
) -> Option<worth_query_host::facade::runtime::WorthQueryPerformedRelationalProductChange> {
    let runtime = home.runtime();
    let scope = fresh_request_scope();
    let adapter = admit_local_adapter(runtime.installed_schema());
    let principal = adapter.authenticate_local(&scope).unwrap();
    let outcome = runtime
        .request(&principal, &scope)
        .advanced()
        .mutate(MutateReminderIntent(input))
        .without_source()
        .idempotency(&command_key)
        .execute_in_program(
            runtime,
            runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("reminder mutation executes");
    match outcome {
        WorthQueryApplicationMutationOutcome::Committed { mut receipt, .. } => {
            receipt.take_performed_relational_product_change()
        }
        other => panic!("expected committed reminder mutation, got {other:?}"),
    }
}

pub fn read_reminders(home: &ReminderHome) -> BTreeMap<String, ReminderRow> {
    let runtime = home.runtime();
    let scope = fresh_request_scope();
    let adapter = admit_local_adapter(runtime.installed_schema());
    let principal = adapter.authenticate_local(&scope).unwrap();
    let result = runtime
        .request(&principal, &scope)
        .query(ReminderTargetsReadRequest {
            hub: "hub".to_owned(),
        })
        .execute()
        .expect("reminder targets query executes");
    result
        .rows()
        .iter()
        .cloned()
        .map(|row| (row.identity.clone(), row))
        .collect()
}

pub fn unique_temp_dir(tag: &str) -> PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "worth-temporal-catchup-{tag}-{}-{seq}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}
