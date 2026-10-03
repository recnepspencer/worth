use crate::domain_computation::primary_graph::{
    application_contribution::InstalledProducerEdition,
    output_lineage::{invalidation::InvalidationEditAdmission, PreparedInputReuseKey},
    WorthQueryObservedSource, WorthQuerySelectedProductOperation,
};
use worth_query_installation::facade::ApplicationSchema;

use super::{denial, WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind};
use crate::domain_computation::primary_graph::application_contribution::producer::WorthQueryProducerCommitAuthority;

/// Actual selected program meaning, resolved before canonical input work.
/// Only this module constructs the progression consumed by `prepared_key`.
pub(super) struct ResolvedProducerProgram {
    selected: Option<crate::domain_computation::primary_graph::WorthQuerySelectedProgramInspection>,
}

pub(super) fn resolve_program<Schema: ApplicationSchema>(
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    authority: &WorthQueryProducerCommitAuthority,
    admission: &mut InvalidationEditAdmission,
) -> Result<ResolvedProducerProgram, WorthQueryOutputDemandDenial> {
    let selected_program = match authority {
        WorthQueryProducerCommitAuthority::Ordinary => None,
        WorthQueryProducerCommitAuthority::ProgramOutput
        | WorthQueryProducerCommitAuthority::SelectedProgram { .. } => {
            admission
                .charge_external_work(1)
                .map_err(|_| work_denial())?;
            let actual = selected.inspect_selected_program().map_err(|_| {
                denial(
                    WorthQueryOutputDemandDenialKind::PublicationStale,
                    "selected producer program could not be inspected",
                )
            })?;
            if let WorthQueryProducerCommitAuthority::SelectedProgram { identity, revision } =
                authority
            {
                let comparison = identity
                    .as_str()
                    .len()
                    .max(actual.identity().as_str().len())
                    .checked_add(32)
                    .and_then(|work| u64::try_from(work).ok())
                    .ok_or_else(work_denial)?;
                admission
                    .charge_external_work(comparison)
                    .map_err(|_| work_denial())?;
                if identity != actual.identity() || revision != actual.revision() {
                    return Err(denial(
                        WorthQueryOutputDemandDenialKind::PublicationStale,
                        "presented producer program differs from the selected occurrence",
                    ));
                }
            }
            Some(actual)
        }
    };
    Ok(ResolvedProducerProgram {
        selected: selected_program,
    })
}

/// An incomplete restored source cannot authorize a contact-free cutoff.
/// Program meaning is carried from the earlier selected occurrence check.
pub(super) fn prepared_key<Query>(
    source: &WorthQueryObservedSource<Query>,
    input_identity: [u8; 32],
    edition: InstalledProducerEdition,
    program: ResolvedProducerProgram,
    admission: &mut InvalidationEditAdmission,
) -> Result<Option<PreparedInputReuseKey>, WorthQueryOutputDemandDenial> {
    admission
        .charge_external_work(1)
        .map_err(|_| work_denial())?;
    let Some(selection) = source.retain_selected_membership() else {
        return Ok(None);
    };
    let selected_program = program
        .selected
        .map(|actual| (actual.identity().clone(), *actual.revision()));
    Ok(Some(PreparedInputReuseKey::new(
        selection,
        input_identity,
        edition,
        selected_program,
    )))
}

fn work_denial() -> WorthQueryOutputDemandDenial {
    denial(
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        "prepared producer edition exceeds request work",
    )
}
