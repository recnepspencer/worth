pub use worth_query_host::facade::primary_graph;

pub use super::super::adapters::{admitted_identity_adapter, block_on, request_scope};
pub use super::super::schema::*;
pub use super::FinancialCourtroomWorld;

#[derive(Default)]
pub struct MarketAmendment {
    pub revision: u64,
    pub risk_value: u64,
    pub curve_rate: Option<u64>,
    pub quote_mid: Option<u64>,
    pub portfolio_value: Option<u64>,
    pub portfolio_desk: Option<String>,
    pub portfolio_rank: Option<u64>,
}

impl FinancialCourtroomWorld {
    pub fn amend_curve(&mut self, revision: u64, curve_rate: u64, risk_value: u64) {
        self.commit_amendment(MarketAmendment {
            revision,
            risk_value,
            curve_rate: Some(curve_rate),
            ..MarketAmendment::default()
        });
    }

    pub fn amend_quote(&mut self, revision: u64, quote_mid: u64, risk_value: u64) {
        self.commit_amendment(MarketAmendment {
            revision,
            risk_value,
            quote_mid: Some(quote_mid),
            ..MarketAmendment::default()
        });
        self.quote_output.set(quote_mid);
    }

    pub fn amend_portfolio_value(&mut self, revision: u64, value: u64) {
        self.commit_amendment_for(
            self.record_identity,
            MarketAmendment {
                revision,
                risk_value: value,
                portfolio_value: Some(value),
                ..MarketAmendment::default()
            },
        );
    }

    pub fn amend_sibling_portfolio_value(&mut self, revision: u64, value: u64) {
        self.commit_amendment_for(
            "curve-usd-rates-10y",
            MarketAmendment {
                revision,
                risk_value: value,
                portfolio_value: Some(value),
                ..MarketAmendment::default()
            },
        );
    }

    pub fn amend_portfolio_desk(&mut self, revision: u64, desk: &str) {
        self.commit_amendment(MarketAmendment {
            revision,
            risk_value: 5_100,
            portfolio_desk: Some(desk.to_owned()),
            ..MarketAmendment::default()
        });
    }

    pub fn amend_portfolio_rank(&mut self, revision: u64, rank: u64) {
        self.commit_amendment(MarketAmendment {
            revision,
            risk_value: 5_100,
            portfolio_rank: Some(rank),
            ..MarketAmendment::default()
        });
    }

    pub fn commit_amendment(&mut self, amendment: MarketAmendment) {
        self.commit_amendment_for(self.record_identity, amendment);
    }

