//! The first Relational transaction of a program-hosted installation.
//!
//! The branch program activation record is created before any ordinary
//! bootstrap row, because the invariant adapters lowered for this installation
//! read it to decide whose rules speak for a candidate. Seeding it first means
//! the ordinary bootstrap rows are themselves validated under the initial
//! program's declared rules, with no window in which activation is unreadable.

use std::collections::BTreeMap;

use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    AspectFieldPatch, CreateIntent, CreatedEntityRef, EntitySpec, MutationIntent, WorkerIntentBatch,
};

use super::super::bootstrap_publication::{
    map_bootstrap_basis_denial, map_bootstrap_commit_denial, map_bootstrap_staging_denial,
    map_bootstrap_transaction_admission_denial,
};
use super::super::program_occurrence::{
    program_revision_rendering, WorthQueryProgramActivationCell,
};
use super::super::{
    WorthQueryPrimaryGraph, WorthQueryPrimaryGraphInstallationDenial,
    WorthQueryPrimaryGraphInstallationDenialKind,
};

const ACTIVATION_CLIENT_KEY: &str = "worth-query-program-activation";

/// The initial program one installation activates, carried from program
/// admission to the bootstrap transaction that publishes it.
pub(in crate::domain_computation::primary_graph) struct WorthQueryProgramActivationSeed {
    rendering: AspectValue,
    cell: WorthQueryProgramActivationCell,
}

impl WorthQueryProgramActivationSeed {
    pub(in crate::domain_computation::primary_graph) fn for_initial_program(
        revision: &ApplicationProgramRevision,
        cell: WorthQueryProgramActivationCell,
    ) -> Self {
        Self {
            rendering: program_revision_rendering(revision),
            cell,
        }
    }
}

pub(super) fn commit_initial_program_activation(
    graph: &WorthQueryPrimaryGraph,
    seed: WorthQueryProgramActivationSeed,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let layout = graph.layout.program_activation().clone();
    let created = CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: layout.entity_kind,
        client_key: ClientKey::raw(ACTIVATION_CLIENT_KEY),
    };
    let identity = graph.integration_handle().with_runtime_mut(|runtime| {
        let main_identity = runtime.main_branch_identity();
        let options = runtime
            .admit_branch_basis(&main_identity)
            .map_err(map_bootstrap_basis_denial)?;
        let mut transaction = runtime
            .begin_branch_transaction(
                &options,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .map_err(map_bootstrap_transaction_admission_denial)?;
        transaction
            .push_batch(
                WorkerIntentBatch::new("application-program-activation").push(
                    MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                        partition_id: created.partition_id,
                        kind_id: created.kind_id,
                        client_key: created.client_key.clone(),
                        fields: AspectFieldPatch::from(BTreeMap::from([(
                            layout.program_revision_locator.clone(),
                            seed.rendering.clone(),
                        )])),
                    })),
                ),
            )
            .map_err(map_bootstrap_staging_denial)?;
        let committed = transaction
            .commit(runtime)
            .map_err(map_bootstrap_commit_denial)?;
        let identity = committed.created_entity(&created);
        crate::relational_snapshot_release::release_query_snapshot(runtime, &committed.snapshot);
        identity.ok_or_else(|| {
            activation_denial("Relational bound no identity to the program activation record")
        })
    })?;
    seed.cell.publish(identity).map_err(|published| {
        activation_denial(format!(
            "program activation was already published as {}:{}:{}",
            published.partition_value(),
            published.local_slot_value(),
            published.generation_value()
        ))
    })
}

fn activation_denial(subject: impl Into<String>) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(
        WorthQueryPrimaryGraphInstallationDenialKind::RelationalCommitRejected,
        subject,
    )
}
