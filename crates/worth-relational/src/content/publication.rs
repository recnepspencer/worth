//! Content admission belongs to the same candidate as its referencing fields.
use super::{ContentDirectory, RelationalContentAttachments, RelationalContentDenial as Denial};
use crate::branch::{RelationalBranchRoot, RelationalBranchRootCaptureDenial};
use serde::{Deserialize, Serialize};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{AspectShape, AspectValue, ContractValidatedAspectValueView};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct ContentPublication {
    records: RelationalContentAttachments,
    #[serde(skip)]
    admitted: Option<ContentDirectory>,
}

impl PartialEq for ContentPublication {
    fn eq(&self, other: &Self) -> bool {
        self.records == other.records
    }
}
impl Eq for ContentPublication {}

impl ContentPublication {
    pub(crate) fn directory(&self) -> Result<ContentDirectory, RelationalBranchRootCaptureDenial> {
        match &self.admitted {
            Some(directory) => Ok(directory.clone()),
            None if self.records.is_empty() => Ok(ContentDirectory::default()),
            None => Err(RelationalBranchRootCaptureDenial::UnadmittedContent),
        }
    }
}

pub(crate) fn prepare_publication(
    execution: &ExecutionRequest<'_, '_>,
    root: &RelationalBranchRoot,
    schema: &crate::schema::data::RelationalSchemaRegistry,
    records: &RelationalContentAttachments,
    state: &crate::runtime::WorkingState,
    changed: &[crate::transactions::data::RecordRef],
    version: crate::identity::data::VersionId,
) -> Result<ContentPublication, Denial> {
    super::control::run(execution, |_control| {
        // Ordinary edits retain this immutable registry. Replacing it needs
        // complete closure reconstruction, rather than a global schema scan.
        if !root.retained_content().is_empty()
            && !std::ptr::eq(root.retained_schema_authority().registry(), schema)
        {
            return Err(Denial::SchemaTransitionRequiresReconstruction);
        }
        let directory =
            root.retained_content()
                .prepare(execution, &schema.content_formats, records)?;
        validate_fields(execution, &directory, schema, state, changed, version)?;
        Ok(ContentPublication {
            records: records.clone(),
            admitted: Some(directory),
        })
    })
}

impl ContentPublication {
    pub(crate) fn validate_final_fields(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        schema: &crate::schema::data::RelationalSchemaRegistry,
        state: &crate::runtime::WorkingState,
        changed: &[crate::transactions::data::RecordRef],
        version: crate::identity::data::VersionId,
    ) -> Result<(), Denial> {
        let directory = self
            .admitted
            .as_ref()
            .ok_or(Denial::EncodingInconsistency)?;
        validate_fields(execution, directory, schema, state, changed, version)
    }
}

fn validate_fields(
    execution: &ExecutionRequest<'_, '_>,
    directory: &ContentDirectory,
    schema: &crate::schema::data::RelationalSchemaRegistry,
    state: &crate::runtime::WorkingState,
    changed: &[crate::transactions::data::RecordRef],
    version: crate::identity::data::VersionId,
) -> Result<(), Denial> {
    super::control::run(execution, |control| {
        let view = crate::validation::engine::state_view::InvariantStateView::new(state, version);
        for record in changed {
            control.checkpoint(1)?;
            let aspects = match record {
                crate::transactions::data::RecordRef::Entity(id) => view.entity_aspect_state(*id),
                crate::transactions::data::RecordRef::Relation(id) => {
                    view.relation_aspect_state(*id)
                }
            };
            let Some(aspects) = aspects else {
                continue;
            };
            for (_, value) in aspects.aspects().entries() {
                match value.view() {
                    ContractValidatedAspectValueView::Scalar(AspectValue::ContentRef(_)) => {
                        return Err(Denial::FormatBindingMissing)
                    }
                    ContractValidatedAspectValueView::Scalar(_) => {}
                    ContractValidatedAspectValueView::Struct(values) => {
                        let AspectShape::Struct(shape) = value.contract().shape() else {
                            return Err(Denial::EncodingInconsistency);
                        };
                        for (key, value) in values.fields() {
                            control.checkpoint(1)?;
                            if let AspectValue::ContentRef(identity) = value {
                                let height = usize::BITS - shape.fields().len().leading_zeros() + 1;
                                control
                                    .checkpoint(u64::from(height) * key.as_str().len() as u64)?;
                                let index = shape
                                    .fields()
                                    .binary_search_by(|field| field.key().cmp(key))
                                    .map_err(|_| Denial::FormatBindingMissing)?;
                                let declaration = &shape.fields()[index];
                                let binding =
                                    declaration.codec().ok_or(Denial::FormatBindingMissing)?;
                                control.checkpoint(schema.content_formats.lookup_bound())?;
                                let codec = schema.content_formats.resolve(binding)?;
                                directory.read(execution, *identity, codec.descriptor())?;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    })
}

pub(crate) fn commit_denial(denial: Denial) -> crate::transactions::data::TransactionCommitError {
    use crate::transactions::data::{
        CommitConflict, CommitExecutionDenial, CommitExecutionDenialKind, ConflictClass,
        TransactionCommitError,
    };
    match denial {
        Denial::Execution(cause) => TransactionCommitError::execution(CommitExecutionDenial {
            kind: CommitExecutionDenialKind::Cause(cause),
            partition_identity: None,
        }),
        denial => {
            TransactionCommitError::conflict(CommitConflict::new(ConflictClass::ContentAdmission {
                denial,
            }))
        }
    }
}
