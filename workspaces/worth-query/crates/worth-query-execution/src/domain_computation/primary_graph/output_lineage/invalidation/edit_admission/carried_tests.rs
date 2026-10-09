use super::*;
use crate::domain_computation::execution_runtime::source_invalidation::{
    WorthQueryInvalidationResourceInstallation, WorthQueryInvalidationResources,
};

fn owner() -> SourceInvalidationOwner {
    let resources = WorthQueryInvalidationResources::install(
        WorthQueryInvalidationResourceInstallation::bounded(20, 4096, 4096, 1),
    )
    .unwrap();
    SourceInvalidationOwner::new(resources, 1)
}

#[test]
fn publication_custody_shares_spent_request_work_and_scratch_without_reset() {
    let owner = owner();
    let mut request = owner.edit_admission();
    request.charge_external_work(5).unwrap();
    request.admit_read_scratch(17).unwrap();
    let mut publication = request
        .carry_for_publication(&owner)
        .unwrap()
        .into_admission();
    assert_eq!(request.charged_work(), 6);
    assert_eq!(publication.charged_work(), 6);
    let retained = owner.resources.retained_capacity_bytes();
    assert!(retained > 0);
    request.charge_external_work(2).unwrap();
    publication.charge_external_work(3).unwrap();
    assert_eq!(request.charged_work(), 11);
    assert_eq!(publication.remaining_work(), 9);
    assert!(matches!(
        publication.charge_external_work(10),
        Err(CompanionPreflightStop::WorkExhausted {
            required: 21,
            maximum: 20
        })
    ));
    assert_eq!(
        request.charged_work(),
        11,
        "failed reservation spends no unperformed work"
    );
    publication.charge_external_work(9).unwrap();
    assert_eq!(request.remaining_work(), 0);
    let remaining_bytes = 4096 - request.charged_bytes();
    publication.admit_read_scratch(remaining_bytes - 1).unwrap();
    assert!(matches!(
        request.admit_read_scratch(2),
        Err(CompanionPreflightStop::PreparationMemoryExhausted {
            required: 4097,
            maximum: 4096
        })
    ));
    assert_eq!(publication.charged_bytes(), 4095);
    drop(request);
    assert_eq!(
        owner.resources.retained_capacity_bytes(),
        retained,
        "publication keeps the counter cell after the stack request is gone"
    );
    drop(publication);
    assert_eq!(owner.resources.retained_capacity_bytes(), 0);
}

#[test]
fn successive_custody_handoffs_share_one_cell_and_foreign_owner_cannot_rebind_it() {
    let owner = owner();
    let mut request = owner.edit_admission();
    request.charge_external_work(5).unwrap();
    let mut first = request
        .carry_for_publication(&owner)
        .unwrap()
        .into_admission();
    let retained = owner.resources.retained_capacity_bytes();
    let scratch = request.charged_bytes();
    let mut second = first
        .carry_for_publication(&owner)
        .unwrap()
        .into_admission();
    assert_eq!(owner.resources.retained_capacity_bytes(), retained);
    assert_eq!(
        request.charged_bytes(),
        scratch,
        "second handoff pins existing backing"
    );
    second.charge_external_work(4).unwrap();
    assert_eq!(request.charged_work(), 11);
    let foreign = SourceInvalidationOwner::new(owner.resources.clone(), 2);
    assert!(matches!(
        second.carry_for_publication(&foreign),
        Err(CompanionPreflightStop::SelectedSourceMismatch)
    ));
    assert_eq!(request.charged_work(), 11);
    drop(first);
    drop(second);
    assert_eq!(owner.resources.retained_capacity_bytes(), retained);
    drop(request);
    assert_eq!(owner.resources.retained_capacity_bytes(), 0);
}

#[test]
fn external_work_settlement_refunds_only_its_unused_share_with_another_claimant_live() {
    let owner = owner();
    let mut request = owner.edit_admission();
    let mut publication = request
        .carry_for_publication(&owner)
        .unwrap()
        .into_admission();
    let mut reservation = request.reserve_external_work(10).unwrap();
    publication.charge_external_work(2).unwrap();
    assert_eq!(publication.charged_work(), 13);
    reservation.admission().charge_external_work(3).unwrap();
    assert_eq!(publication.charged_work(), 16);
    reservation.settle(4).unwrap();
    assert_eq!(publication.charged_work(), 10);
    assert_eq!(request.charged_work(), 10);
    publication.charge_external_work(10).unwrap();
    assert_eq!(request.remaining_work(), 0);
}

#[test]
fn denied_custody_preparation_preserves_spent_local_counters_and_refunds_retention() {
    let owner = owner();
    let mut request = InvalidationEditAdmission::new(CompanionPreflightBudget {
        maximum_work_visits: 5,
        maximum_preparation_bytes: 4096,
    });
    request.charge_external_work(5).unwrap();
    assert!(matches!(
        request.carry_for_publication(&owner),
        Err(CompanionPreflightStop::WorkExhausted {
            required: 6,
            maximum: 5
        })
    ));
    assert_eq!(request.charged_work(), 5);
    assert_eq!(owner.resources.retained_capacity_bytes(), 0);
}

#[test]
fn retained_index_subtotal_is_shared_but_scratch_and_separate_tickets_are_not_index() {
    let owner = owner();
    let mut request = owner.edit_admission();
    request.index_bytes(13).unwrap();
    request.admit_read_scratch(17).unwrap();
    let checkpoint = request.index_checkpoint();
    let mut publication = request
        .carry_for_publication(&owner)
        .unwrap()
        .into_admission();
    assert_eq!(publication.charged_index_bytes(), 13);
    publication.index_bytes(23).unwrap();
    assert_eq!(request.charged_index_bytes(), 36);
    assert!(request.charged_bytes() > request.charged_index_bytes());
    let before = request.charged_bytes();
    assert!(matches!(
        publication.record_index_bytes(u64::MAX),
        Err(CompanionPreflightStop::PreparationMemoryCounterOverflow)
    ));
    assert_eq!(publication.charged_index_bytes(), 36);
    assert_eq!(request.charged_index_bytes(), 36);
    assert_eq!(request.charged_bytes(), before);
    assert_eq!(request.index_bytes_since(checkpoint).unwrap(), 23);
}