    pub fn commit_amendment_for(&mut self, record_identity: &str, amendment: MarketAmendment) {
        let schema = self.application.installed_schema();
        let authentication = admitted_identity_adapter(schema);
        let request = request_scope();
        let external = block_on(authentication.authenticate((), &request)).unwrap();
        let principal = self
            .application
            .with_host_advancement(|host_phase| {
                self.application
                    .on_branch(self.application.current_world())
                    .select(&host_phase)
                    .expect("the selected product branch remains admitted")
                    .resolve_authenticated_principal(
                        &host_phase,
                        &schema
                            .principal_binding(FinancialPrincipalBinding::reference())
                            .unwrap(),
                        &external,
                        &request,
                        primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap()
            })
            .expect("installed host call opens");
        let record = self
            .application
            .with_host_advancement(|host_phase| {
                self.application
                    .on_branch(self.application.current_world())
                    .select(&host_phase)
                    .expect("the selected product branch remains admitted")
                    .resolve_entity(
                        &host_phase,
                        MarketIdentityField::at("MarketIdentityField"),
                        record_identity.to_string(),
                        &request,
                        primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap()
            })
            .expect("installed host call opens");
        let admission = self
            .application
            .with_host_advancement(|host_phase| {
                self.application
                    .on_branch(self.application.current_world())
                    .select(&host_phase)
                    .unwrap()
                    .authorize_operation(
                        &host_phase,
                        &principal,
                        &record,
                        &schema
                            .installed_operation(AmendMarket::reference())
                            .unwrap(),
                        Default::default(),
                        &request,
                    )
                    .unwrap()
            })
            .expect("installed host call opens");
        let (_, projection, _) = self
            .application
            .with_host_advancement(|host_phase| {
                self
            .invariant
            .project_admitted_operation(
                &host_phase, &admission,
                |reader, scope| {
                    reader
                        .decision_field(scope, MarketRevisionField::at("MarketRevisionField"))
                        .unwrap();
                    reader
                        .decision_field(scope, MarketDueField::at("MarketDueField"))
                        .unwrap();
                    reader
                        .decision_field(scope, MarketLifecycleField::at("MarketLifecycleField"))
                        .unwrap();
                    reader
                        .decision_field(scope, MarketInputField::at("MarketInputField"))
                        .unwrap();
                    reader
                        .decision_field(scope, CurveZeroRateField::at("CurveZeroRateField"))
                        .unwrap();
                    reader
                        .decision_field(scope, QuoteMidField::at("QuoteMidField"))
                        .unwrap();
                    reader
                        .decision_field(scope, RiskValueField::at("RiskValueField"))
                        .unwrap();
                    reader
                        .decision_field(scope, PortfolioValueField::at("PortfolioValueField"))
                        .unwrap();
                    reader
                        .decision_field(scope, PortfolioDeskField::at("PortfolioDeskField"))
                        .unwrap();
                    reader
                        .decision_field(scope, PortfolioRankField::at("PortfolioRankField"))
                        .unwrap();
                },
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
            .into_parts()
            })
            .expect("installed host call opens");
        let finished = self
            .application
            .with_host_advancement(|host_phase| {
                let reads = self
                    .application
                    .begin_projected_application_read_attempt(
                        &host_phase,
                        admission,
                        projection,
                        worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                    )
                    .unwrap();
                let mut effects = reads
                    .complete_projected_dependencies(
                        &host_phase,
                        worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                    )
                    .unwrap()
                    .begin_effect_program();
                let record = effects.existing_entity(&record).unwrap();
                effects
                    .write_field(
                        &record,
                        MarketRevisionField::at("MarketRevisionField"),
                        amendment.revision,
                    )
                    .unwrap();
                effects
                    .write_field(
                        &record,
                        MarketDueField::at("MarketDueField"),
                        11 + u64::from(self.amendment_ordinal),
                    )
                    .unwrap();
                effects
                    .write_field(
                        &record,
                        MarketLifecycleField::at("MarketLifecycleField"),
                        "active".to_owned(),
                    )
                    .unwrap();
                effects
                    .write_field(
                        &record,
                        MarketInputField::at("MarketInputField"),
                        amendment.risk_value.to_string(),
                    )
                    .unwrap();
                if let Some(value) = amendment.curve_rate {
                    effects
                        .write_field(&record, CurveZeroRateField::at("CurveZeroRateField"), value)
                        .unwrap();
                }
                if let Some(value) = amendment.quote_mid {
                    effects
                        .write_field(&record, QuoteMidField::at("QuoteMidField"), value)
                        .unwrap();
                }
                if let Some(value) = amendment.portfolio_value {
                    effects
                        .write_field(&record, PortfolioValueField::at("PortfolioValueField"), value)
                        .unwrap();
                }
                if let Some(value) = amendment.portfolio_desk {
                    effects
                        .write_field(&record, PortfolioDeskField::at("PortfolioDeskField"), value)
                        .unwrap();
                }
                if let Some(value) = amendment.portfolio_rank {
                    effects
                        .write_field(&record, PortfolioRankField::at("PortfolioRankField"), value)
                        .unwrap();
                }
                effects.finish().unwrap()
            })
            .expect("installed host call opens");
        self.amendment_ordinal = self.amendment_ordinal.saturating_add(1);
        let idempotency =
            primary_graph::WorthQueryApplicationIdempotencyBinding::for_host_commit::<
                FinancialHostSchema,
                AmendMarket,
                _,
                _,
            >(&self.amendment_ordinal, &self.amendment_ordinal)
            .unwrap();
        match self.application.compare_and_commit_application(
            finished,
            idempotency,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        ) {
            primary_graph::WorthQueryApplicationCommitOutcome::Committed(_) => {}
            outcome => panic!("financial market amendment was not committed: {outcome:?}"),
        }
    }
}
