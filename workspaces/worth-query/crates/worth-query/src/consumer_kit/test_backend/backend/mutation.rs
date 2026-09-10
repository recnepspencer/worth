use super::*;

impl WorthQueryInMemoryTestBackend {
    pub(super) fn apply_backend_admissible_mutation(
        &mut self,
        mutation: WorthQueryBackendAdmissibleMutation,
    ) -> Result<WorthQueryMutationReceipt, WorthQueryWorkspaceError> {
        match mutation.mutation_family() {
            WorthQueryMutationFamily::Insert => {
                let collection = mutation.declared_collection_identity().ok_or_else(|| {
                    WorthQueryWorkspaceError::new(
                        "insert mutation missing declared collection after admission",
                    )
                })?;
                self.ensure_declared_collection(&collection)?;
                self.workspace
                    .insert_portable_patch(mutation.portable_patch())
            }
            WorthQueryMutationFamily::Update => {
                let entity_identity = mutation.declared_entity_identity().ok_or_else(|| {
                    WorthQueryWorkspaceError::new(
                        "update mutation missing declared entity after admission",
                    )
                })?;
                self.workspace
                    .update_portable_patch(entity_identity, mutation.portable_patch())
            }
            WorthQueryMutationFamily::Delete => {
                let entity_identity = mutation.declared_entity_identity().ok_or_else(|| {
                    WorthQueryWorkspaceError::new(
                        "delete mutation missing declared entity after admission",
                    )
                })?;
                self.workspace.delete(entity_identity)
            }
            unsupported => Err(WorthQueryWorkspaceError::new(format!(
                "write command `{}` should be rejected by admission before execution",
                unsupported.as_str()
            ))),
        }
    }

    pub(super) fn prepare_batch_mutation(
        &self,
        mutation: WorthQueryBackendAdmissibleMutation,
    ) -> Result<WorthQueryMemoryBatchMutation, WorthQueryWorkspaceError> {
        if let Some(collection) = mutation.declared_collection_identity() {
            self.ensure_declared_collection(&collection)?;
        }
        match mutation.mutation_family() {
            WorthQueryMutationFamily::Insert => Ok(WorthQueryMemoryBatchMutation::Insert {
                patch: mutation.portable_patch().clone(),
                touches: mutation.declared_aspect_touches(),
            }),
            WorthQueryMutationFamily::Update => {
                let entity = mutation.declared_entity_identity().ok_or_else(|| {
                    WorthQueryWorkspaceError::new(
                        "batch update missing declared entity after admission",
                    )
                })?;
                Ok(WorthQueryMemoryBatchMutation::Update {
                    entity,
                    patch: mutation.portable_patch().clone(),
                    touches: mutation.declared_aspect_touches(),
                })
            }
            WorthQueryMutationFamily::Delete => {
                let entity = mutation.declared_entity_identity().ok_or_else(|| {
                    WorthQueryWorkspaceError::new(
                        "batch delete missing declared entity after admission",
                    )
                })?;
                Ok(WorthQueryMemoryBatchMutation::Delete {
                    entity,
                    touches: mutation.admitted_touched_aspects().to_vec(),
                })
            }
            unsupported => Err(WorthQueryWorkspaceError::new(format!(
                "batch write command `{}` should be rejected by admission before execution",
                unsupported.as_str()
            ))),
        }
    }
}
