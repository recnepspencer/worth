//! `TrustedDates`: a commit-boundary invariant judging each touched lot's
//! caller-supplied dates against the trusted admission instant, converted to
//! the Mountain civil date, and checking the handler's stamp is that instant.
use worth_query_decl::facade::application_schema::*;
use worth_query_host::facade::application_contribution::WorthQueryApplicationContributionSetup;
use worth_query_host::facade::application_invariants::{
    WorthQueryApplicationInvariantContext, WorthQueryApplicationInvariantEntity,
    WorthQueryApplicationInvariantEntityBinding, WorthQueryApplicationInvariantError,
    WorthQueryApplicationInvariantFieldBinding, WorthQueryApplicationInvariantRule,
    WorthQueryApplicationInvariantSchemaResolver, WorthQueryApplicationInvariantScopePlanner,
    WorthQueryApplicationInvariantVerdict, WorthQueryDomainRefusal, WorthQueryDomainRefusalBinding,
};
use worth_query_host::facade::local_calendar::WorthQueryTimeZone;
use worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial;

use super::schema::*;

pub const BACKDATED: &str = "backdated_effective_date";
pub const EXPIRED: &str = "lot_expired";
pub const MISMATCH: &str = "admission_time_mismatch";

pub fn backdated_message(lot: u64) -> String {
    format!("Lot {lot} takes effect before today in Mountain time.")
}

pub fn expired_message(lot: u64) -> String {
    format!("Lot {lot} expired before today in Mountain time.")
}

/// US Mountain time as its POSIX rule, so the court reads no zone file; an
/// application would load `WorthQueryTimeZone::from_tz_variable(":America/Denver")`.
pub fn mountain() -> WorthQueryTimeZone {
    WorthQueryTimeZone::from_tz_variable("MST7MDT,M3.2.0,M11.1.0")
        .expect("the Mountain rule parses")
}

pub struct TrustedDates;
impl ApplicationInvariantMarkerIdentity<DatedSchema> for TrustedDates {
    const IDENTIFIER: &'static str = "admission-time-trusted-dates";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<DatedSchema>,
) -> ApplicationSchemaDeclarationBuilder<DatedSchema> {
    let targets = || [ApplicationInvariantScopeTarget::Entity("Lot".into())];
    schema.invariant(ApplicationInvariantDefinition::new(
        TrustedDates::reference(),
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
    setup: &mut WorthQueryApplicationContributionSetup<'_, DatedSchema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    setup.invariant(
        TrustedDates::reference(),
        ApplicationInvariantExecutionPoint::CommitBoundary,
        resolve,
    )
}

fn resolve(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, DatedSchema>,
) -> Result<Rule, String> {
    Ok(Rule {
        lot: resolver.typed_entity(Lot::reference()).ok_or("lot")?,
        lot_id: resolver.typed_field(LotFacts::lot_id()).ok_or("lot id")?,
        effective_on: resolver
            .typed_field(LotFacts::effective_on())
            .ok_or("effective")?,
        expires_on: resolver
            .typed_field(LotFacts::expires_on())
            .ok_or("expires")?,
        stamped_at: resolver
            .typed_field(LotFacts::stamped_at())
            .ok_or("stamped")?,
        zone: mountain(),
    })
}

pub struct Rule {
    lot: WorthQueryApplicationInvariantEntityBinding<DatedSchema, Lot>,
    lot_id: WorthQueryApplicationInvariantFieldBinding<DatedSchema, Lot, u64>,
    effective_on: WorthQueryApplicationInvariantFieldBinding<DatedSchema, Lot, CanonicalDate>,
    expires_on: WorthQueryApplicationInvariantFieldBinding<DatedSchema, Lot, CanonicalDate>,
    stamped_at: WorthQueryApplicationInvariantFieldBinding<DatedSchema, Lot, CanonicalTimestamp>,
    zone: WorthQueryTimeZone,
}

type Error = WorthQueryApplicationInvariantError<WorthQueryDomainRefusal>;
type Touched = Vec<WorthQueryApplicationInvariantEntity<DatedSchema, Lot>>;
type Verdict = WorthQueryApplicationInvariantVerdict<WorthQueryDomainRefusal>;

fn violation(code: &str, message: &str) -> Verdict {
    Verdict::Violation(WorthQueryDomainRefusal::new(code, message).expect("an admissible reason"))
}

impl WorthQueryApplicationInvariantRule<DatedSchema> for Rule {
    type Refusal = WorthQueryDomainRefusal;
    type RefusalBinding = WorthQueryDomainRefusalBinding;
    type Scope = Touched;

    fn prepare_scope(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, DatedSchema>,
    ) -> Result<Touched, Error> {
        let proposed = planner.proposed();
        let lots = proposed.touched_entities_of(&self.lot)?;
        for lot in &lots {
            proposed.field(&self.lot_id, lot)?;
            proposed.field(&self.effective_on, lot)?;
            proposed.field(&self.expires_on, lot)?;
            proposed.field(&self.stamped_at, lot)?;
        }
        Ok(lots)
    }

    fn evaluate(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, DatedSchema>,
        lots: &Touched,
    ) -> Result<Verdict, Error> {
        // Fail closed: a commit without trusted time cannot prove its dates.
        let Ok(admitted_at) = context.admitted_at() else {
            return Ok(violation(
                "trusted_time_unavailable",
                "Trusted time is unavailable.",
            ));
        };
        let Ok(today) = self.zone.date_at(admitted_at) else {
            return Ok(violation(
                "local_date_unavailable",
                "Today's date is unavailable.",
            ));
        };
        let proposed = context.proposed();
        for lot in lots {
            let id = proposed.field(&self.lot_id, lot)?.unwrap_or_default();
            // An unstamped (seeded, epoch-zero) lot carries no handler stamp.
            let stamped = proposed.field(&self.stamped_at, lot)?;
            if stamped.is_some_and(|at| at.micros_since_unix_epoch != 0 && at != admitted_at) {
                return Ok(violation(MISMATCH, "The handler stamped another instant."));
            }
            if proposed
                .field(&self.effective_on, lot)?
                .is_none_or(|on| on < today)
            {
                return Ok(violation(BACKDATED, &backdated_message(id)));
            }
            if proposed
                .field(&self.expires_on, lot)?
                .is_none_or(|on| on < today)
            {
                return Ok(violation(EXPIRED, &expired_message(id)));
            }
        }
        Ok(Verdict::Pass)
    }
}
