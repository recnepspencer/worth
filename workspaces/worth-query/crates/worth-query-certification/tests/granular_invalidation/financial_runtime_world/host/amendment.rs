use worth_query_host::facade::primary_graph;

use super::super::adapters::{admitted_identity_adapter, block_on, request_scope};
use super::super::schema::*;
use super::FinancialCourtroomWorld;

#[derive(Default)]
struct MarketAmendment {
    revision: u64,
    risk_value: u64,
    curve_rate: Option<u64>,
    quote_mid: Option<u64>,
    portfolio_value: Option<u64>,
    portfolio_desk: Option<String>,
    portfolio_rank: Option<u64>,
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

    fn commit_amendment(&mut self, amendment: MarketAmendment) {
        self.commit_amendment_for(self.record_identity, amendment);
    }

    fn commit_amendment_for(&mut self, record_identity: &str, amendment: MarketAmendment) {
        let schema = self.application.installed_schema();
        let authentication = admitted_identity_adapter(schema);
        let request = request_scope();
        let external = block_on(authentication.authenticate((), &request)).unwrap();
        let principal = self
            .application
            .select_product_branch(self.application.product_runtime().default_branch())
            .expect("the selected product branch remains admitted")
            .resolve_authenticated_principal(
                &schema
                    .principal_binding(FinancialPrincipalBinding::reference())
                    .unwrap(),
                external,
                &request,
                primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let record = self
            .application
            .select_product_branch(self.application.product_runtime().default_branch())
            .expect("the selected product branch remains admitted")
            .resolve_entity(
                MarketIdentityField::reference(),
                record_identity.to_string(),
                &request,
                primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let admission = self
            .application
            .select_product_branch(self.application.product_runtime().default_branch())
            .unwrap()
            .authorize_operation(
                &principal,
                &record,
                &schema
                    .installed_operation(AmendMarket::reference())
                    .unwrap(),
                Default::default(),
                &request,
            )
            .unwrap();
        let (_, projection, _) = self
            .invariant
            .project_admitted_operation(&admission, |reader, scope| {
                reader
                    .decision_field(scope, MarketRevisionField::reference())
                    .unwrap();
                reader
                    .decision_field(scope, MarketDueField::reference())
                    .unwrap();
                reader
                    .decision_field(scope, MarketLifecycleField::reference())
                    .unwrap();
                reader
                    .decision_field(scope, MarketInputField::reference())
                    .unwrap();
                reader
                    .decision_field(scope, CurveZeroRateField::reference())
                    .unwrap();
                reader
                    .decision_field(scope, QuoteMidField::reference())
                    .unwrap();
                reader
                    .decision_field(scope, RiskValueField::reference())
                    .unwrap();
                reader
                    .decision_field(scope, PortfolioValueField::reference())
                    .unwrap();
                reader
                    .decision_field(scope, PortfolioDeskField::reference())
                    .unwrap();
                reader
                    .decision_field(scope, PortfolioRankField::reference())
                    .unwrap();
            })
            .unwrap()
            .into_parts();
        let reads = self
            .application
            .begin_projected_application_read_attempt(admission, projection)
            .unwrap();
        let mut effects = reads
            .complete_projected_dependencies()
            .unwrap()
            .begin_effect_program();
        let record = effects.existing_entity(&record).unwrap();
        effects
            .write_field(
                &record,
                MarketRevisionField::reference(),
                amendment.revision,
            )
            .unwrap();
        effects
            .write_field(
                &record,
                MarketDueField::reference(),
                11 + u64::from(self.amendment_ordinal),
            )
            .unwrap();
        effects
            .write_field(
                &record,
                MarketLifecycleField::reference(),
                "active".to_owned(),
            )
            .unwrap();
        effects
            .write_field(
                &record,
                MarketInputField::reference(),
                amendment.risk_value.to_string(),
            )
            .unwrap();
        if let Some(value) = amendment.curve_rate {
            effects
                .write_field(&record, CurveZeroRateField::reference(), value)
                .unwrap();
        }
        if let Some(value) = amendment.quote_mid {
            effects
                .write_field(&record, QuoteMidField::reference(), value)
                .unwrap();
        }
        if let Some(value) = amendment.portfolio_value {
            effects
                .write_field(&record, PortfolioValueField::reference(), value)
                .unwrap();
        }
        if let Some(value) = amendment.portfolio_desk {
            effects
                .write_field(&record, PortfolioDeskField::reference(), value)
                .unwrap();
        }
        if let Some(value) = amendment.portfolio_rank {
            effects
                .write_field(&record, PortfolioRankField::reference(), value)
                .unwrap();
        }
        self.amendment_ordinal = self.amendment_ordinal.saturating_add(1);
        let mut key_identity = [0x31; 32];
        key_identity[31] = self.amendment_ordinal;
        let idempotency = primary_graph::WorthQueryApplicationIdempotencyBinding::new(
            key_identity,
            [self.amendment_ordinal; 32],
        );
        match self
            .application
            .compare_and_commit_application(effects.finish().unwrap(), idempotency)
        {
            primary_graph::WorthQueryApplicationCommitOutcome::Committed(_) => {}
            outcome => panic!("financial market amendment was not committed: {outcome:?}"),
        }
    }
}
