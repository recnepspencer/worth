use super::*;
use crate::runtime::{WorthQueryAspectTouch, WorthQueryAuthoredAspectMutation};
use worth_foundational::facade::AspectValue;
use worth_relational::facade::config::{RelationalConfigOverride, RelationalRuntimeProfile};
use worth_relational::facade::runtime::{
    CustomInvariantRegistration, EntityProjectionRecord, InvariantCatalog, ProjectionAspectScope,
    RelationalRuntimeApi, RelationalRuntimeConfig,
};
use worth_relational::facade::schema::{
    DeclaredAspectContractBinding, EntityKindRegistration, KindAspectContractDeclarations,
    RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};
use worth_relational::facade::transactions::{
    DeleteEntityIntent, EntityMutationIntent, MutationIntent, WorkerIntentBatch,
};

use super::workspace_schema::{inferred_string_declarations, native_contract_declarations};

impl WorthQueryMemoryWorkspace {
    pub fn collection(
        kind_name: impl Into<String>,
        aspects: impl IntoIterator<Item = WorthQueryAspect>,
    ) -> Result<Self, WorthQueryWorkspaceError> {
        Self::collection_with_invariants(kind_name, aspects, InvariantCatalog::default(), [])
    }

    pub(crate) fn collection_with_invariants(
        kind_name: impl Into<String>,
        aspects: impl IntoIterator<Item = WorthQueryAspect>,
        invariant_catalog: InvariantCatalog,
        custom_invariants: impl IntoIterator<Item = CustomInvariantRegistration>,
    ) -> Result<Self, WorthQueryWorkspaceError> {
        let aspects = aspects.into_iter().collect::<Vec<_>>();
        let declarations = inferred_string_declarations(&aspects)?;
        Self::collection_with_declarations(
            kind_name,
            aspects,
            declarations,
            invariant_catalog,
            custom_invariants,
            None,
        )
    }

    pub(crate) fn collection_with_native_contracts_for_initial_seed(
        kind_name: impl Into<String>,
        aspects: impl IntoIterator<Item = WorthQueryAspect>,
        contracts: impl IntoIterator<Item = worth_foundational::facade::AspectContract>,
        invariant_catalog: InvariantCatalog,
        custom_invariants: impl IntoIterator<Item = CustomInvariantRegistration>,
        initial_seed_rows: usize,
    ) -> Result<Self, WorthQueryWorkspaceError> {
        let aspects = aspects.into_iter().collect::<Vec<_>>();
        let declarations = native_contract_declarations(&aspects, contracts)?;
        Self::collection_with_declarations(
            kind_name,
            aspects,
            declarations,
            invariant_catalog,
            custom_invariants,
            Some(initial_seed_rows),
        )
    }

    fn collection_with_declarations(
        kind_name: impl Into<String>,
        aspects: Vec<WorthQueryAspect>,
        declared_aspects: Vec<DeclaredAspectContractBinding>,
        invariant_catalog: InvariantCatalog,
        custom_invariants: impl IntoIterator<Item = CustomInvariantRegistration>,
        initial_seed_rows: Option<usize>,
    ) -> Result<Self, WorthQueryWorkspaceError> {
        let kind_name = kind_name.into();
        let kind_id = KindId(1);
        let aspect_contracts = declared_aspects
            .iter()
            .map(|binding| binding.contract.clone())
            .collect::<Vec<_>>();
        let aspect_contracts =
            crate::runtime::native_aspect_contracts::WorthQueryNativeAspectContractRegistry::from_contracts(
                aspect_contracts,
            )
            .map_err(|error| WorthQueryWorkspaceError::new(format!("{error:?}")))?;
        let registry = RelationalSchemaRegistry::new()
            .register_entity_kind(EntityKindRegistration {
                kind_id,
                kind_name: kind_name.clone(),
                schema_id: SchemaId("worth-query-memory".to_string()),
                schema_version_id: SchemaVersionId(1),
                aspect_contract_declarations: KindAspectContractDeclarations::new(declared_aspects),
            })
            .map_err(|error| WorthQueryWorkspaceError::new(format!("{error:?}")))?;
        let mut runtime_builder =
            memory_runtime_builder(registry, invariant_catalog, initial_seed_rows);
        for custom_invariant in custom_invariants {
            runtime_builder = runtime_builder.custom_invariant(custom_invariant);
        }
        let runtime = WorthQueryRelationalSourceOwner::new(runtime_builder.build(), "model")
            .map_err(|error| WorthQueryWorkspaceError::new(format!("{error:?}")))?;
        Ok(Self {
            runtime,
            kind_id,
            kind_name,
            aspects,
            aspect_contracts,
            next_client_key: 0,
        })
    }

    pub fn kind_name(&self) -> &str {
        &self.kind_name
    }

