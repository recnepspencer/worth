//! `BinCapacity`: a commit-boundary invariant refusing any lot over its bin's
//! capacity with a structured domain refusal.
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

pub const BIN_CAPACITY: u64 = 100;
pub const BIN_CAPACITY_EXCEEDED: &str = "stock.bin_capacity_exceeded";

pub fn capacity_message(lot: u64, quantity: u64) -> String {
    format!("Lot {lot} would hold {quantity} units; its bin holds {BIN_CAPACITY}.")
}

pub struct BinCapacity;
impl ApplicationInvariantMarkerIdentity<StockSchema> for BinCapacity {
    const IDENTIFIER: &'static str = "domain-refusal-bin-capacity";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<StockSchema>,
) -> ApplicationSchemaDeclarationBuilder<StockSchema> {
    let targets = || [ApplicationInvariantScopeTarget::Entity("Lot".into())];
    schema.invariant(ApplicationInvariantDefinition::new(
        BinCapacity::reference(),
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
    setup: &mut WorthQueryApplicationContributionSetup<'_, StockSchema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    setup.invariant(
        BinCapacity::reference(),
        ApplicationInvariantExecutionPoint::CommitBoundary,
        resolve,
    )
}

fn resolve(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, StockSchema>,
) -> Result<Rule, String> {
    Ok(Rule {
        lot: resolver.typed_entity(Lot::reference()).ok_or("lot")?,
        lot_id: resolver.typed_field(LotFacts::lot_id()).ok_or("lot id")?,
        quantity: resolver
            .typed_field(LotFacts::quantity())
            .ok_or("quantity")?,
    })
}

pub struct Rule {
    lot: WorthQueryApplicationInvariantEntityBinding<StockSchema, Lot>,
    lot_id: WorthQueryApplicationInvariantFieldBinding<StockSchema, Lot, u64>,
    quantity: WorthQueryApplicationInvariantFieldBinding<StockSchema, Lot, u64>,
}

type Error = WorthQueryApplicationInvariantError<WorthQueryDomainRefusal>;
type Touched = Vec<WorthQueryApplicationInvariantEntity<StockSchema, Lot>>;

impl WorthQueryApplicationInvariantRule<StockSchema> for Rule {
    type Refusal = WorthQueryDomainRefusal;
    type RefusalBinding = WorthQueryDomainRefusalBinding;
    type Scope = Touched;

    fn prepare_scope(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, StockSchema>,
    ) -> Result<Touched, Error> {
        let proposed = planner.proposed();
        let lots = proposed.touched_entities_of(&self.lot)?;
        for lot in &lots {
            proposed.field(&self.lot_id, lot)?;
            proposed.field(&self.quantity, lot)?;
        }
        Ok(lots)
    }

    fn evaluate(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, StockSchema>,
        lots: &Touched,
    ) -> Result<WorthQueryApplicationInvariantVerdict<WorthQueryDomainRefusal>, Error> {
        let proposed = context.proposed();
        for lot in lots {
            let quantity = proposed.field(&self.quantity, lot)?.unwrap_or_default();
            if quantity > BIN_CAPACITY {
                let id = proposed.field(&self.lot_id, lot)?.unwrap_or_default();
                let refusal = WorthQueryDomainRefusal::new(
                    BIN_CAPACITY_EXCEEDED,
                    &capacity_message(id, quantity),
                )
                .expect("the capacity reason is admissible");
                return Ok(WorthQueryApplicationInvariantVerdict::Violation(refusal));
            }
        }
        Ok(WorthQueryApplicationInvariantVerdict::Pass)
    }
}
