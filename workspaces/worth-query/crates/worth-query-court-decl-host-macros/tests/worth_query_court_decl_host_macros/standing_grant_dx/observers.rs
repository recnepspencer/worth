//! The two observers every W10A court compares: a Host session's capability
//! eligibility and capability admission on the selected branch head, for one
//! site role. They must agree on the outcome and on its policy cause.
use worth_query_host::facade::runtime::time::ControlledTime;
use worth_query_decl::facade::application_capability::ApplicationCapabilityRef;
use worth_query_decl::facade::application_schema::ApplicationOperationRef;
use worth_query_host::facade::{
    eligibility::{
        WorthQueryCapabilityEligibility as Verdict, WorthQueryCapabilityEligibilityQuestion,
    },
    local_calendar::WorthQueryTimeZone,
    primary_graph::{
        WorthQueryApplicationAuthorizationExplanationCause as Cause,
        WorthQueryOperationAuthorizationDenial, WorthQueryPrincipalResolutionMode,
    },
};

use super::grants::{SiteRequest, SiteRequestInput};
use super::november;
use super::program::{caller, open, QmsHome};
use super::schema::{Desktop, MemberBinding, QmsSchema};

/// One installed site role: its capability and the operation exercising it.
pub struct SiteRole<Cap, Op> {
    pub capability: ApplicationCapabilityRef<QmsSchema, Cap>,
    pub operation: ApplicationOperationRef<QmsSchema, Op, SiteRequest>,
}

impl<Cap, Op> Clone for SiteRole<Cap, Op> {
    fn clone(&self) -> Self {
        Self {
            capability: self.capability,
            operation: self.operation,
        }
    }
}

pub fn home(path: &std::path::Path, clock: &std::sync::Arc<ControlledTime>) -> QmsHome {
    let denver = WorthQueryTimeZone::from_tz_variable("MST7MDT,M3.2.0,M11.1.0").unwrap();
    open(path, clock.clone(), Some(denver), november())
        .unwrap_or_else(|denial| panic!("QMS Home opens: {denial}"))
}

impl<Cap, Op> SiteRole<Cap, Op>
where
    Cap: 'static,
    Op: worth_query_decl::facade::application_schema::ApplicationOperationMarkerIdentity<
            QmsSchema,
            InputBinding = SiteRequestInput,
        > + 'static,
    SiteRequest: worth_query_decl::facade::application_capability::ApplicationCapabilityRequest<
        QmsSchema,
        Cap,
    >,
{
    pub fn question(
        &self,
        site: &str,
    ) -> WorthQueryCapabilityEligibilityQuestion<QmsSchema, Desktop, Cap, Op, SiteRequest> {
        let request = SiteRequest {
            site: site.to_owned(),
        };
        WorthQueryCapabilityEligibilityQuestion::of_request(
            Desktop,
            self.capability,
            self.operation,
            &request,
        )
        .expect("the site request projects")
    }

    /// `name`'s own verdict on `site`, asked through a Host session.
    pub fn eligibility(&self, home: &QmsHome, name: &str, site: &str) -> Verdict {
        home.session(caller(home, name).principal)
            .eligible_for_capability(&self.question(site))
            .unwrap_or_else(|denial| panic!("the role question is answered: {denial:?}"))
    }

    /// `name`'s admission on `site`: `Ok` admits, else the denial.
    pub fn admission(
        &self,
        home: &QmsHome,
        name: &str,
        site: &str,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        let app = home.runtime().runtime();
        let who = caller(home, name);
        let schema = app.installed_schema();
        let capability = schema.capability(self.capability, self.operation).unwrap();
        let binding = schema
            .principal_binding(MemberBinding::reference())
            .unwrap();
        app.with_application_advancement(&who.scope, |phase| {
            let selected = app.on_branch(app.current_world()).select(&phase).unwrap();
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &binding,
                    &who.principal,
                    &who.scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let request = SiteRequest {
                site: site.to_owned(),
            };
            selected
                .admit_capability_access(&phase, &principal, &capability, request, &who.scope)
                .map(drop)
        })
        .unwrap()
    }

    /// Asserts both observers agree, returning the shared outcome: `None`
    /// when both allow, else the one policy cause both report.
    pub fn agreed(&self, home: &QmsHome, name: &str, site: &str) -> Option<Cause> {
        let verdict = self.eligibility(home, name, site);
        match self.admission(home, name, site) {
            Ok(()) => {
                assert_eq!(verdict, Verdict::Eligible, "admission admits {name}");
                None
            }
            Err(denial) => {
                let cause = denial.explanation_cause();
                assert!(
                    cause.is_some(),
                    "a policy refusal explains itself: {denial:?}"
                );
                assert_eq!(verdict.denial_cause(), cause, "{name}: {denial:?}");
                cause
            }
        }
    }
}
