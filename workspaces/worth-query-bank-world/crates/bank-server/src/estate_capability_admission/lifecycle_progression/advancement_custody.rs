//! Mandatory review retains its fact-only authority when opening is refused.
use super::*;
use std::num::NonZeroUsize;
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial as Denial,
        WorthQueryManagedComputationResourceDenial as Resource,
        WorthQueryMemoryLimitLevel as Level,
    },
    application_entry::WorthQueryApplicationRequestMutationDenial as Mutation,
    primary_graph::{
        advancement_requests_on_this_thread_for_test as reports,
        bound_advancement_requests_on_this_thread_for_test as bound,
        installed_source_reads_on_this_thread_for_test as reads,
        place_managed_computations_on_this_thread_for_test as place,
        WorthQueryExecutionPlacementForTest as Placement,
    },
};
struct Restore(Placement, Option<worth_foundational::ExecutionBudget>);
impl Drop for Restore {
    fn drop(&mut self) {
        place(self.0);
        bound(self.1);
    }
}
fn assert_refused(denial: BankEstateProgressionDenial, placement: Placement, memory: bool) {
    let BankEstateProgressionDenial::ApplicationEntry(Mutation::ExecutionRequest(cause)) = denial
    else {
        panic!("the public entry names its opening refusal")
    };
    match (placement, memory) {
        (_, false) => assert_eq!(cause, Denial::Resource(Resource::WorkExhausted)),
        (Placement::Serial, true) => {
            assert_eq!(cause, Denial::Resource(Resource::PolicyMemoryLimit))
        }
        (Placement::Leased(_), true) => assert!(matches!(
            cause,
            Denial::Resource(Resource::MemoryLimit {
                level: Level::Policy,
                admitted: 0,
                ..
            })
        )),
        (Placement::World | Placement::Certified { .. }, true) => {
            panic!("the custody matrix selects serial and leased placement explicitly")
        }
    }
}

#[test]
fn mandatory_review_refuses_before_its_admitted_principal_reader() {
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _restore = Restore(place(placement), bound(None));
        for refusal in [None, Some(false), Some(true)] {
            bound(None);
            let fixture = emergency_request_world(
                "mandatory-review-custody",
                GrantSpec::emergency_view(),
                EstateWorkflowStage::Administration,
            );
            let requester = fixture.authenticate();
            let requested = request_elevation(
                &fixture,
                &requester,
                ElevationRequestSpec {
                    grant: GRANT,
                    access: 351,
                    review: 352,
                    idempotency: 81,
                    field: RestrictedBankField::AccountDetails,
                    duration: Duration::from_secs(300),
                },
            );
            let approver = fixture.authenticate_approver();
            let approved = approve_elevation(
                &fixture,
                &approver,
                requested,
                ElevationApprovalSpec {
                    access: 351,
                    idempotency: 83,
                },
            );
            let mandatory = close_approved_elevation(&fixture, &approver, approved);
            let reviewer = fixture.authenticate_reviewer();
            if let Some(memory) = refusal {
                bound(Some(worth_foundational::ExecutionBudget::new(
                    NonZeroUsize::MIN,
                    if memory { 0 } else { 64 * 1024 * 1024 },
                    if memory { 8_000_000 } else { 0 },
                )));
            }
            reports();
            let before = reads();
            let outcome = fixture.runtime.complete_estate_mandatory_review_with_key(
                &reviewer,
                mandatory,
                EstateAction::CompleteMandatoryReview {
                    estate: ESTATE,
                    access: EmergencyAccessId::new(351).unwrap(),
                    review: MandatoryReviewId::new(352).unwrap(),
                },
                &bank_idempotency(87),
                &request_scope(),
            );
            assert_eq!(
                reports().len(),
                1,
                "one review call owns preparation and commit"
            );
            match refusal {
                Some(memory) => {
                    let (denial, retained) = outcome.unwrap_err().into_parts();
                    assert!(
                        retained.is_some(),
                        "opening refused before consuming the mandatory review"
                    );
                    assert_refused(denial, placement, memory);
                    assert_eq!(reads(), before);
                }
                None => {
                    assert!(matches!(
                        outcome.unwrap(),
                        BankEstateMandatoryReviewOutcome::Reviewed(_)
                    ));
                    assert!(
                        reads() > before,
                        "the admitted review reaches its principal reader"
                    );
                }
            }
        }
    }
}
