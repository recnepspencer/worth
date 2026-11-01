//! One additional ordinary Native commit, after all Directory seeds and indexes.
use super::super::super::bootstrap_publication::{
    map_bootstrap_basis_denial, map_bootstrap_commit_denial, map_bootstrap_staging_denial,
    map_bootstrap_transaction_admission_denial,
};
use super::super::{WorthQueryPrimaryGraphBootstrap, WorthQueryPrimaryGraphPublication};
use super::{
    denial, record::BootstrapPublicationRecord, WorthQueryPrimaryGraphInstallationDenial as Denial,
};
use worth_foundational::facade::ApplicationHomeBinding;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::{
    identity::PartitionId,
    symbols::ClientKey,
    transactions::{AspectFieldPatch, CreateIntent, EntitySpec, MutationIntent, WorkerIntentBatch},
};

pub(in crate::domain_computation::primary_graph) fn publish<Schema: ApplicationSchema>(
    bootstrap: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
    phase: &super::super::super::WorthQueryBootstrapAdvancementPhase<'_>,
    binding: impl FnOnce() -> Result<ApplicationHomeBinding, Denial>,
) -> Result<WorthQueryPrimaryGraphPublication, Denial> {
    let publication = bootstrap.publish_home_bootstrap_rows(phase)?;
    // This existing failure fact precedes every fallible completion operation.
    // Accepted seeds remain retained; neither no-effect nor uncertainty permits replay.
    bootstrap.seed_batch_failed = true;
    let execution = phase.execution_request();
    let record = BootstrapPublicationRecord::prepare(&binding()?, &publication, &execution)?;
    let layout = &bootstrap.graph.layout.bootstrap_publication;
    bootstrap
        .graph
        .integration_handle()
        .with_runtime(&execution, |native| {
            let branch = native.main_branch_identity();
            let basis = native
                .admit_branch_basis(&branch)
                .map_err(map_bootstrap_basis_denial)?;
            let parent = native
                .history()
                .branch_head_for_observation(&basis.observation())
                .map_err(map_bootstrap_basis_denial)?;
            if parent.as_ref().map(|receipt| receipt.commit_id)
                != Some(publication.bootstrap_commit_id())
            {
                return Err(denial(
                    "Home completion parent is not the actual terminal bootstrap receipt",
                ));
            }
            let mut transaction = native
                .begin_branch_transaction(
                    &basis,
                    worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
                )
                .map_err(map_bootstrap_transaction_admission_denial)?;
            let BootstrapPublicationRecord { values, funding } = record;
            transaction
                .push_batch(
                    WorkerIntentBatch::new("application-home-bootstrap-completion").push(
                        MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                            partition_id: PartitionId::main(),
                            kind_id: layout.kind,
                            client_key: ClientKey::raw("worth-query-bootstrap-publication"),
                            fields: AspectFieldPatch::from(
                                layout
                                    .fields
                                    .iter()
                                    .cloned()
                                    .zip(values)
                                    .collect::<std::collections::BTreeMap<_, _>>(),
                            ),
                        })),
                    ),
                    execution.allocation_policy(),
                )
                .map_err(map_bootstrap_staging_denial)?;
            let committed = transaction
                .commit(&execution, native)
                .map_err(map_bootstrap_commit_denial)?;
            let exact_parent =
                committed.commit.parents.as_slice() == [publication.bootstrap_commit_id()];
            crate::relational_snapshot_release::release_query_snapshot(native, &committed.snapshot);
            drop(funding);
            if !exact_parent {
                return Err(denial(
                    "Home completion receipt changed its terminal bootstrap parent",
                ));
            }
            Ok(())
        })?;
    Ok(publication)
}
