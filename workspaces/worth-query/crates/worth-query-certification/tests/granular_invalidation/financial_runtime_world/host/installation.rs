//! One installation request owns the financial graph and its temporal bindings.
use super::*;
use worth_query_host::facade::application_contribution::{
    with_bootstrap_advancement, WorthQueryBootstrapAdvancementPhase,
};

impl FinancialCourtroomWorld {
    pub(super) fn publish(
        record_identity: &'static str,
        curve_rate: u64,
        quote_mid: u64,
        risk_value: u64,
    ) -> Self {
        let resources =
            worth_query_execution::facade::integration::product_world_resources_for_test(1_024);
        with_bootstrap_advancement(resources.execution_policy(), |phase| {
            Self::publish_in_advancement(
                &phase,
                resources,
                record_identity,
                curve_rate,
                quote_mid,
                risk_value,
            )
        })
        .expect("the declared courtroom policy admits its installation")
    }

    fn publish_in_advancement(
        phase: &WorthQueryBootstrapAdvancementPhase<'_>,
        resources: runtime::WorthQueryProductWorldResources,
        record_identity: &'static str,
        curve_rate: u64,
        quote_mid: u64,
        risk_value: u64,
    ) -> Self {
        let declaration = FinancialHostSchema::declaration().unwrap();
        let conditional_binding = contract::conditional_binding();
        let package = domain::WorthQueryPortableDomainPackage::new(
            domain::WorthQueryPortableDomainIdentity::new("granular_financial_courtroom", 1, 0),
        )
        .application_schema(declaration.clone())
        .domain_operation(contract::operation_definition().into_portable())
        .conditional_application_operation(conditional_binding.clone())
        .validate()
        .unwrap();
        let admitted = domain::WorthQueryInstallationAdmissionProfile::new("host", "financial")
            .admit(package)
            .unwrap();
        let installation = runtime::WorthQueryExecutionRuntimeInstaller::new()
            .install(
                domain::WorthQueryInstallationGeneration::initial(),
                [admitted],
            )
            .unwrap();
        let (installed_runtime, authority) = installation.into_parts();
        let schema = installed_runtime
            .installed_packages()
            .bind_application_schema(declaration)
            .unwrap();
        let principal_binding = schema
            .principal_binding(FinancialPrincipalBinding::reference())
            .unwrap();
        let curve_query = schema
            .certification_query(FinancialIntentQuery::reference())
            .unwrap();
        let quote_query = schema
            .certification_query(FinancialIntentQuery::reference())
            .unwrap();
        let portfolio_query = schema
            .certification_query(FinancialIntentQuery::reference())
            .unwrap();
        let sibling_portfolio_query = schema
            .certification_query(FinancialIntentQuery::reference())
            .unwrap();

        let mut graph = authority
            .prepare_primary_graph(phase, &installed_runtime, &schema, resources)
            .unwrap()
            .semantic_truth_partition(
                worth_foundational::facade::TruthPartitionRole::new("usd-rates").unwrap(),
            );
        seed_graph(
            &mut graph,
            &principal_binding,
            record_identity,
            curve_rate,
            quote_mid,
            risk_value,
        );
        let invariant = Arc::new(graph.retain_invariant_projection_authority());
        let (clock_source, curve_clock_control) = crate::adapters::ClockSource::due();
        let (quote_clock_source, quote_clock_control) = crate::adapters::ClockSource::due();
        let (portfolio_clock_source, portfolio_clock_control) = crate::adapters::ClockSource::due();
        let (sibling_portfolio_clock_source, sibling_portfolio_clock_control) =
            crate::adapters::ClockSource::due();
        let (curve_predicate, curve_gate) = FinancialPredicate::blocked();
        let (quote_predicate, quote_gate) = FinancialPredicate::blocked();
        let (portfolio_predicate, portfolio_gate) = FinancialPredicate::blocked();
        let (sibling_portfolio_predicate, sibling_portfolio_gate) = FinancialPredicate::blocked();
        let quote_output = FinancialQuoteOutputState::new(quote_mid);

        let curve = installed_runtime
            .installed_packages()
            .bind_conditional_application_operation(
                schema
                    .installed_operation(ExecuteFinancial::reference())
                    .unwrap(),
                &conditional_binding,
            )
            .unwrap()
            .bind_node(CurveRiskNode::reference())
            .unwrap()
            .bind_host_predicate_provider(curve_predicate)
            .unwrap()
            .bind_named_clock::<crate::adapters::CourtroomClock, _>(clock_source)
            .unwrap()
            .bind_temporal_intent_projection(
                curve_query,
                ApplicationQueryParameterSet::new(),
                FinancialIntentProjector,
                domain::WorthQueryTemporalIntentBounds::new(8, 16, 8).unwrap(),
            )
            .unwrap();
        let quote = installed_runtime
            .installed_packages()
            .bind_conditional_application_operation(
                schema
                    .installed_operation(ExecuteFinancial::reference())
                    .unwrap(),
                &conditional_binding,
            )
            .unwrap()
            .bind_node(QuoteRiskNode::reference())
            .unwrap()
            .bind_host_predicate_provider(quote_predicate)
            .unwrap()
            .bind_host_output_comparator_provider(QuoteToleranceComparator)
            .unwrap()
            .bind_host_output_version_provider(QuoteOutputVersionProvider(quote_output.clone()))
            .unwrap()
            .bind_named_clock::<crate::adapters::CourtroomClock, _>(quote_clock_source)
            .unwrap()
            .bind_temporal_intent_projection(
                quote_query,
                ApplicationQueryParameterSet::new(),
                FinancialIntentProjector,
                domain::WorthQueryTemporalIntentBounds::new(8, 16, 8).unwrap(),
            )
            .unwrap();
        let portfolio = installed_runtime
            .installed_packages()
            .bind_conditional_application_operation(
                schema
                    .installed_operation(ExecuteFinancial::reference())
                    .unwrap(),
                &conditional_binding,
            )
            .unwrap()
            .bind_node(PortfolioRiskNode::reference())
            .unwrap()
            .bind_host_predicate_provider(portfolio_predicate)
            .unwrap()
            .bind_named_clock::<crate::adapters::CourtroomClock, _>(portfolio_clock_source)
            .unwrap()
            .bind_temporal_intent_projection(
                portfolio_query,
                ApplicationQueryParameterSet::new(),
                FinancialIntentProjector,
                domain::WorthQueryTemporalIntentBounds::new(8, 16, 8).unwrap(),
            )
            .unwrap();
        let sibling_portfolio = installed_runtime
            .installed_packages()
            .bind_conditional_application_operation(
                schema
                    .installed_operation(ExecuteFinancial::reference())
                    .unwrap(),
                &conditional_binding,
            )
            .unwrap()
            .bind_node(PortfolioSiblingRiskNode::reference())
            .unwrap()
            .bind_host_predicate_provider(sibling_portfolio_predicate)
            .unwrap()
            .bind_named_clock::<crate::adapters::CourtroomClock, _>(sibling_portfolio_clock_source)
            .unwrap()
            .bind_temporal_intent_projection(
                sibling_portfolio_query,
                ApplicationQueryParameterSet::new(),
                FinancialIntentProjector,
                domain::WorthQueryTemporalIntentBounds::new(8, 16, 8).unwrap(),
            )
            .unwrap();

        let curve_reconstruction_binding = schema
            .principal_binding(FinancialPrincipalBinding::reference())
            .unwrap();
        let quote_reconstruction_binding = schema
            .principal_binding(FinancialPrincipalBinding::reference())
            .unwrap();
        let portfolio_reconstruction_binding = schema
            .principal_binding(FinancialPrincipalBinding::reference())
            .unwrap();
        let sibling_portfolio_reconstruction_binding = schema
            .principal_binding(FinancialPrincipalBinding::reference())
            .unwrap();
        let curve_authentication = Arc::new(admitted_identity_adapter(&schema));
        let quote_authentication = Arc::new(admitted_identity_adapter(&schema));
        let portfolio_authentication = Arc::new(admitted_identity_adapter(&schema));
        let sibling_portfolio_authentication = Arc::new(admitted_identity_adapter(&schema));
        let mut conditional_installation = graph
            .conditional_application_runtime_installation(
                phase,
                installed_runtime,
                authority,
                schema,
                primary_graph::SignalConditionalEvaluationBudget::development(),
            )
            .unwrap();
        let curve_clock = conditional_installation
            .bind_temporal_operation(
                curve,
                execution(Arc::clone(&invariant)),
                reconstruction(
                    curve_reconstruction_binding,
                    curve_authentication,
                    record_identity,
                ),
            )
            .unwrap();
        let quote_clock = conditional_installation
            .bind_temporal_operation(
                quote,
                execution(Arc::clone(&invariant)),
                reconstruction(
                    quote_reconstruction_binding,
                    quote_authentication,
                    record_identity,
                ),
            )
            .unwrap();
        let portfolio_clock = conditional_installation
            .bind_temporal_operation(
                portfolio,
                execution(Arc::clone(&invariant)),
                reconstruction(
                    portfolio_reconstruction_binding,
                    portfolio_authentication,
                    record_identity,
                ),
            )
            .unwrap();
        let sibling_portfolio_clock = conditional_installation
            .bind_temporal_operation(
                sibling_portfolio,
                execution(Arc::clone(&invariant)),
                reconstruction(
                    sibling_portfolio_reconstruction_binding,
                    sibling_portfolio_authentication,
                    "curve-usd-rates-10y",
                ),
            )
            .unwrap();
        let application = conditional_installation.publish(phase).unwrap();
        Self {
            application,
            curve_clock,
            quote_clock,
            portfolio_clock,
            sibling_portfolio_clock,
            curve_gate,
            quote_gate,
            portfolio_gate,
            sibling_portfolio_gate,
            curve_clock_control,
            quote_clock_control,
            portfolio_clock_control,
            sibling_portfolio_clock_control,
            quote_output,
            invariant,
            record_identity,
            amendment_ordinal: 0,
        }
    }
}
