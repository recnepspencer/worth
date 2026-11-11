use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use std::collections::BTreeSet;

use crate::consumer_kit::support_pinning::error::{
    WorthQuerySupportPinningError, WorthQuerySupportPinningErrorKind,
};
use crate::consumer_kit::support_pinning::evidence::{
    derive_support_pin_contract_identity, derive_support_pin_observed_row_identity,
    derive_support_pin_requirement_identity,
};
use crate::consumer_kit::support_pinning::observed_row::WorthQueryObservedSupportPin;
use crate::consumer_kit::support_pinning::requirement::WorthQuerySupportPinRequirement;
use crate::runtime::WorthQueryRuntimeFacadeFamily;

pub(super) fn rebuild_contract_digest(
    consumer_name: &str,
    contract_schema_identity: &str,
    pinned_vocabulary_identity: &str,
    support_snapshot_schema_identity: &str,
    source_matrix_digest: &str,
    requirements: &[WorthQuerySupportPinRequirement],
    observed_rows: &[WorthQueryObservedSupportPin],
) -> Result<String, IdentityDerivationDenial> {
    let requirement_identities = requirements
        .iter()
        .map(derive_support_pin_requirement_identity)
        .collect::<Result<Vec<_>, IdentityDerivationDenial>>()?;
    let observed_identities = observed_rows
        .iter()
        .map(derive_support_pin_observed_row_identity)
        .collect::<Result<Vec<_>, IdentityDerivationDenial>>()?;
    Ok(derive_support_pin_contract_identity(
        consumer_name,
        contract_schema_identity,
        pinned_vocabulary_identity,
        support_snapshot_schema_identity,
        source_matrix_digest,
        &requirement_identities,
        &observed_identities,
    )?
    .terminal_projection_for_reporting()
    .to_string())
}

pub(super) fn reject_duplicate_document_families(
    requirements: &[WorthQuerySupportPinRequirement],
    observed_rows: &[WorthQueryObservedSupportPin],
) -> Result<(), WorthQuerySupportPinningError> {
    let mut required = BTreeSet::new();
    let mut observed = BTreeSet::new();
    for row in requirements {
        reject_family_insert(&mut required, row.family(), true)?;
    }
    for row in observed_rows {
        reject_family_insert(&mut observed, row.family(), false)?;
        if required.contains(&row.family()) {
            return Err(duplicate_family_error(
                WorthQuerySupportPinningErrorKind::RequiredObservedFamilyConflict,
                row.family(),
            ));
        }
    }
    Ok(())
}

fn reject_family_insert(
    set: &mut BTreeSet<WorthQueryRuntimeFacadeFamily>,
    family: WorthQueryRuntimeFacadeFamily,
    required: bool,
) -> Result<(), WorthQuerySupportPinningError> {
    if set.insert(family) {
        Ok(())
    } else if required {
        Err(duplicate_family_error(
            WorthQuerySupportPinningErrorKind::DuplicateRequiredFamily,
            family,
        ))
    } else {
        Err(duplicate_family_error(
            WorthQuerySupportPinningErrorKind::DuplicateObservedFamily,
            family,
        ))
    }
}

fn duplicate_family_error(
    kind: WorthQuerySupportPinningErrorKind,
    family: WorthQueryRuntimeFacadeFamily,
) -> WorthQuerySupportPinningError {
    WorthQuerySupportPinningError::with_family(
        kind,
        "support pin contract contains duplicate or conflicting family declarations",
        family.as_str(),
    )
}
