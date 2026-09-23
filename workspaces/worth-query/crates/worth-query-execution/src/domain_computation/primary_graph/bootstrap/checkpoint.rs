use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledApplicationSchema};
use worth_relational::facade::replay::CanonicalCommitAuthorityKind;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::transactions::{CreateIntent, EntityReference, MutationIntent};

use crate::domain_computation::execution_runtime::{
    product_world::WorthQueryProductWorldResources, WorthQueryExecutionInstallationAuthority,
    WorthQueryExecutionRuntime,
};

use super::super::{
    application_checkpoint::DecodedApplicationCheckpoint, WorthQueryApplicationInvariantFactories,
    WorthQueryPrimaryGraph, WorthQueryPrimaryGraphBootstrap,
    WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
};
use super::WorthQueryPrimaryGraphPublication;

pub(super) fn primary_graph_for_installation(
    runtime_authority: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    binding_identity: ApplicationSchemaBindingIdentity,
    layout: super::super::schema_layout::WorthQueryPrimaryGraphLayout,
    relational_runtime: RelationalRuntime,
    recovered: bool,
) -> Result<WorthQueryPrimaryGraph, WorthQueryPrimaryGraphInstallationDenial> {
    if !recovered {
        return Ok(WorthQueryPrimaryGraph::new(
            runtime_authority,
            binding_identity,
            layout,
            relational_runtime,
        ));
    }
    WorthQueryPrimaryGraph::from_recovered_runtime(
        runtime_authority,
        binding_identity,
        layout,
        relational_runtime,
    )
    .map_err(checkpoint_denial)
}

impl DecodedApplicationCheckpoint {
    pub(super) fn recover_publication(
        &self,
        graph: &WorthQueryPrimaryGraph,
    ) -> Result<WorthQueryPrimaryGraphPublication, WorthQueryPrimaryGraphInstallationDenial> {
        let envelope = graph.integration_handle().with_runtime(|runtime| {
            runtime
                .replay()
                .canonical_commit_envelope(self.bootstrap_commit_id)
        });
        let envelope = envelope
            .ok_or_else(|| checkpoint_denial("checkpoint omitted its claimed bootstrap commit"))?;
        if envelope.branch_context != super::super::primary_relational_branch_id()
            || envelope.authority_kind() != CanonicalCommitAuthorityKind::VersionedTransaction
        {
            return Err(checkpoint_denial(
                "checkpoint bootstrap commit has incompatible authority",
            ));
        }
        let (principal_binding_count, policy_entity_count, policy_relation_count) =
            publication_row_counts(graph, &envelope.merged_plan.merged_intents)?;
        let principal_index_ids = graph
            .layout
            .principal_bindings()
            .map(|(_, binding)| binding.index_id)
            .collect::<std::collections::BTreeSet<_>>();
        let equality_index_ids = graph
            .layout
            .equality_index_ids()
            .collect::<std::collections::BTreeSet<_>>();
        verify_bootstrap_indexes(
            graph,
            principal_index_ids
                .iter()
                .chain(equality_index_ids.iter())
                .copied(),
        )?;
        Ok(WorthQueryPrimaryGraphPublication {
            binding_identity: graph.binding_identity().clone(),
            principal_binding_count,
            identity_index_count: principal_index_ids.len(),
            application_equality_index_count: equality_index_ids.len(),
            policy_entity_count,
            policy_relation_count,
            bootstrap_commit_id: self.bootstrap_commit_id,
        })
    }
}

fn publication_row_counts(
    graph: &WorthQueryPrimaryGraph,
    intents: &[MutationIntent],
) -> Result<(usize, usize, usize), WorthQueryPrimaryGraphInstallationDenial> {
    let mut entities = Vec::new();
    let mut relations = Vec::new();
    for intent in intents {
        match intent {
            MutationIntent::Create(CreateIntent::Entity(entity)) => entities.push(entity),
            MutationIntent::Create(CreateIntent::Relation(relation)) => relations.push(relation),
            _ => {
                return Err(checkpoint_denial(
                    "checkpoint bootstrap commit contains non-bootstrap mutation intent",
                ))
            }
        }
    }
    let mut principal_entities = std::collections::BTreeSet::new();
    let principal_binding_count = relations
        .iter()
        .filter(|relation| {
            graph.layout.principal_bindings().any(|(_, layout)| {
                let EntityReference::Created(source) = &relation.source else {
                    return false;
                };
                let EntityReference::Created(target) = &relation.target else {
                    return false;
                };
                if layout.mapping_kind != source.kind_id
                    || layout.principal_kind != target.kind_id
                    || layout.relation_kind != relation.kind_id
                    || !entities
                        .iter()
                        .any(|entity| created_reference_matches(&relation.source, entity))
                    || !entities
                        .iter()
                        .any(|entity| created_reference_matches(&relation.target, entity))
                {
                    return false;
                }
                principal_entities.insert((
                    source.partition_id,
                    source.kind_id,
                    source.client_key.clone(),
                ));
                principal_entities.insert((
                    target.partition_id,
                    target.kind_id,
                    target.client_key.clone(),
                ));
                true
            })
        })
        .count();
    if principal_binding_count == 0 {
        return Err(checkpoint_denial(
            "checkpoint bootstrap commit contains no principal binding",
        ));
    }
    if principal_entities.len() != principal_binding_count.saturating_mul(2) {
        return Err(checkpoint_denial(
            "checkpoint bootstrap reuses principal row endpoints",
        ));
    }
    Ok((
        principal_binding_count,
        entities.len() - principal_entities.len(),
        relations.len() - principal_binding_count,
    ))
}

fn created_reference_matches(
    reference: &EntityReference,
    entity: &worth_relational::facade::transactions::EntitySpec,
) -> bool {
    matches!(reference, EntityReference::Created(created)
        if created.partition_id == entity.partition_id
            && created.kind_id == entity.kind_id
            && created.client_key == entity.client_key)
}

fn verify_bootstrap_indexes(
    graph: &WorthQueryPrimaryGraph,
    index_ids: impl Iterator<Item = worth_relational::facade::indexes::DerivedIndexId>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    graph.integration_handle().with_runtime(|runtime| {
        let indexes = runtime.index_access();
        for index_id in index_ids {
            let generation =
                indexes.latest_generation(index_id, &super::super::primary_relational_branch_id());
            if generation.is_none() {
                return Err(checkpoint_denial(
                    "checkpoint omitted a bootstrap identity-index generation",
                ));
            }
        }
        Ok(())
    })
}

fn checkpoint_denial(subject: impl Into<String>) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(
        WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected,
        subject,
    )
}

impl WorthQueryExecutionInstallationAuthority {
    pub(in crate::domain_computation::primary_graph) fn prepare_primary_graph_from_native_checkpoint_with_invariants<
        Schema,
    >(
        &self,
        runtime: &WorthQueryExecutionRuntime,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        relational_runtime: RelationalRuntime,
        product_world_resources: WorthQueryProductWorldResources,
        invariant_factories: WorthQueryApplicationInvariantFactories<Schema>,
        checkpoint: &DecodedApplicationCheckpoint,
    ) -> Result<WorthQueryPrimaryGraphBootstrap<Schema>, WorthQueryPrimaryGraphInstallationDenial>
    where
        Schema: ApplicationSchema,
    {
        self.prepare_primary_graph_with_optional_checkpoint(
            runtime,
            installed_schema,
            relational_runtime,
            product_world_resources,
            invariant_factories,
            Some(checkpoint),
        )
    }
}
