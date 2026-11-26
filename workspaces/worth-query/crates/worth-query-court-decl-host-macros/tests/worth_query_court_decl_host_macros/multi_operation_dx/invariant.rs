//! `CounterCeiling`: a commit-boundary invariant, declared and installed by
//! one `invariant(..)` contribution member, refusing any counter above the
//! ceiling with a structured domain refusal.
use worth_query_decl::facade::application_schema::*;
use worth_query_host::facade::application_invariants::{
    WorthQueryApplicationInvariantContext, WorthQueryApplicationInvariantEntity,
    WorthQueryApplicationInvariantEntityBinding, WorthQueryApplicationInvariantError,
    WorthQueryApplicationInvariantFieldBinding, WorthQueryApplicationInvariantRule,
    WorthQueryApplicationInvariantSchemaResolver, WorthQueryApplicationInvariantScopePlanner,
    WorthQueryApplicationInvariantVerdict, WorthQueryDomainRefusal, WorthQueryDomainRefusalBinding,
};

use super::schema::*;

pub const CEILING: u64 = 1_000;
pub const CEILING_EXCEEDED: &str = "board.ceiling_exceeded";

pub fn ceiling_message(counter: u64, value: u64) -> String {
    format!("Counter {counter} would read {value}; counters stop at {CEILING}.")
}

pub struct CounterCeiling;
impl ApplicationInvariantMarkerIdentity<BoardSchema> for CounterCeiling {
    const IDENTIFIER: &'static str = "multi-operation-counter-ceiling";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}

/// The invariant's declaration, evaluated where the contribution declares it
/// and again where it installs the rule.
pub fn definition() -> ApplicationInvariantDefinition<BoardSchema, CounterCeiling> {
    let targets = || [ApplicationInvariantScopeTarget::Entity("Counter".into())];
    ApplicationInvariantDefinition::new(
        CounterCeiling::reference(),
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
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, BoardSchema>,
) -> Result<Rule, String> {
    Ok(Rule {
        counter: resolver
            .typed_entity(Counter::reference())
            .ok_or("counter")?,
        id: resolver
            .typed_field(CounterFacts::counter_id())
            .ok_or("counter id")?,
        value: resolver.typed_field(CounterFacts::value()).ok_or("value")?,
    })
}

pub struct Rule {
    counter: WorthQueryApplicationInvariantEntityBinding<BoardSchema, Counter>,
    id: WorthQueryApplicationInvariantFieldBinding<BoardSchema, Counter, u64>,
    value: WorthQueryApplicationInvariantFieldBinding<BoardSchema, Counter, u64>,
}

type Error = WorthQueryApplicationInvariantError<WorthQueryDomainRefusal>;
type Touched = Vec<WorthQueryApplicationInvariantEntity<BoardSchema, Counter>>;

impl WorthQueryApplicationInvariantRule<BoardSchema> for Rule {
    type Refusal = WorthQueryDomainRefusal;
    type RefusalBinding = WorthQueryDomainRefusalBinding;
    type Scope = Touched;

    fn prepare_scope(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, BoardSchema>,
    ) -> Result<Touched, Error> {
        let proposed = planner.proposed();
        let counters = proposed.touched_entities_of(&self.counter)?;
        for counter in &counters {
            proposed.field(&self.id, counter)?;
            proposed.field(&self.value, counter)?;
        }
        Ok(counters)
    }

    fn evaluate(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, BoardSchema>,
        counters: &Touched,
    ) -> Result<WorthQueryApplicationInvariantVerdict<WorthQueryDomainRefusal>, Error> {
        let proposed = context.proposed();
        for counter in counters {
            let value = proposed.field(&self.value, counter)?.unwrap_or_default();
            if value > CEILING {
                let id = proposed.field(&self.id, counter)?.unwrap_or_default();
                let refusal =
                    WorthQueryDomainRefusal::new(CEILING_EXCEEDED, &ceiling_message(id, value))
                        .expect("the ceiling reason is admissible");
                return Ok(WorthQueryApplicationInvariantVerdict::Violation(refusal));
            }
        }
        Ok(WorthQueryApplicationInvariantVerdict::Pass)
    }
}
