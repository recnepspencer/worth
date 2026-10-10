//! The real elevation entries refuse before their own principal and decision reads.
use super::fixture::{emergency_request_world, request_scope, GrantSpec, ESTATE, GRANT};
use super::lifecycle_journey::{
    approve_elevation, request_elevation, ElevationApprovalSpec, ElevationRequestSpec,
};
use crate::estate_progression::BankEstateProgressionDenial;
use bank_domain::{
    estate::{
        EmergencyAccessId, EmergencyAccessReason, EstateAction, EstateWorkflowStage,
        MandatoryReviewId, RestrictedBankField,
    },
    proposals::BankIdempotencyKey,
};
use std::{num::NonZeroUsize, time::Duration};
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial as Denial,
        WorthQueryManagedComputationResourceDenial as Resource,
    },
    application_entry::WorthQueryApplicationRequestMutationDenial as Mutation,
    primary_graph::{
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
#[derive(Clone, Copy)]
enum Entry {
    Request,
    Approval,
    Close,
}
#[test]
fn elevation_entries_refuse_before_reading_at_both_placements() {
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _restore = Restore(place(placement), bound(None));
        for entry in [Entry::Request, Entry::Approval, Entry::Close] {
            run(entry, placement, None);
            for memory in [false, true] {
                run(entry, placement, Some(memory));
            }
        }
    }
}
fn run(entry: Entry, placement: Placement, refusal: Option<bool>) {
    bound(None);
    let fixture = emergency_request_world(
        "elevation-custody-probe",
        GrantSpec::emergency_view(),
        EstateWorkflowStage::Administration,
    );
    let requester = fixture.authenticate();
    let approver = fixture.authenticate_approver();
    let scope = request_scope();
    let key = BankIdempotencyKey::new("elevation-custody").unwrap();
    let action = EstateAction::RequestEmergencyAccess {
        estate: ESTATE,
        access: EmergencyAccessId::new(871).unwrap(),
        review: MandatoryReviewId::new(872).unwrap(),
        grant: GRANT,
        reason: EmergencyAccessReason::PreventImmediateLoss,
        field: RestrictedBankField::AccountDetails,
        duration: Duration::from_secs(300),
    };
    let requested = (!matches!(entry, Entry::Request)).then(|| {
        request_elevation(
            &fixture,
            &requester,
            ElevationRequestSpec {
                grant: GRANT,
                access: 871,
                review: 872,
                idempotency: 173,
                field: RestrictedBankField::AccountDetails,
                duration: Duration::from_secs(300),
            },
        )
    });
    let mut requested = requested;
    let approved = matches!(entry, Entry::Close).then(|| {
        approve_elevation(
            &fixture,
            &approver,
            requested.take().unwrap(),
            ElevationApprovalSpec {
                access: 871,
                idempotency: 174,
            },
        )
    });
    if let Some(memory) = refusal {
        bound(Some(worth_foundational::ExecutionBudget::new(
            NonZeroUsize::MIN,
            if memory { 0 } else { 128 * 1024 * 1024 },
            if memory { 16_000_000 } else { 0 },
        )));
    }
    let before = reads();
    let denial = match entry {
        Entry::Request => fixture
            .runtime
            .request_estate_emergency_access_with_key(&requester, action, &key, &scope)
            .err(),
        Entry::Approval => fixture
            .runtime
            .approve_estate_emergency_access_with_key(
                &approver,
                requested.take().unwrap(),
                EstateAction::ApproveEmergencyAccess {
                    estate: ESTATE,
                    access: EmergencyAccessId::new(871).unwrap(),
                },
                &key,
                &scope,
            )
            .err()
            .map(|f| f.into_denial()),
        Entry::Close => fixture
            .runtime
            .revoke_estate_emergency_access_with_key(
                &approver,
                approved.unwrap(),
                EstateAction::RevokeEmergencyAccess {
                    estate: ESTATE,
                    access: EmergencyAccessId::new(871).unwrap(),
                },
                &key,
                &scope,
            )
            .err()
            .map(|f| f.into_denial()),
    };
    if let Some(memory) = refusal {
        let Some(BankEstateProgressionDenial::ApplicationEntry(Mutation::ExecutionRequest(cause))) =
            denial
        else {
            panic!("opening has one spelling: {denial:?}")
        };
        if !memory {
            assert_eq!(cause, Denial::Resource(Resource::WorkExhausted));
        } else if placement == Placement::Serial {
            assert_eq!(cause, Denial::Resource(Resource::PolicyMemoryLimit));
        } else {
            let Denial::Resource(Resource::MemoryLimit {
                level,
                requested,
                admitted,
            }) = cause
            else {
                panic!("exact lease bytes")
            };
            assert_eq!(level, worth_query_host::facade::application_contribution::WorthQueryMemoryLimitLevel::Policy);
            assert!(requested > 0);
            assert_eq!(admitted, 0);
        }
        assert_eq!(reads(), before);
    } else {
        assert!(denial.is_none(), "admitted entry: {denial:?}");
        assert!(
            reads() > before,
            "admitted elevation contacts its real reader"
        );
    }
    bound(None);
}

#[allow(
    dead_code,
    reason = "the workflow proof reuses real Bank journey fixtures across test targets"
)]
#[path = "../approved_payment_workflow/request_phase.rs"]
mod workflow_owner;
