use crate::runtime::WorthQueryRuntimeFacadeFamily;
use std::collections::BTreeSet;

use super::super::support_snapshot::WorthQuerySupportSnapshot;
use super::contract::WorthQuerySupportPinContract;
use super::document::schema::{
    support_pin_vocabulary_identity, WorthQuerySupportPinContractSchemaVersion,
};
use super::error::WorthQuerySupportPinningError;
use super::error::WorthQuerySupportPinningErrorKind;
use super::evidence::{
    derive_support_pin_contract_identity, derive_support_pin_observed_row_identity,
    derive_support_pin_requirement_identity,
};
use super::observed_row::WorthQueryObservedSupportPin;
use super::requirement::{WorthQuerySupportPinRequirement, WorthQuerySupportPinRequirementDraft};
use super::snapshot_index::SupportPinSnapshotIndex;

pub fn support_pinning_contract(
    consumer_name: impl Into<String>,
) -> WorthQuerySupportPinDeclaration {
    WorthQuerySupportPinDeclaration {
        consumer_name: consumer_name.into(),
    }
}

#[derive(Debug)]
pub struct WorthQuerySupportPinDeclaration {
    consumer_name: String,
}

impl WorthQuerySupportPinDeclaration {
    pub fn against_snapshot(
        self,
        snapshot: &WorthQuerySupportSnapshot,
    ) -> Result<WorthQuerySupportPinContractBuilder<'_>, WorthQuerySupportPinningError> {
        Ok(WorthQuerySupportPinContractBuilder {
            consumer_name: self.consumer_name,
            contract_schema_version: WorthQuerySupportPinContractSchemaVersion::current(),
            contract_schema_identity: WorthQuerySupportPinContractSchemaVersion::current()
                .identity()
                .terminal_projection_for_reporting()
                .to_string(),
            pinned_vocabulary_identity: support_pin_vocabulary_identity()?
                .terminal_projection_for_reporting()
                .to_string(),
            support_snapshot_schema_identity: snapshot.schema_identity().to_string(),
            source_matrix_digest: snapshot.source_matrix_digest().to_string(),
            basis_index: SupportPinSnapshotIndex::new(snapshot)?,
            requirements: Vec::new(),
            observed_rows: Vec::new(),
            required_families: BTreeSet::new(),
            observed_families: BTreeSet::new(),
        })
    }
}

#[derive(Debug)]
pub struct WorthQuerySupportPinContractBuilder<'a> {
    consumer_name: String,
    contract_schema_version: WorthQuerySupportPinContractSchemaVersion,
    contract_schema_identity: String,
    pinned_vocabulary_identity: String,
    support_snapshot_schema_identity: String,
    source_matrix_digest: String,
    basis_index: SupportPinSnapshotIndex<'a>,
    requirements: Vec<WorthQuerySupportPinRequirement>,
    observed_rows: Vec<WorthQueryObservedSupportPin>,
    required_families: BTreeSet<WorthQueryRuntimeFacadeFamily>,
    observed_families: BTreeSet<WorthQueryRuntimeFacadeFamily>,
}

impl WorthQuerySupportPinContractBuilder<'_> {
    pub fn require_family(
        mut self,
        family: WorthQueryRuntimeFacadeFamily,
        declare: impl FnOnce(
            WorthQuerySupportPinRequirementDraft,
        ) -> WorthQuerySupportPinRequirementDraft,
    ) -> Result<Self, WorthQuerySupportPinningError> {
        self.reject_duplicate_required_family(family)?;
        let row = self.basis_index.required_row(family)?;
        let draft = declare(WorthQuerySupportPinRequirementDraft::from_snapshot_row(
            family, row,
        ));
        self.requirements
            .push(WorthQuerySupportPinRequirement::from_draft(draft)?);
        self.required_families.insert(family);
        Ok(self)
    }

    pub fn observe_family(
        mut self,
        family: WorthQueryRuntimeFacadeFamily,
    ) -> Result<Self, WorthQuerySupportPinningError> {
        self.reject_duplicate_observed_family(family)?;
        let observed = match self.basis_index.optional_row(family) {
            Some(row) => WorthQueryObservedSupportPin::present(family, row),
            None => WorthQueryObservedSupportPin::missing(family),
        };
        self.observed_rows.push(observed);
        self.observed_families.insert(family);
        Ok(self)
    }

    pub fn seal(self) -> Result<WorthQuerySupportPinContract, WorthQuerySupportPinningError> {
        let requirement_identities = self
            .requirements
            .iter()
            .map(derive_support_pin_requirement_identity)
            .collect::<Vec<_>>();
        let observed_identities = self
            .observed_rows
            .iter()
            .map(derive_support_pin_observed_row_identity)
            .collect::<Vec<_>>();
        let contract_digest = derive_support_pin_contract_identity(
            &self.consumer_name,
            &self.contract_schema_identity,
            &self.pinned_vocabulary_identity,
            &self.support_snapshot_schema_identity,
            &self.source_matrix_digest,
            &requirement_identities,
            &observed_identities,
        )?
        .terminal_projection_for_reporting()
        .to_string();
        WorthQuerySupportPinContract::new(
            self.consumer_name,
            self.contract_schema_version,
            self.contract_schema_identity,
            self.pinned_vocabulary_identity,
            self.support_snapshot_schema_identity,
            self.source_matrix_digest,
            self.requirements,
            self.observed_rows,
            contract_digest,
        )
    }

    fn reject_duplicate_required_family(
        &self,
        family: WorthQueryRuntimeFacadeFamily,
    ) -> Result<(), WorthQuerySupportPinningError> {
        if self.required_families.contains(&family) {
            return Err(WorthQuerySupportPinningError::with_family(
                WorthQuerySupportPinningErrorKind::DuplicateRequiredFamily,
                "support pin required family is declared more than once",
                family.as_str(),
            ));
        }
        if self.observed_families.contains(&family) {
            return Err(WorthQuerySupportPinningError::with_family(
                WorthQuerySupportPinningErrorKind::RequiredObservedFamilyConflict,
                "support pin family cannot be both required and observed",
                family.as_str(),
            ));
        }
        Ok(())
    }

    fn reject_duplicate_observed_family(
        &self,
        family: WorthQueryRuntimeFacadeFamily,
    ) -> Result<(), WorthQuerySupportPinningError> {
        if self.observed_families.contains(&family) {
            return Err(WorthQuerySupportPinningError::with_family(
                WorthQuerySupportPinningErrorKind::DuplicateObservedFamily,
                "support pin observed family is declared more than once",
                family.as_str(),
            ));
        }
        if self.required_families.contains(&family) {
            return Err(WorthQuerySupportPinningError::with_family(
                WorthQuerySupportPinningErrorKind::RequiredObservedFamilyConflict,
                "support pin family cannot be both required and observed",
                family.as_str(),
            ));
        }
        Ok(())
    }
}
