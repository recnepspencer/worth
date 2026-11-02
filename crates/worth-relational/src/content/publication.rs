//! Content admission belongs to the same candidate as its referencing fields.
use super::{ContentDirectory, RelationalContentAttachments, RelationalContentDenial as Denial};
use crate::branch::{RelationalBranchRoot, RelationalBranchRootCaptureDenial};
use serde::{Deserialize, Serialize};
use worth_execution::ExecutionRequest;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct ContentPublication {
    records: RelationalContentAttachments,
    #[serde(skip)]
    admitted: Option<ContentDirectory>,
    #[serde(skip)]
    source: Option<ContentDirectory>,
}

impl PartialEq for ContentPublication {
    fn eq(&self, other: &Self) -> bool {
        self.records == other.records
    }
}
impl Eq for ContentPublication {}

impl ContentPublication {
    pub(super) fn from_readmitted_records(records: RelationalContentAttachments) -> Self {
        Self {
            records,
            admitted: None,
            source: None,
        }
    }

    pub(crate) fn readmit_unchanged_root(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        root: &RelationalBranchRoot,
    ) -> Result<Self, Denial> {
        super::control::run(execution, |control| control.checkpoint(1))?;
        if !self.records.is_empty() {
            return Err(Denial::NonCanonicalContent);
        }
        Ok(Self {
            records: self.records.clone(),
            admitted: Some(root.retained_content().clone()),
            source: None,
        })
    }
    pub(crate) fn records(&self) -> &RelationalContentAttachments {
        &self.records
    }
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
    source: Option<&ContentDirectory>,
) -> Result<ContentPublication, Denial> {
    super::control::run(execution, |_control| {
        let complete = !std::ptr::eq(root.retained_schema_authority().registry(), schema);
        let directory = root.retained_content().prepare(
            execution, schema, records, state, changed, version, source, complete,
        )?;
        Ok(ContentPublication {
            records: records.clone(),
            admitted: Some(directory),
            source: source.cloned(),
        })
    })
}

impl ContentPublication {
    pub(crate) fn prepare_final_fields(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        root: &RelationalBranchRoot,
        schema: &crate::schema::data::RelationalSchemaRegistry,
        state: &crate::runtime::WorkingState,
        changed: &[crate::transactions::data::RecordRef],
        version: crate::identity::data::VersionId,
    ) -> Result<Self, Denial> {
        let prior = root.retained_content();
        let directory = prior.prepare(
            execution,
            schema,
            &self.records,
            state,
            changed,
            version,
            self.source.as_ref(),
            !std::ptr::eq(root.retained_schema_authority().registry(), schema),
        )?;
        let records = directory.publication_records(execution, prior, changed)?;
        Ok(Self {
            records,
            admitted: Some(directory),
            source: None,
        })
    }
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
