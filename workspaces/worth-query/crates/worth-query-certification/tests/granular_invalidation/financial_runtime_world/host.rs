use std::sync::Arc;

use worth_query_host::facade::{
    declaration::application_query::ApplicationQueryParameterSet, domain, primary_graph, runtime,
};

use super::adapters::{
    admitted_identity_adapter, FinancialIntentProjector, FinancialPredicate,
    FinancialQuoteOutputState, QuoteOutputVersionProvider, QuoteToleranceComparator,
};
use super::contract::{
    self, CurveRiskNode, PortfolioRiskNode, PortfolioSiblingRiskNode, QuoteRiskNode,
};
use super::schema::*;

#[path = "host/access.rs"]
mod access;
#[path = "host/amendment.rs"]
mod amendment;
#[path = "host/installation.rs"]
mod installation;
#[path = "host/seed.rs"]
mod seed;

use access::{execution, reconstruction};
use seed::seed_graph;

pub struct FinancialCourtroomWorld {
    pub application: primary_graph::WorthQueryPrimaryGraphApplicationRuntime<FinancialHostSchema>,
    pub curve_clock: primary_graph::WorthQueryConditionalClockHandle<
        FinancialHostSchema,
        CurveRiskNode,
        crate::adapters::CourtroomClock,
    >,
    pub quote_clock: primary_graph::WorthQueryConditionalClockHandle<
        FinancialHostSchema,
        QuoteRiskNode,
        crate::adapters::CourtroomClock,
    >,
    pub portfolio_clock: primary_graph::WorthQueryConditionalClockHandle<
        FinancialHostSchema,
        PortfolioRiskNode,
        crate::adapters::CourtroomClock,
    >,
    pub sibling_portfolio_clock: primary_graph::WorthQueryConditionalClockHandle<
        FinancialHostSchema,
        PortfolioSiblingRiskNode,
        crate::adapters::CourtroomClock,
    >,
    pub curve_gate: super::adapters::FinancialGateController,
    pub quote_gate: super::adapters::FinancialGateController,
    pub portfolio_gate: super::adapters::FinancialGateController,
    pub sibling_portfolio_gate: super::adapters::FinancialGateController,
    pub curve_clock_control: crate::adapters::ClockController,
    pub quote_clock_control: crate::adapters::ClockController,
    pub portfolio_clock_control: crate::adapters::ClockController,
    pub sibling_portfolio_clock_control: crate::adapters::ClockController,
    pub quote_output: FinancialQuoteOutputState,
    invariant:
        Arc<primary_graph::WorthQueryApplicationInvariantProjectionAuthority<FinancialHostSchema>>,
    record_identity: &'static str,
    amendment_ordinal: u8,
}

impl FinancialCourtroomWorld {
    pub fn conditional_clock<'world, Node>(
        &'world self,
        handle: &'world primary_graph::WorthQueryConditionalClockHandle<
            FinancialHostSchema,
            Node,
            crate::adapters::CourtroomClock,
        >,
    ) -> Result<
        primary_graph::WorthQueryConditionalClockObservationPort<
            'world,
            FinancialHostSchema,
            Node,
            crate::adapters::CourtroomClock,
        >,
        primary_graph::WorthQueryConditionalClockObservationDenial,
    > {
        self.application
            .on_branch(self.application.current_world())
            .select()
            .unwrap()
            .conditional_clock(handle)
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
            .on_branch(self.application.current_world())
            .select()
            .expect("the selected product branch remains admitted")
            .resolve_entity(
                MarketIdentityField::reference(),
                self.record_identity.to_string(),
                &super::adapters::request_scope(),
                primary_graph::WorthQueryPrincipalResolutionMode::Certification,
            )
            .unwrap()
            .relational_record_identity_parts()
    }

    pub fn sibling_curve_record_identity(
        &self,
    ) -> worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts {
        self.application
            .on_branch(self.application.current_world())
            .select()
            .expect("the selected product branch remains admitted")
            .resolve_entity(
                MarketIdentityField::reference(),
                "curve-usd-rates-10y".to_string(),
                &super::adapters::request_scope(),
                primary_graph::WorthQueryPrincipalResolutionMode::Certification,
            )
            .unwrap()
            .relational_record_identity_parts()
    }
}
