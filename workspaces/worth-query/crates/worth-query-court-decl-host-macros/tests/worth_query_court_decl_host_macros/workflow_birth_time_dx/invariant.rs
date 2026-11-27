//! `BornAtAdmission`: a commit-boundary invariant refusing any touched
//! request whose `created_at` is not the commit's own trusted admission
//! instant. It fails closed when trusted time is unavailable, so a committed
//! birth proves the body's stamp and the invariant's instant are one value.
use worth_query_decl::facade::application_schema::*;
use worth_query_host::facade::application_invariants::{
    WorthQueryApplicationInvariantContext, WorthQueryApplicationInvariantEntity,
    WorthQueryApplicationInvariantEntityBinding, WorthQueryApplicationInvariantError,
    WorthQueryApplicationInvariantFieldBinding, WorthQueryApplicationInvariantRule,
    WorthQueryApplicationInvariantSchemaResolver, WorthQueryApplicationInvariantScopePlanner,
    WorthQueryApplicationInvariantVerdict, WorthQueryDomainRefusal, WorthQueryDomainRefusalBinding,
};

use super::schema::*;

pub struct BornAtAdmission;
impl ApplicationInvariantMarkerIdentity<BirthSchema> for BornAtAdmission {
    const IDENTIFIER: &'static str = "workflow-birth-time-born-at-admission";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}

pub fn definition() -> ApplicationInvariantDefinition<BirthSchema, BornAtAdmission> {
    let targets = || [ApplicationInvariantScopeTarget::Entity("Request".into())];
    ApplicationInvariantDefinition::new(
        BornAtAdmission::reference(),
        ApplicationInvariantExecutionPoint::CommitBoundary,
        RelationalInvariantWorkBudget::new(None),
        ApplicationInvariantOperationalContract::new(
            ApplicationInvariantEnforcement::BlockCommit,
            [ApplicationInvariantGroup::SchemaCompliance],
            targets(),
            targets(),
            "primary-relational-provider",
            ApplicationInvariantCostPosture::Touched,
        ),
    )
}

pub fn resolve(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, BirthSchema>,
) -> Result<Rule, String> {
    Ok(Rule {
        request: resolver
            .typed_entity(Request::reference())
            .ok_or("request")?,
        created_at: resolver
            .typed_field(RequestFacts::created_at())
            .ok_or("created")?,
    })
}

pub struct Rule {
    request: WorthQueryApplicationInvariantEntityBinding<BirthSchema, Request>,
    created_at:
        WorthQueryApplicationInvariantFieldBinding<BirthSchema, Request, CanonicalTimestamp>,
}

type Error = WorthQueryApplicationInvariantError<WorthQueryDomainRefusal>;
type Verdict = WorthQueryApplicationInvariantVerdict<WorthQueryDomainRefusal>;
type Touched = Vec<WorthQueryApplicationInvariantEntity<BirthSchema, Request>>;

fn violation(code: &str, message: &str) -> Verdict {
    Verdict::Violation(WorthQueryDomainRefusal::new(code, message).expect("an admissible reason"))
}

impl WorthQueryApplicationInvariantRule<BirthSchema> for Rule {
    type Refusal = WorthQueryDomainRefusal;
    type RefusalBinding = WorthQueryDomainRefusalBinding;
    type Scope = Touched;

    fn prepare_scope(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, BirthSchema>,
    ) -> Result<Touched, Error> {
        let proposed = planner.proposed();
        let requests = proposed.touched_entities_of(&self.request)?;
        for request in &requests {
            proposed.field(&self.created_at, request)?;
        }
        Ok(requests)
    }

    fn evaluate(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, BirthSchema>,
        requests: &Touched,
    ) -> Result<Verdict, Error> {
        let Ok(admitted_at) = context.admitted_at() else {
            return Ok(violation(
                "trusted_time_unavailable",
                "Trusted time is unavailable.",
            ));
        };
        let proposed = context.proposed();
        for request in requests {
            if proposed.field(&self.created_at, request)? != Some(admitted_at) {
                return Ok(violation(
                    "admission_time_mismatch",
                    "A request was born at another instant.",
                ));
            }
        }
        Ok(Verdict::Pass)
    }
}