    pub fn insert_aspects(
        &mut self,
        aspects: Vec<WorthQueryAuthoredAspectMutation>,
    ) -> Result<WorthQueryMutationReceipt, WorthQueryWorkspaceError> {
        let patch = self.portable_creation_patch_from_authored(&aspects)?;
        self.insert_portable_patch_with_touches(
            &patch,
            aspects.iter().map(|aspect| aspect.aspect_touch()).collect(),
        )
    }

    pub fn update_aspects(
        &mut self,
        entity_identity: WorthQueryEntityIdentity,
        aspects: Vec<WorthQueryAuthoredAspectMutation>,
    ) -> Result<WorthQueryMutationReceipt, WorthQueryWorkspaceError> {
        let patch = self.portable_mutation_patch_from_authored(&aspects)?;
        self.update_portable_patch_with_touches(
            entity_identity,
            &patch,
            aspects.iter().map(|aspect| aspect.aspect_touch()).collect(),
        )
    }

    pub fn delete(
        &mut self,
        entity_identity: WorthQueryEntityIdentity,
    ) -> Result<WorthQueryMutationReceipt, WorthQueryWorkspaceError> {
        let entity_id = super::runtime_identity::entity_id_from_identity(entity_identity.clone())?;
        let (result, snapshot) = self.commit_batch(
            WorkerIntentBatch::new("query-memory-delete").push(MutationIntent::Entity(
                EntityMutationIntent::Delete(DeleteEntityIntent { entity_id }),
            )),
        )?;
        let mut receipt = self.receipt_from_commit(
            result,
            snapshot,
            WorthQueryMutationKind::Deleted,
            Vec::new(),
        );
        if receipt.deltas.is_empty() {
            receipt
                .deltas
                .push(WorthQueryMutationDelta::from_collection_identity(
                    self.mutation_delta_collection_identity(),
                    entity_identity,
                    WorthQueryMutationKind::Deleted,
                    Vec::new(),
                ));
        }
        Ok(receipt)
    }

    pub fn entities(&self) -> Result<Vec<WorthQueryEntity>, WorthQueryWorkspaceError> {
        self.runtime.with_runtime(|runtime| {
            let basis = Self::current_main_basis(runtime)?;
            let projection_scope = self.workspace_projection_scope();
            let view = runtime
                .read_truth()
                .project_observation(&basis.observation())
                .map_err(|_| workspace_basis_unavailable("workspace entity collection"))?;
            Ok(view.entity_records_with_projection_scope(
                self.kind_id,
                projection_scope,
                |record| {
                    let (aspect_values, struct_aspect_values, native_field_values) =
                        self.aspect_projection_from_projection_record(record);
                    Some(WorthQueryEntity::from_aspect_projection(
                        super::runtime_identity::entity_identity(record.entity_id()),
                        aspect_values,
                        struct_aspect_values,
                        native_field_values,
                    ))
                },
            ))
        })
    }

    pub(crate) fn entity(
        &self,
        identity: &WorthQueryEntityIdentity,
    ) -> Result<Option<WorthQueryEntity>, WorthQueryWorkspaceError> {
        let entity_id = super::runtime_identity::entity_id_from_identity(identity.clone())
            .map_err(|_| WorthQueryWorkspaceError::new("entity identity is malformed"))?;
        self.runtime.with_runtime(|runtime| {
            let basis = Self::current_main_basis(runtime)?;
            let projection_scope = self.workspace_projection_scope();
            Ok(runtime
                .read_truth()
                .project_observation(&basis.observation())
                .map_err(|_| workspace_basis_unavailable("workspace entity lookup"))?
                .entity_record_with_projection_scope(entity_id, projection_scope, |record| {
                    let (aspect_values, struct_aspect_values, native_field_values) =
                        self.aspect_projection_from_projection_record(record);
                    Some(WorthQueryEntity::from_aspect_projection(
                        super::runtime_identity::entity_identity(record.entity_id()),
                        aspect_values,
                        struct_aspect_values,
                        native_field_values,
                    ))
                }))
        })
    }

    pub fn snapshot_identity(&self) -> WorthQuerySnapshotIdentity {
        self.runtime
            .with_runtime(super::runtime_identity::snapshot_identity_from_runtime)
    }

    pub(super) fn ensure_entity_exists(
        &self,
        entity_id: EntityId,
    ) -> Result<(), WorthQueryWorkspaceError> {
        self.runtime.with_runtime(|runtime| {
            let basis = Self::current_main_basis(runtime)?;
            runtime
                .read_truth()
                .project_observation(&basis.observation())
                .map_err(|_| WorthQueryWorkspaceError::new("entity basis is unavailable"))?
                .entity_record_with_projection_scope(
                    entity_id,
                    ProjectionAspectScope::empty(),
                    |_| Some(()),
                )
                .ok_or_else(|| WorthQueryWorkspaceError::new("entity not found"))?;
            Ok(())
        })
    }

