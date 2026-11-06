use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use std::collections::BTreeSet;

use super::super::support_snapshot::WorthQuerySupportSnapshot;
use super::document::schema::WorthQuerySupportPinContractSchemaVersion;
use super::document::{
    WorthQuerySupportPinContractDocument, WorthQuerySupportPinContractTerminalJsonDocument,
};
use super::error::{WorthQuerySupportPinningError, WorthQuerySupportPinningErrorKind};
use super::evaluation::{evaluate_support_pin_contract, WorthQuerySupportPinReport};
use super::observed_row::WorthQueryObservedSupportPin;
use super::requirement::WorthQuerySupportPinRequirement;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQuerySupportPinContract {
    consumer_name: String,
    contract_schema_version: WorthQuerySupportPinContractSchemaVersion,
    contract_schema_identity: String,
    pinned_vocabulary_identity: String,
    support_snapshot_schema_identity: String,
    source_matrix_digest: String,
    requirements: Vec<WorthQuerySupportPinRequirement>,
    observed_rows: Vec<WorthQueryObservedSupportPin>,
    contract_digest: String,
}

impl WorthQuerySupportPinContract {
    pub(crate) fn new(
        consumer_name: String,
        contract_schema_version: WorthQuerySupportPinContractSchemaVersion,
        contract_schema_identity: String,
        pinned_vocabulary_identity: String,
        support_snapshot_schema_identity: String,
        source_matrix_digest: String,
        requirements: Vec<WorthQuerySupportPinRequirement>,
        observed_rows: Vec<WorthQueryObservedSupportPin>,
        contract_digest: String,
    ) -> Result<Self, WorthQuerySupportPinningError> {
        if consumer_name.trim().is_empty() {
            return Err(WorthQuerySupportPinningError::new(
                WorthQuerySupportPinningErrorKind::BlankConsumerName,
                "support pin consumer name must not be blank",
            ));
        }
        reject_duplicate_contract_families(&requirements, &observed_rows)?;
        Ok(Self {
            consumer_name,
            contract_schema_version,
            contract_schema_identity,
            pinned_vocabulary_identity,
            support_snapshot_schema_identity,
            source_matrix_digest,
            requirements,
            observed_rows,
            contract_digest,
        })
    }

    pub fn evaluate_snapshot(
        &self,
        snapshot: &WorthQuerySupportSnapshot,
    ) -> Result<WorthQuerySupportPinReport, WorthQuerySupportPinningError> {
        evaluate_support_pin_contract(self, snapshot)
    }

    pub fn to_canonical_terminal_json_document(
        &self,
    ) -> Result<WorthQuerySupportPinContractTerminalJsonDocument, WorthQuerySupportPinningError>
    {
        self.to_document().to_canonical_terminal_json_document()
    }

    pub fn to_stable_terminal_json_document(
        &self,
    ) -> Result<WorthQuerySupportPinContractTerminalJsonDocument, WorthQuerySupportPinningError>
    {
        self.to_canonical_terminal_json_document()
    }

    pub(crate) fn to_document(
        &self,
    ) -> Result<WorthQuerySupportPinContractDocument, IdentityDerivationDenial> {
        Ok(WorthQuerySupportPinContractDocument::from_contract(self)?)
    }

    pub fn consumer_name(&self) -> &str {
        &self.consumer_name
    }

    pub fn contract_schema_version(&self) -> WorthQuerySupportPinContractSchemaVersion {
        self.contract_schema_version
    }

    pub fn contract_schema_identity(&self) -> &str {
        &self.contract_schema_identity
    }

    pub fn pinned_vocabulary_identity(&self) -> &str {
        &self.pinned_vocabulary_identity
    }

    pub fn schema_identity(&self) -> &str {
        &self.support_snapshot_schema_identity
    }

    pub fn support_snapshot_schema_identity(&self) -> &str {
        &self.support_snapshot_schema_identity
    }

    pub fn source_matrix_digest(&self) -> &str {
        &self.source_matrix_digest
    }

    pub fn requirements(&self) -> &[WorthQuerySupportPinRequirement] {
        &self.requirements
    }

    pub fn observed_rows(&self) -> &[WorthQueryObservedSupportPin] {
        &self.observed_rows
    }

    pub fn contract_digest(&self) -> &str {
        &self.contract_digest
    }
}

fn reject_duplicate_contract_families(
    requirements: &[WorthQuerySupportPinRequirement],
    observed_rows: &[WorthQueryObservedSupportPin],
) -> Result<(), WorthQuerySupportPinningError> {
    let mut required = BTreeSet::new();
    let mut observed = BTreeSet::new();
    for requirement in requirements {
        if !required.insert(requirement.family()) {
            return Err(WorthQuerySupportPinningError::with_family(
                WorthQuerySupportPinningErrorKind::DuplicateRequiredFamily,
                "support pin required family is declared more than once",
                requirement.family().as_str(),
            ));
        }
    }
    for observed_row in observed_rows {
        if !observed.insert(observed_row.family()) {
            return Err(WorthQuerySupportPinningError::with_family(
                WorthQuerySupportPinningErrorKind::DuplicateObservedFamily,
                "support pin observed family is declared more than once",
                observed_row.family().as_str(),
            ));
        }
        if required.contains(&observed_row.family()) {
            return Err(WorthQuerySupportPinningError::with_family(
                WorthQuerySupportPinningErrorKind::RequiredObservedFamilyConflict,
                "support pin family cannot be both required and observed",
                observed_row.family().as_str(),
            ));
        }
    }
    Ok(())
}
