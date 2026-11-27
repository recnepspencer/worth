//! `StampedAtAdmission`: a commit-boundary invariant refusing any touched
//! ticket whose `created_at`, child entry whose `opened_at`, or desk whose
//! `noted_at`, is not the commit's
//! own trusted admission instant. It fails closed without trusted time.
use worth_query_decl::facade::application_schema::*;
use worth_query_host::facade::application_contribution::WorthQueryApplicationContributionSetup;
use worth_query_host::facade::application_invariants::{
    WorthQueryApplicationInvariantContext, WorthQueryApplicationInvariantEntity,
    WorthQueryApplicationInvariantEntityBinding, WorthQueryApplicationInvariantError,
    WorthQueryApplicationInvariantFieldBinding, WorthQueryApplicationInvariantRule,
    WorthQueryApplicationInvariantSchemaResolver, WorthQueryApplicationInvariantScopePlanner,
    WorthQueryApplicationInvariantVerdict, WorthQueryDomainRefusal, WorthQueryDomainRefusalBinding,
};
use worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial;

use super::schema::*;

pub const UNAVAILABLE: &str = "trusted_time_unavailable";
pub const MISMATCH: &str = "admission_time_mismatch";

pub struct StampedAtAdmission;
impl ApplicationInvariantMarkerIdentity<DeskSchema> for StampedAtAdmission {
    const IDENTIFIER: &'static str = "body-admission-time-stamped-at-admission";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<DeskSchema>,
) -> ApplicationSchemaDeclarationBuilder<DeskSchema> {
    let targets = || {
        [
            ApplicationInvariantScopeTarget::Entity("Ticket".into()),
            ApplicationInvariantScopeTarget::Entity("Entry".into()),
            ApplicationInvariantScopeTarget::Entity("Desk".into()),
        ]
    };
    schema.invariant(ApplicationInvariantDefinition::new(
        StampedAtAdmission::reference(),
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
    ))
}

pub fn install(
    setup: &mut WorthQueryApplicationContributionSetup<'_, DeskSchema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    setup.invariant(
        StampedAtAdmission::reference(),
        ApplicationInvariantExecutionPoint::CommitBoundary,
        resolve,
    )
}

fn resolve(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, DeskSchema>,
) -> Result<Rule, String> {
    Ok(Rule {
        ticket: resolver.typed_entity(Ticket::reference()).ok_or("ticket")?,
        created_at: resolver
            .typed_field(TicketFacts::created_at())
            .ok_or("created")?,
        entry: resolver.typed_entity(Entry::reference()).ok_or("entry")?,
        opened_at: resolver
            .typed_field(EntryFacts::opened_at())
            .ok_or("opened")?,
        desk: resolver.typed_entity(Desk::reference()).ok_or("desk")?,
        noted_at: resolver.typed_field(DeskFacts::noted_at()).ok_or("noted")?,
    })
}

pub struct Rule {
    ticket: WorthQueryApplicationInvariantEntityBinding<DeskSchema, Ticket>,
    created_at: WorthQueryApplicationInvariantFieldBinding<DeskSchema, Ticket, CanonicalTimestamp>,
    entry: WorthQueryApplicationInvariantEntityBinding<DeskSchema, Entry>,
    opened_at: WorthQueryApplicationInvariantFieldBinding<DeskSchema, Entry, CanonicalTimestamp>,
    desk: WorthQueryApplicationInvariantEntityBinding<DeskSchema, Desk>,
    noted_at: WorthQueryApplicationInvariantFieldBinding<DeskSchema, Desk, CanonicalTimestamp>,
}

type Error = WorthQueryApplicationInvariantError<WorthQueryDomainRefusal>;
type Verdict = WorthQueryApplicationInvariantVerdict<WorthQueryDomainRefusal>;

pub struct Touched {
    tickets: Vec<WorthQueryApplicationInvariantEntity<DeskSchema, Ticket>>,
    entries: Vec<WorthQueryApplicationInvariantEntity<DeskSchema, Entry>>,
    desks: Vec<WorthQueryApplicationInvariantEntity<DeskSchema, Desk>>,
}

fn violation(code: &str, message: &str) -> Verdict {
    Verdict::Violation(WorthQueryDomainRefusal::new(code, message).expect("an admissible reason"))
}

impl WorthQueryApplicationInvariantRule<DeskSchema> for Rule {
    type Refusal = WorthQueryDomainRefusal;
    type RefusalBinding = WorthQueryDomainRefusalBinding;
    type Scope = Touched;

    fn prepare_scope(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, DeskSchema>,
    ) -> Result<Touched, Error> {
        let proposed = planner.proposed();
        let tickets = proposed.touched_entities_of(&self.ticket)?;
        for ticket in &tickets {
            proposed.field(&self.created_at, ticket)?;
        }
        let entries = proposed.touched_entities_of(&self.entry)?;
        for entry in &entries {
            proposed.field(&self.opened_at, entry)?;
        }
        let desks = proposed.touched_entities_of(&self.desk)?;
        for desk in &desks {
            proposed.field(&self.noted_at, desk)?;
        }
        Ok(Touched {
            tickets,
            entries,
            desks,
        })
    }

    fn evaluate(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, DeskSchema>,
        touched: &Touched,
    ) -> Result<Verdict, Error> {
        let Ok(admitted_at) = context.admitted_at() else {
            return Ok(violation(UNAVAILABLE, "Trusted time is unavailable."));
        };
        let proposed = context.proposed();
        for ticket in &touched.tickets {
            if proposed.field(&self.created_at, ticket)? != Some(admitted_at) {
                return Ok(violation(MISMATCH, "A ticket was born at another instant."));
            }
        }
        for entry in &touched.entries {
            if proposed.field(&self.opened_at, entry)? != Some(admitted_at) {
                return Ok(violation(
                    MISMATCH,
                    "An entry was opened at another instant.",
                ));
            }
        }
        for desk in &touched.desks {
            // The seeded desk carries epoch zero until a handler notes it.
            let noted = proposed.field(&self.noted_at, desk)?;
            if noted.is_some_and(|at| at.micros_since_unix_epoch != 0 && at != admitted_at) {
                return Ok(violation(MISMATCH, "A desk was noted at another instant."));
            }
        }
        Ok(Verdict::Pass)
    }
}
