pub use std::sync::Arc;

pub use worth_query_host::facade::{
    declaration::application_query::ApplicationQueryParameterSet, domain, primary_graph, runtime,
};

pub use super::adapters::{
    admitted_identity_adapter, FinancialIntentProjector, FinancialPredicate,
    FinancialQuoteOutputState, QuoteOutputVersionProvider, QuoteToleranceComparator,
};
pub use super::contract::{
    self, CurveRiskNode, PortfolioRiskNode, PortfolioSiblingRiskNode, QuoteRiskNode,
};
pub use super::schema::*;

#[path = "host/access.rs"]
pub mod access;
#[path = "host/amendment.rs"]
pub mod amendment;
#[path = "host/installation.rs"]
pub mod installation;
#[path = "host/seed.rs"]
pub mod seed;

pub use access::{execution, reconstruction};
pub use seed::seed_graph;

pub struct FinancialCourtroomWorld {
    pub application: primary_graph::WorthQueryPrimaryGraphApplicationRuntime<FinancialHostSchema>,
    pub curve_clock: primary_graph::WorthQueryConditionalClockHandle<
        FinancialHostSchema,
        CurveRiskNode,
        worth_query_testkit_host::facade::host::adapters::CourtroomClock,
    >,
    pub quote_clock: primary_graph::WorthQueryConditionalClockHandle<
        FinancialHostSchema,
        QuoteRiskNode,
        worth_query_testkit_host::facade::host::adapters::CourtroomClock,
    >,
    pub portfolio_clock: primary_graph::WorthQueryConditionalClockHandle<
        FinancialHostSchema,
        PortfolioRiskNode,
        worth_query_testkit_host::facade::host::adapters::CourtroomClock,
    >,
    pub sibling_portfolio_clock: primary_graph::WorthQueryConditionalClockHandle<
        FinancialHostSchema,
        PortfolioSiblingRiskNode,
        worth_query_testkit_host::facade::host::adapters::CourtroomClock,
    >,
    pub curve_gate: super::adapters::FinancialGateController,
    pub quote_gate: super::adapters::FinancialGateController,
    pub portfolio_gate: super::adapters::FinancialGateController,
    pub sibling_portfolio_gate: super::adapters::FinancialGateController,
    pub curve_clock_control: worth_query_testkit_host::facade::host::adapters::ClockController,
    pub quote_clock_control: worth_query_testkit_host::facade::host::adapters::ClockController,
    pub portfolio_clock_control: worth_query_testkit_host::facade::host::adapters::ClockController,
    pub sibling_portfolio_clock_control:
        worth_query_testkit_host::facade::host::adapters::ClockController,
    pub quote_output: FinancialQuoteOutputState,
    pub invariant:
        Arc<primary_graph::WorthQueryApplicationInvariantProjectionAuthority<FinancialHostSchema>>,
    pub record_identity: &'static str,
    pub amendment_ordinal: u8,
}

impl FinancialCourtroomWorld {
    pub fn conditional_clock<'world, Node>(
        &'world self,
        handle: &'world primary_graph::WorthQueryConditionalClockHandle<
            FinancialHostSchema,
            Node,
            worth_query_testkit_host::facade::host::adapters::CourtroomClock,
        >,
    ) -> Result<
        primary_graph::WorthQueryConditionalClockObservationPort<
            'world,
            FinancialHostSchema,
            Node,
            worth_query_testkit_host::facade::host::adapters::CourtroomClock,
        >,
        primary_graph::WorthQueryConditionalClockObservationDenial,
    > {
        self.application
            .with_host_advancement(|host_phase| {
                self.application
                    .on_branch(self.application.current_world())
                    .select(&host_phase)
                    .unwrap()
                    .conditional_clock(
                        &host_phase
                            .checked_execution_request_for(self.application.product_runtime())
                            .expect("phase belongs to application"),
                        handle,
                    )
            })
            .expect("installed host call opens")
    }

    pub fn publish_curve() -> Self {
        Self::publish("curve-usd-rates-5y", 4_250, 100, 5_100)
    }

    pub fn publish_quote() -> Self {
        Self::publish("quote-instrument-17", 4_250, 100, 5_100)
    }

    pub fn publish_portfolio() -> Self {
        Self::publish("portfolio-position-17", 4_250, 100, 5_100)
    }

    pub fn record_identity(
        &self,
    ) -> worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts {
        self.application
            .with_host_advancement(|host_phase| {
                self.application
                    .on_branch(self.application.current_world())
                    .select(&host_phase)
                    .expect("the selected product branch remains admitted")
                    .resolve_entity(
                        &host_phase,
                        MarketIdentityField::reference(),
                        self.record_identity.to_string(),
                        &super::adapters::request_scope(),
                        primary_graph::WorthQueryPrincipalResolutionMode::Certification,
                    )
                    .unwrap()
                    .relational_record_identity_parts()
            })
            .expect("installed host call opens")
    }

    pub fn sibling_curve_record_identity(
        &self,
    ) -> worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts {
        self.application
            .with_host_advancement(|host_phase| {
                self.application
                    .on_branch(self.application.current_world())
                    .select(&host_phase)
                    .expect("the selected product branch remains admitted")
                    .resolve_entity(
                        &host_phase,
                        MarketIdentityField::reference(),
                        "curve-usd-rates-10y".to_string(),
                        &super::adapters::request_scope(),
                        primary_graph::WorthQueryPrincipalResolutionMode::Certification,
                    )
                    .unwrap()
                    .relational_record_identity_parts()
            })
            .expect("installed host call opens")
    }
}
