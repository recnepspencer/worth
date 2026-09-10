use std::collections::{BTreeMap, BTreeSet};

use super::{
    error::{WorthQueryTestBackendError, WorthQueryTestBackendErrorKind},
    WorthQueryInMemoryTestRuntimeBuilder,
};

pub struct WorthQueryTestSeedRow {
    identity: String,
    collection: crate::runtime::WorthQueryMutationTargetCollectionIdentity,
    aspects: Vec<crate::runtime::WorthQueryAuthoredAspectMutation>,
}

pub(super) struct WorthQueryTestSeedSpecification {
    identity_touch: crate::runtime::WorthQueryAspectTouch,
    rows: Vec<WorthQueryTestSeedRow>,
}

impl WorthQueryTestSeedSpecification {
    pub(super) fn len(&self) -> usize {
        self.rows.len()
    }
}

#[derive(Default)]
pub struct WorthQueryTestSeedReceipt {
    entities: BTreeMap<String, crate::memory_workspace::WorthQueryEntityIdentity>,
    commit_identity: Option<crate::memory_workspace::WorthQueryCommitIdentity>,
}

impl WorthQueryTestSeedRow {
    pub fn new(
        identity: impl Into<String>,
        collection: impl Into<String>,
        declaration: impl FnOnce(
            crate::runtime::WorthQueryAspectMutationBuilder,
        ) -> crate::runtime::WorthQueryAspectMutationBuilder,
    ) -> Result<Self, crate::runtime::WorthQueryRuntimeError> {
        let command = declaration(crate::runtime::WorthQueryAspectMutationBuilder::new())
            .build_insert(collection)?;
        let crate::runtime::WorthQueryWriteCommand::InsertAspects {
            collection,
            aspects,
            ..
        } = command
        else {
            unreachable!("the seed-row constructor only builds an insertion")
        };
        Ok(Self {
            identity: identity.into(),
            collection,
            aspects,
        })
    }
}

impl WorthQueryTestSeedReceipt {
    pub fn relational_record(
        &self,
        identity: &str,
    ) -> Option<worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts> {
        self.entities.get(identity)?.relational_record_parts()
    }

    pub fn entity(
        &self,
        identity: &str,
    ) -> Option<&crate::memory_workspace::WorthQueryEntityIdentity> {
        self.entities.get(identity)
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    pub fn commit_count(&self) -> usize {
        usize::from(self.commit_identity.is_some())
    }

    pub fn commit_identity(&self) -> Option<&crate::memory_workspace::WorthQueryCommitIdentity> {
        self.commit_identity.as_ref()
    }
}

impl WorthQueryInMemoryTestRuntimeBuilder {
    pub fn seed_collection_rows(
        mut self,
        identity_touch: crate::runtime::WorthQueryAspectTouch,
        rows: Vec<WorthQueryTestSeedRow>,
    ) -> Result<Self, WorthQueryTestBackendError> {
        if rows.is_empty() {
            return Err(seed_error("initial collection seed must contain rows"));
        }
        if self.initial_seed.is_some() {
            return Err(seed_error(
                "the in-memory test builder accepts one canonical initial seed",
            ));
        }
        let identities = rows
            .iter()
            .map(|row| row.identity.as_str())
            .collect::<BTreeSet<_>>();
        if identities.len() != rows.len() {
            return Err(seed_error(
                "initial collection seed identities must be unique",
            ));
        }
        self.initial_seed = Some(WorthQueryTestSeedSpecification {
            identity_touch,
            rows,
        });
        Ok(self)
    }
}

pub(super) fn apply_initial_seed(
    workspace: &mut crate::memory_workspace::WorthQueryMemoryWorkspace,
    collection: &str,
    specification: Option<WorthQueryTestSeedSpecification>,
) -> Result<WorthQueryTestSeedReceipt, WorthQueryTestBackendError> {
    let Some(specification) = specification else {
        return Ok(WorthQueryTestSeedReceipt::default());
    };
    if specification
        .rows
        .iter()
        .any(|row| row.collection.as_str() != collection)
    {
        return Err(seed_error(
            "every initial seed row must target the test schema collection",
        ));
    }
    let expected = specification
        .rows
        .iter()
        .map(|row| row.identity.clone())
        .collect::<BTreeSet<_>>();
    let commit_identity = workspace
        .insert_seed_rows_atomically(
            specification
                .rows
                .into_iter()
                .map(|row| row.aspects)
                .collect(),
        )
        .map_err(|error| seed_error(format!("initial seed commit failed: {error}")))?;
    let entities = workspace
        .seeded_identity_entities(&specification.identity_touch)
        .map_err(|error| seed_error(error.to_string()))?;
    if entities.keys().cloned().collect::<BTreeSet<_>>() != expected {
        return Err(seed_error(
            "initial seed identity values do not match the declared seed keys",
        ));
    }
    Ok(WorthQueryTestSeedReceipt {
        entities,
        commit_identity: Some(commit_identity),
    })
}

fn seed_error(message: impl Into<String>) -> WorthQueryTestBackendError {
    WorthQueryTestBackendError::new(
        WorthQueryTestBackendErrorKind::WorkspaceBuildFailed,
        message,
    )
}
