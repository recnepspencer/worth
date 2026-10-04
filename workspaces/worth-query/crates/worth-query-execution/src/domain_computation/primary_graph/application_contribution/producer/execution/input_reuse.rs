use crate::domain_computation::primary_graph::{
    application_contribution::InstalledProducerEdition,
    output_lineage::{invalidation::InvalidationEditAdmission, PreparedInputReuseKey},
    WorthQueryObservedSource, WorthQuerySelectedProductOperation,
};
use worth_query_installation::facade::ApplicationSchema;

use super::{denial, WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind};
use crate::domain_computation::primary_graph::application_contribution::producer::WorthQueryProducerCommitAuthority;

/// A program-bound producer commits under the program its occurrence
/// actually selects. This checks the commit authority before canonical input
/// work; the prepared input's reuse key holds no program.
pub(super) fn require_selected_program<Schema: ApplicationSchema>(
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    authority: &WorthQueryProducerCommitAuthority,
    admission: &mut InvalidationEditAdmission,
) -> Result<(), WorthQueryOutputDemandDenial> {
    match authority {
        WorthQueryProducerCommitAuthority::Ordinary => {}
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
        }
    }
    Ok(())
}

/// An incomplete restored source cannot authorize a contact-free cutoff.
pub(super) fn prepared_key<Query>(
    source: &WorthQueryObservedSource<Query>,
    input_identity: [u8; 32],
    edition: InstalledProducerEdition,
    admission: &mut InvalidationEditAdmission,
) -> Result<Option<PreparedInputReuseKey>, WorthQueryOutputDemandDenial> {
    admission
        .charge_external_work(1)
        .map_err(|_| work_denial())?;
    let Some(selection) = source.retain_selected_membership() else {
        return Ok(None);
    };
    Ok(Some(PreparedInputReuseKey::new(
        selection,
        input_identity,
        edition,
    )))
}

fn work_denial() -> WorthQueryOutputDemandDenial {
    denial(
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        "prepared producer edition exceeds request work",
    )
}
