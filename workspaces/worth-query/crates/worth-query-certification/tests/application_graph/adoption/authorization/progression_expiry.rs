//! Real authentication expires while the native trusted clock services progression.
use super::lifecycle::ExpiringOperator;
use super::*;
use std::collections::VecDeque;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};
use worth_query_host::facade::admission::authenticated_principal::*;
use worth_query_host::facade::application_entry::{
    WorthQueryBranchSetAdoptionAdvanceDenial, WorthQueryBranchSetAdoptionProgress,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryOperationAuthorizationRefusalKind, WorthQueryRuntimeTimeSource,
    WorthQueryRuntimeTimeSourceDenial,
};
#[derive(Clone, Default)]
struct ExpiryClock(Arc<Mutex<VecDeque<Option<Instant>>>>);
impl WorthQueryRuntimeTimeSource for ExpiryClock {
    fn current_time(&self) -> Result<SystemTime, WorthQueryRuntimeTimeSourceDenial> {
        let wait = self.0.lock().unwrap().pop_front().flatten();
        if let Some(deadline) = wait {
            std::thread::sleep(
                deadline.saturating_duration_since(Instant::now()) + Duration::from_millis(1),
            );
        }
        Ok(SystemTime::now())
    }
}
impl ExpiryClock {
    fn expire_during_progression(&self, deadline: Instant) {
        // Admission samples first; the existing owner's currentness refresh samples
        // next. Only the native clock blocks; no authorization decision is replaced.
        *self.0.lock().unwrap() = VecDeque::from([None, Some(deadline)]);
    }
    fn assert_serviced(&self) {
        assert!(self.0.lock().unwrap().is_empty());
    }
}
fn authenticate_live_expiring(
    host: &DocumentRetentionRuntime<RetentionProgramP0>,
    scope: &WorthQueryRequestScope,
) -> WorthQueryAuthenticatedExternalPrincipal<
    crate::document_retention_model::schema::DocumentRetentionSchema,
