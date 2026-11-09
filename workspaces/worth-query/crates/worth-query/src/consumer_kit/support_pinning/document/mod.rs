use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
mod integrity;
mod row_documents;
pub(crate) mod schema;
mod semantic_admission;
mod terminal_json_codec;

use std::borrow::Cow;

use super::contract::WorthQuerySupportPinContract;
use super::error::{WorthQuerySupportPinningError, WorthQuerySupportPinningErrorKind};
use super::evidence::derive_support_pin_contract_document_identity;
use super::observed_row::WorthQueryObservedSupportPin;
use super::requirement::WorthQuerySupportPinRequirement;
use integrity::{rebuild_contract_digest, reject_duplicate_document_families};
use row_documents::{
    WorthQueryObservedSupportPinDocument, WorthQuerySupportPinRequirementDocument,
};
use schema::support_pin_vocabulary_identity;

pub use schema::WorthQuerySupportPinContractSchemaVersion;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryExternalSupportPinContractTerminalJsonDocument {
    text: Cow<'static, str>,
}

impl WorthQueryExternalSupportPinContractTerminalJsonDocument {
    pub fn from_external_terminal_json_document(text: impl Into<String>) -> Self {
        Self {
            text: Cow::Owned(text.into()),
        }
    }

    pub const fn from_static_external_terminal_json_document(text: &'static str) -> Self {
        Self {
            text: Cow::Borrowed(text),
        }
    }