    fn current_main_basis(
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
    ) -> Result<
        worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        WorthQueryWorkspaceError,
    > {
        let identity = runtime.main_branch_identity();
        runtime
            .observe_branch(&identity)
            .map_err(super::transaction_denial::basis)
            .map(|(_, basis)| basis)
    }
    fn workspace_projection_scope(&self) -> ProjectionAspectScope {
        ProjectionAspectScope::whole_aspects(
            self.aspects
                .iter()
                .map(|aspect| aspect.aspect_touch().parsed_target().aspect_key().clone()),
        )
    }

    fn aspect_projection_from_projection_record(
        &self,
        record: EntityProjectionRecord<'_>,
    ) -> (
        std::collections::BTreeMap<worth_foundational::facade::AspectKey, AspectValue>,
        std::collections::BTreeMap<
            worth_foundational::facade::AspectKey,
            worth_foundational::facade::StructAspectValue,
        >,
        std::collections::BTreeMap<worth_foundational::facade::CanonicalFieldPath, AspectValue>,
    ) {
        let mut aspect_values = std::collections::BTreeMap::new();
        let mut struct_aspect_values = std::collections::BTreeMap::new();
        let mut native_field_values = std::collections::BTreeMap::new();
        for aspect in &self.aspects {
            let target = aspect.aspect_touch().parsed_target();
            if let Some(value) = record.aspect_value(target.aspect_key()) {
                aspect_values.insert(target.aspect_key().clone(), value.clone());
                native_field_values.insert(aspect.native_field_path().clone(), value.clone());
            } else if let Some(value) = record.struct_aspect_value(target.aspect_key()) {
                struct_aspect_values.insert(target.aspect_key().clone(), value.clone());
                if let Some(field) = target
                    .field_path()
                    .and_then(|path| path.fields().first())
                    .and_then(|field| value.get(field))
                {
                    native_field_values.insert(aspect.native_field_path().clone(), field.clone());
                }
            }
        }
        (aspect_values, struct_aspect_values, native_field_values)
    }

    pub(super) fn receipt_from_commit(
        &self,
        result: worth_relational::facade::transactions::CommitResult,
        snapshot_identity: WorthQuerySnapshotIdentity,
        kind: WorthQueryMutationKind,
        touched_aspects: Vec<WorthQueryAspectTouch>,
    ) -> WorthQueryMutationReceipt {
        let deltas = result
            .changed_records
            .iter()
            .filter_map(|record| match record {
                worth_relational::facade::transactions::RecordRef::Entity(entity) => {
                    Some(WorthQueryMutationDelta::from_collection_identity(
                        self.mutation_delta_collection_identity(),
                        super::runtime_identity::entity_identity(*entity),
                        kind.clone(),
                        touched_aspects.clone(),
                    ))
                }
                worth_relational::facade::transactions::RecordRef::Relation(_) => None,
            })
            .collect::<Vec<_>>();
        WorthQueryMutationReceipt {
            commit_identity: WorthQueryCommitIdentity::from_runtime_receipt_commit(
                result.commit.commit_id.0,
            ),
            snapshot_identity,
            deltas,
            bridge_authority: None,
        }
    }

    pub(super) fn mutation_delta_collection_identity(
        &self,
    ) -> crate::runtime::WorthQueryMutationTargetCollectionIdentity {
        crate::runtime::WorthQueryMutationTargetCollectionIdentity::new(
            "memory-workspace-mutation-delta-collection",
            &self.kind_name,
        )
    }
}

fn workspace_basis_unavailable(subject: &str) -> WorthQueryWorkspaceError {
    WorthQueryWorkspaceError::with_kind(
        super::WorthQueryWorkspaceErrorKind::RelationalBasisUnavailable,
        format!("{subject} basis is unavailable"),
    )
}

fn memory_runtime_builder(
    registry: RelationalSchemaRegistry,
    invariant_catalog: InvariantCatalog,
    initial_seed_rows: Option<usize>,
) -> worth_relational::facade::runtime::RelationalRuntimeBuilder {
    let builder = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(registry)
        .invariant_catalog(invariant_catalog.canonicalized());
    let Some(initial_seed_rows) = initial_seed_rows.filter(|rows| *rows > 0) else {
        return builder;
    };
    let defaults = RelationalRuntimeConfig::resolved(
        RelationalRuntimeProfile::CertificationCore,
        RelationalConfigOverride::default(),
    );
    let mut publication = defaults.publication.policy;
    publication.max_patch_records_per_commit = publication
        .max_patch_records_per_commit
        .max(initial_seed_rows);
    builder
        .entity_capacity(
            defaults
                .storage
                .initial_entity_capacity
                .max(initial_seed_rows),
        )
        .publication(publication)
}
