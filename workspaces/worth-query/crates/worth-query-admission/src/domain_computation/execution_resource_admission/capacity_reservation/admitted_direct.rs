//! Metered direct capacity custody, including terminal receipt storage.

use super::*;

#[cfg(test)]
mod tests;

pub(crate) fn reserve_execution_resource_plan_admitted<Stop>(
    mut resources: WorthQueryAdmittedExecutionResourcePlan,
    admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
) -> Result<
    Option<WorthQueryCapacityReservedExecutionResourcePlan>,
    WorthQueryCapacityReservationAdmissionStop<Stop>,
> {
    use WorthQueryCapacityReservationAdmissionStop as Refusal;
    // Snapshot collection headers and the terminal identity header.
    admit(6, 0).map_err(Refusal::Admission)?;
    resources.record_capacity_reservation_check();
    let snapshot = resources.support_snapshot();
    let count = snapshot
        .conditional_nodes()
        .len()
        .checked_add(snapshot.graph_providers().len())
        .and_then(|count| count.checked_add(snapshot.commit_providers().len()))
        .and_then(|count| count.checked_add(usize::from(snapshot.parallel_admission().is_some())))
        .and_then(|count| count.checked_add(1))
        .ok_or(Refusal::AccountingOverflow)?;
    let identity_bytes =
        u64::try_from(resources.identity().len()).map_err(|_| Refusal::AccountingOverflow)?;
    let storage = count
        .checked_mul(
            std::mem::size_of::<&WorthQueryExecutionResourceSupport>()
                + std::mem::size_of::<Box<dyn WorthQueryExecutionCapacityReservation>>(),
        )
        .and_then(|bytes| u64::try_from(bytes).ok())
        .and_then(|bytes| bytes.checked_add(identity_bytes))
        .ok_or(Refusal::AccountingOverflow)?;
    admit(identity_bytes, storage).map_err(Refusal::Admission)?;
    let release_identity = resources.identity().to_owned();
    let mut requested: Vec<&WorthQueryExecutionResourceSupport> = Vec::with_capacity(count);
    let mut reservations = Vec::with_capacity(count);
    // Iterator selection reads each participating collection before its body.
    admit(
        u64::try_from(count).map_err(|_| Refusal::AccountingOverflow)?,
        0,
    )
    .map_err(Refusal::Admission)?;
    for support in snapshot.all_supports() {
        admit(1, 0).map_err(Refusal::Admission)?;
        let identity = support.capacity_subject_identity();
        let mut duplicate = false;
        for existing in &requested {
            admit(1, 0).map_err(Refusal::Admission)?;
            let existing_identity = existing.capacity_subject_identity();
            let work = identity
                .len()
                .checked_add(existing_identity.len())
                .and_then(|bytes| u64::try_from(bytes).ok())
                .and_then(|bytes| bytes.checked_add(1))
                .ok_or(Refusal::AccountingOverflow)?;
            admit(work, 0).map_err(Refusal::Admission)?;
            if identity == existing_identity {
                admit(3, 0).map_err(Refusal::Admission)?;
                if !existing.has_same_capacity_authority(support) {
                    return Ok(None);
                }
                duplicate = true;
                break;
            }
        }
        if !duplicate {
            admit(
                std::mem::size_of::<&WorthQueryExecutionResourceSupport>() as u64,
                0,
            )
            .map_err(Refusal::Admission)?;
            requested.push(support);
        }
    }
    admit(1, 0).map_err(Refusal::Admission)?;
    admit(
        u64::try_from(requested.len()).map_err(|_| Refusal::AccountingOverflow)?,
        0,
    )
    .map_err(Refusal::Admission)?;
    for support in requested {
        admit(1, 0).map_err(Refusal::Admission)?;
        let (work, bytes) = support
            .capacity()
            .reservation_preflight_cost()
            .ok_or(Refusal::AccountingOverflow)?;
        let work = work
            .checked_add(
                std::mem::size_of::<Box<dyn WorthQueryExecutionCapacityReservation>>() as u64,
            )
            .ok_or(Refusal::AccountingOverflow)?;
        admit(work, bytes).map_err(Refusal::Admission)?;
        let Some(reservation) = support.capacity().try_reserve() else {
            return Ok(None);
        };
        reservations.push(reservation);
    }
    admit(
        std::mem::size_of::<WorthQueryCapacityReservedExecutionResourcePlan>() as u64 + 1,
        0,
    )
    .map_err(Refusal::Admission)?;
    resources.record_capacity_reservation();
    Ok(Some(WorthQueryCapacityReservedExecutionResourcePlan {
        resources,
        provider_reservations: reservations,
        release_identity,
    }))
}