    pub fn as_str(&self) -> &str {
        self.text.as_ref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQuerySupportPinContractTerminalJsonDocument {
    text: String,
}

impl WorthQuerySupportPinContractTerminalJsonDocument {
    pub(crate) fn from_native_terminal_projection(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    pub fn as_str(&self) -> &str {
        &self.text
    }

    pub fn to_external_terminal_json_document(
        &self,
    ) -> WorthQueryExternalSupportPinContractTerminalJsonDocument {
        WorthQueryExternalSupportPinContractTerminalJsonDocument::from_external_terminal_json_document(
            self.text.clone(),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub(crate) struct WorthQuerySupportPinContractDocument {
    schema_version: u16,
    schema_identity: String,
    pinned_vocabulary_identity: String,
    support_snapshot_schema_identity: String,
    source_matrix_digest: String,
    consumer_name: String,
    contract_digest: String,
    document_digest: String,
    requirements: Vec<WorthQuerySupportPinRequirementDocument>,
    observed_rows: Vec<WorthQueryObservedSupportPinDocument>,
}

impl WorthQuerySupportPinContractDocument {
    pub(crate) fn from_contract(
        contract: &WorthQuerySupportPinContract,
    ) -> Result<Self, IdentityDerivationDenial> {
        let mut document = Self {
            schema_version: contract.contract_schema_version().major(),
            schema_identity: contract.contract_schema_identity().to_string(),
            pinned_vocabulary_identity: contract.pinned_vocabulary_identity().to_string(),
            support_snapshot_schema_identity: contract
                .support_snapshot_schema_identity()
                .to_string(),
            source_matrix_digest: contract.source_matrix_digest().to_string(),
            consumer_name: contract.consumer_name().to_string(),
            contract_digest: contract.contract_digest().to_string(),
            document_digest: String::new(),
            requirements: contract
                .requirements()
                .iter()
                .map(WorthQuerySupportPinRequirementDocument::from_requirement)
                .collect(),
            observed_rows: contract
                .observed_rows()
                .iter()
                .map(WorthQueryObservedSupportPinDocument::from_observed)
                .collect(),
        };
        document.document_digest =
            derive_support_pin_contract_document_identity(&document.contract_digest)?
                .terminal_projection_for_reporting()
                .to_string();
        Ok(document)
    }

    pub fn from_terminal_json_document(
        terminal_json_document: &WorthQueryExternalSupportPinContractTerminalJsonDocument,
    ) -> Result<Self, WorthQuerySupportPinningError> {
        terminal_json_codec::decode_external_terminal_json_document(terminal_json_document)
    }

    pub fn to_canonical_terminal_json_document(
        &self,
    ) -> Result<WorthQuerySupportPinContractTerminalJsonDocument, WorthQuerySupportPinningError>
    {
        terminal_json_codec::encode_native_terminal_json_document(self)
    }

    pub(crate) fn validate(
        self,
        expected_schema_version: WorthQuerySupportPinContractSchemaVersion,
    ) -> Result<WorthQuerySupportPinContract, WorthQuerySupportPinningError> {
        self.reject_schema_mismatch(expected_schema_version)?;
        self.reject_vocabulary_mismatch()?;
        self.reject_document_digest_mismatch()?;
        let requirements = self.validated_requirements()?;
        let observed_rows = self.validated_observed_rows()?;
        reject_duplicate_document_families(&requirements, &observed_rows)?;
        let contract_digest = rebuild_contract_digest(
            &self.consumer_name,
            &self.schema_identity,
            &self.pinned_vocabulary_identity,
            &self.support_snapshot_schema_identity,
            &self.source_matrix_digest,
            &requirements,
            &observed_rows,
        )?;
        if contract_digest != self.contract_digest {
            return Err(WorthQuerySupportPinningError::with_expected_found(
                WorthQuerySupportPinningErrorKind::ContractDigestMismatch,
                "support pin contract digest mismatch",
                contract_digest,
                self.contract_digest,
            ));
        }
        WorthQuerySupportPinContract::new(
            self.consumer_name,
            expected_schema_version,
            self.schema_identity,
            self.pinned_vocabulary_identity,
            self.support_snapshot_schema_identity,
            self.source_matrix_digest,
            requirements,
            observed_rows,
            contract_digest,
        )
    }

    fn reject_schema_mismatch(
        &self,
        expected_schema_version: WorthQuerySupportPinContractSchemaVersion,
    ) -> Result<(), WorthQuerySupportPinningError> {
        if self.schema_version != expected_schema_version.major() {
            return Err(WorthQuerySupportPinningError::with_expected_found(
                WorthQuerySupportPinningErrorKind::SchemaMismatch,
                "support pin contract schema version mismatch",
                expected_schema_version.major().to_string(),
                self.schema_version.to_string(),
            ));
        }
        let expected_identity = expected_schema_version
            .identity()?
            .terminal_projection_for_reporting()
            .to_string();
        if self.schema_identity != expected_identity {
            return Err(WorthQuerySupportPinningError::with_expected_found(
                WorthQuerySupportPinningErrorKind::SchemaMismatch,
                "support pin contract schema identity mismatch",
                expected_identity,
                self.schema_identity.clone(),
            ));
        }
        Ok(())
    }

    fn reject_vocabulary_mismatch(&self) -> Result<(), WorthQuerySupportPinningError> {
        let expected = support_pin_vocabulary_identity()?
            .terminal_projection_for_reporting()
            .to_string();
        if self.pinned_vocabulary_identity == expected {
            Ok(())
        } else {
            Err(WorthQuerySupportPinningError::with_expected_found(
                WorthQuerySupportPinningErrorKind::VocabularyMismatch,
                "support pin contract vocabulary identity mismatch",
                expected,
                self.pinned_vocabulary_identity.clone(),
            ))
        }
    }

    fn reject_document_digest_mismatch(&self) -> Result<(), WorthQuerySupportPinningError> {
        let expected = derive_support_pin_contract_document_identity(&self.contract_digest)?
            .terminal_projection_for_reporting()
            .to_string();
        if self.document_digest == expected {
            Ok(())
        } else {
            Err(WorthQuerySupportPinningError::with_expected_found(
                WorthQuerySupportPinningErrorKind::ContractDigestMismatch,
                "support pin contract document digest mismatch",
                expected,
                self.document_digest.clone(),
            ))
        }
    }

    fn validated_requirements(
        &self,
    ) -> Result<Vec<WorthQuerySupportPinRequirement>, WorthQuerySupportPinningError> {
        self.requirements.iter().map(|row| row.validate()).collect()
    }

    fn validated_observed_rows(
        &self,
    ) -> Result<Vec<WorthQueryObservedSupportPin>, WorthQuerySupportPinningError> {
        self.observed_rows
            .iter()
            .map(|row| row.validate())
            .collect()
    }
}