> {
    let adapter = admit_authentication_adapter(
        host.installed_schema(),
        WorthQueryAuthenticationAdapterAdmission::new(
            WorthQueryAuthenticationAudience::new("document-retention-host").unwrap(),
            WorthQueryAuthenticationMethod::new("certification").unwrap(),
        ),
        ExpiringOperator,
    )
    .unwrap();
    let principal = crate::document_retention_model::operator_identity::block_on(
        adapter.authenticate(Duration::from_secs(1), scope),
    )
    .unwrap();
    assert!(!principal.is_expired());
    principal
}
fn expired(cause: &WorthQueryProgramAdoptionAuthorizationDenial) {
    match cause {
        WorthQueryProgramAdoptionAuthorizationDenial::Authorization(denial) => assert_eq!(
            denial.kind(),
            WorthQueryOperationAuthorizationDenialKind::Refused(
                WorthQueryOperationAuthorizationRefusalKind::ExpiredAuthentication
            )
        ),
        other => panic!(
            "expiry after access admission must reach the real capability progression: {other:?}"
        ),
    }
}
#[test]
fn expiry_during_capability_progression_preserves_the_native_prepared_owner() {
    let clock = ExpiryClock::default();
    let host = crate::document_retention_model::host::publish_on_first_program_with_trusted_time(
        clock.clone(),
    );
    let branch = host.current_world();
    let target = *host
        .supported_program::<RetentionProgramP1>()
        .unwrap()
        .owned_revision();
    let scope = request_scope();
    let current = authenticate_operator(host.installed_schema(), &scope);
    let programs = host.runtime().request(&current, &scope).programs();
    let requirements = programs.compare(&target).unwrap();
    let prepared = programs.adopt(&requirements).unwrap().prepare(64).unwrap();
    let before = host
        .with_host_advancement(|phase| {
            host.runtime()
                .on_branch(branch)
                .select(&phase)
                .unwrap()
                .product()
                .selected_commit()
                .clone()
        })
        .unwrap();
    let expiring = authenticate_live_expiring(&host, &scope);
    clock.expire_during_progression(expiring.valid_until());
    let prepared = match programs_publication(&host, prepared, &expiring, &scope) {
        WorthQueryBranchAdoptionPublicationOutcome::AuthorizationDenied { cause, prepared } => {
            expired(&cause);
            prepared
        }
        _ => panic!("lapsed admitted authentication must not publish"),
    };
    clock.assert_serviced();
    let after = host
        .with_host_advancement(|phase| {
            host.runtime()
                .on_branch(branch)
                .select(&phase)
                .unwrap()
                .product()
                .selected_commit()
                .clone()
        })
        .unwrap();
    assert_eq!(before, after);
    assert_eq!(prepared.target(), &target);
    assert_eq!(read_retention(host.runtime(), branch), 7);
    assert!(matches!(
        host.runtime()
            .request(&current, &scope)
            .programs()
            .publish(prepared),
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
    ));
}
fn programs_publication(
    host: &DocumentRetentionRuntime<RetentionProgramP0>,
    prepared: worth_query_host::facade::application_entry::WorthQueryPreparedBranchAdoption,
    principal: &WorthQueryAuthenticatedExternalPrincipal<
        crate::document_retention_model::schema::DocumentRetentionSchema,
    >,
    scope: &WorthQueryRequestScope,
) -> WorthQueryBranchAdoptionPublicationOutcome {
    host.runtime()
        .request(principal, scope)
        .programs()
        .publish(prepared)
}
#[test]
fn expiry_during_progression_preserves_branch_set_prefix_and_pending_owner() {
    let clock = ExpiryClock::default();
    let host = crate::document_retention_model::host::publish_on_first_program_with_trusted_time(
        clock.clone(),
    );
    let a = host.current_world();
    let b = super::branch_set::fork(&host);
    let target = *host
        .supported_program::<RetentionProgramP1>()
        .unwrap()
        .owned_revision();
    let scope = request_scope();
    let current = authenticate_operator(host.installed_schema(), &scope);
    let coverage = host
        .runtime()
        .branches()
        .program_adoption_coverage(&[a, b], NonZeroUsize::new(2).unwrap())
        .unwrap();
    let mut adoption = host
        .runtime()
        .request(&current, &scope)
        .on_branches(coverage, &[a, b])
        .unwrap()
        .programs()
        .adopt(&target)
        .unwrap()
        .prepare(64)
        .unwrap();
    assert!(matches!(
        adoption
            .advance(host.runtime(), &current, &scope)
            .unwrap()
            .unwrap(),
        WorthQueryBranchSetAdoptionProgress::Performed { .. }
    ));
    let expiring = authenticate_live_expiring(&host, &scope);
    clock.expire_during_progression(expiring.valid_until());
    match adoption
        .advance(host.runtime(), &expiring, &scope)
        .err()
        .unwrap()
    {
        WorthQueryBranchSetAdoptionAdvanceDenial::Authorization { branch, cause } => {
            assert_eq!(branch, b);
            expired(&cause);
        }
        other => panic!("expired next-branch admission must preserve pending custody: {other:?}"),
    }
    clock.assert_serviced();
    assert_eq!(adoption.progress().len(), 1);
    assert_eq!(
        host.runtime()
            .request(&current, &scope)
            .on_branch(a)
            .programs()
            .inspect()
            .unwrap()
            .revision(),
        &target
    );
    assert_eq!(
        host.runtime()
            .request(&current, &scope)
            .on_branch(b)
            .programs()
            .inspect()
            .unwrap()
            .revision(),
        host.owned_revision()
    );
    assert!(matches!(
        adoption
            .advance(host.runtime(), &current, &scope)
            .unwrap()
            .unwrap(),
        WorthQueryBranchSetAdoptionProgress::Performed { .. }
    ));
    assert!(adoption.close().is_ok());
}
