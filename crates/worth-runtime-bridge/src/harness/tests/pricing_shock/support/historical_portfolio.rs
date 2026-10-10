use super::*;

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_historical_provenance_bundle(
    policy: BridgeRuntimePolicy,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> PricingHistoricalProvenanceBundle {
    let runtime = build_pricing_runtime_with_policy(
        pricing_reference_source(),
        RecordingSignalBridgeSink::default(),
        policy,
    );
    let scenario = generated_pricing_scenario();
    let shock = scenario
        .commit_attributions
        .get("commit:rubber-shock")
        .expect("generated scenario should retain shock attribution");

    let main = runtime
        .evaluate(
            BridgeTruthViewEvaluationRequest::for_historical_commit(
                crate::truth_identity_fixtures::truth_branch_fixture("main"),
                crate::truth_identity_fixtures::truth_commit_fixture("commit:rubber-main"),
            )
            .with_read_packet(pricing_provenance_read_packet("rubber")),
            execution,
        )
        .expect("historical main provenance should materialize");
    let shock_eval = runtime
        .evaluate(
            BridgeTruthViewEvaluationRequest::for_historical_commit(
                crate::truth_identity_fixtures::truth_branch_fixture("pricing-shock"),
                crate::truth_identity_fixtures::truth_commit_fixture("commit:rubber-shock"),
            )
            .with_read_packet(pricing_provenance_read_packet("rubber")),
            execution,
        )
        .expect("historical shock provenance should materialize");
    let main_provenance_texts = read_pricing_provenance_aspect_text_packet(&main);
    let shock_provenance_texts = read_pricing_provenance_aspect_text_packet(&shock_eval);

    PricingHistoricalProvenanceBundle {
        main_commit: crate::truth_identity_fixtures::truth_commit_fixture("commit:rubber-main"),
        main_snapshot: main.snapshot_identity().clone(),
        main_regime: main_provenance_texts.regime_text().to_owned(),
        main_external_factor_microunits: main_provenance_texts
            .external_factor_text()
            .parse()
            .expect("main external factor should parse"),
        shock_commit: crate::truth_identity_fixtures::truth_commit_fixture("commit:rubber-shock"),
        shock_snapshot: shock_eval.snapshot_identity().clone(),
        shock_regime: shock_provenance_texts.regime_text().to_owned(),
        shock_external_factor_microunits: shock_provenance_texts
            .external_factor_text()
            .parse()
            .expect("shock external factor should parse"),
        shock_factor_delta_microunits: shock_provenance_texts
            .factor_delta_text()
            .parse()
            .expect("shock factor delta should parse"),
        shock_trend_delta_microunits: shock_provenance_texts
            .trend_delta_text()
            .parse()
            .expect("shock trend delta should parse"),
        shock_jump_delta_microunits: shock_provenance_texts
            .jump_delta_text()
            .parse()
            .expect("shock jump delta should parse"),
        shock_delta_microunits: shock_provenance_texts
            .shock_delta_text()
            .parse()
            .expect("shock delta should parse"),
        shock_multiplier_per_mille: shock_provenance_texts
            .shock_multiplier_text()
            .parse()
            .expect("shock multiplier should parse"),
        representative_sku: shock.representative_product.sku.clone(),
        representative_retail_price_cents: shock.representative_product.retail_price_cents,
        representative_shipping_cost_cents: shock.representative_product.shipping_cost_cents,
        representative_fuel_shipping_component_cents: shock
            .representative_product
            .fuel_shipping_component_cents,
    }
}

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_portfolio_blast_radius_bundle(
) -> PricingPortfolioBlastRadiusBundle {
    let scenario = generated_pricing_scenario();
    let product_count = scenario.main_portfolio.len();
    let main_repricing_count = scenario
        .main_portfolio
        .iter()
        .filter(|entry| entry.repricing_triggered)
        .count();
    let shock_repricing_count = scenario
        .speculative_portfolio
        .iter()
        .filter(|entry| entry.repricing_triggered)
        .count();
    let main_margin_floor_breach_count = scenario
        .main_portfolio
        .iter()
        .filter(|entry| entry.margin_floor_breached)
        .count();
    let shock_margin_floor_breach_count = scenario
        .speculative_portfolio
        .iter()
        .filter(|entry| entry.margin_floor_breached)
        .count();

    let mut positive_retail_delta_count = 0usize;
    let mut total_retail_delta_cents = 0i64;
    let mut max_retail_delta_sku = String::new();
    let mut max_retail_delta_cents = i64::MIN;
    let mut family_margin_erosion_cents = BTreeMap::<String, i64>::new();
    let mut family_shipping_delta_cents = BTreeMap::<String, i64>::new();
    let mut family_material_delta_cents = BTreeMap::<String, i64>::new();

    for (main_entry, shock_entry) in scenario
        .main_portfolio
        .iter()
        .zip(scenario.speculative_portfolio.iter())
    {
        let retail_delta_cents = shock_entry.retail_price_cents - main_entry.retail_price_cents;
        if retail_delta_cents > 0 {
            positive_retail_delta_count += 1;
        }
        total_retail_delta_cents += retail_delta_cents;
        if retail_delta_cents > max_retail_delta_cents {
            max_retail_delta_cents = retail_delta_cents;
            max_retail_delta_sku = shock_entry.sku.clone();
        }
        *family_margin_erosion_cents
            .entry(shock_entry.family.clone())
            .or_default() += shock_entry.margin_cents - main_entry.margin_cents;
        *family_shipping_delta_cents
            .entry(shock_entry.family.clone())
            .or_default() += shock_entry.shipping_cost_cents - main_entry.shipping_cost_cents;
        *family_material_delta_cents
            .entry(shock_entry.family.clone())
            .or_default() += shock_entry.material_cost_cents - main_entry.material_cost_cents;
    }

    let (top_margin_erosion_family, top_margin_erosion_cents) = family_margin_erosion_cents
        .into_iter()
        .min_by_key(|(_, erosion)| *erosion)
        .expect("family margin erosion should not be empty");
    let (most_shipping_sensitive_family, most_shipping_sensitive_delta_cents) =
        family_shipping_delta_cents
            .into_iter()
            .max_by_key(|(_, delta)| *delta)
            .expect("family shipping delta should not be empty");
    let (most_material_sensitive_family, most_material_sensitive_delta_cents) =
        family_material_delta_cents
            .into_iter()
            .max_by_key(|(_, delta)| *delta)
            .expect("family material delta should not be empty");

    PricingPortfolioBlastRadiusBundle {
        product_count,
        main_repricing_count,
        shock_repricing_count,
        main_margin_floor_breach_count,
        shock_margin_floor_breach_count,
        positive_retail_delta_count,
        total_retail_delta_cents,
        max_retail_delta_sku,
        max_retail_delta_cents,
        top_margin_erosion_family,
        top_margin_erosion_cents,
        most_shipping_sensitive_family,
        most_shipping_sensitive_delta_cents,
        most_material_sensitive_family,
        most_material_sensitive_delta_cents,
    }
}

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_crisis_bundle(
) -> PricingCrisisBundle {
    let scenario = generated_pricing_scenario();
    let main_total_retail_cents = scenario
        .main_portfolio
        .iter()
        .map(|entry| entry.retail_price_cents)
        .sum::<i64>();
    let crisis_total_retail_cents = scenario
        .crisis_portfolio
        .iter()
        .map(|entry| entry.retail_price_cents)
        .sum::<i64>();
    let affected_product_count = scenario
        .main_portfolio
        .iter()
        .zip(scenario.crisis_portfolio.iter())
        .filter(|(main_entry, crisis_entry)| {
            crisis_entry.retail_price_cents > main_entry.retail_price_cents
        })
        .count();

    let mut family_deltas = BTreeMap::<String, i64>::new();
    for (main_entry, crisis_entry) in scenario
        .main_portfolio
        .iter()
        .zip(scenario.crisis_portfolio.iter())
    {
        let family = crisis_entry.family.clone();
        *family_deltas.entry(family).or_default() +=
            crisis_entry.retail_price_cents - main_entry.retail_price_cents;
    }
    let (top_impacted_family, top_impacted_family_delta_cents) = family_deltas
        .into_iter()
        .max_by_key(|(_, delta)| *delta)
        .expect("family deltas should not be empty");
    let (policy_pressure_family, policy_pressure_bps) = scenario
        .crisis_family_tariff_bps
        .iter()
        .max_by_key(|(_, bps)| **bps)
        .map(|(family, bps)| (family.clone(), *bps))
        .expect("crisis family tariff map should not be empty");
    let mut material_deltas = BTreeMap::<String, i64>::new();
    for (material, crisis_value) in &scenario.crisis_overrides {
        let main_value = scenario
            .main_material_prices
            .get(material)
            .copied()
            .expect("main material price should exist for crisis material");
        material_deltas.insert(material.key().to_owned(), crisis_value - main_value);
    }
    let (top_exposure_material, top_exposure_material_delta_cents) = material_deltas
        .into_iter()
        .max_by_key(|(_, delta)| *delta)
        .expect("material deltas should not be empty");

    PricingCrisisBundle {
        crisis_name: "energy-logistics-industrial-crunch".to_owned(),
        affected_product_count,
        main_total_retail_cents,
        crisis_total_retail_cents,
        total_retail_delta_cents: crisis_total_retail_cents - main_total_retail_cents,
        top_impacted_family,
        top_impacted_family_delta_cents,
        dominant_shock_material: "rubber".to_owned(),
        dominant_shock_multiplier_per_mille: 4_000,
        policy_pressure_family,
        policy_pressure_bps,
        top_exposure_material,
        top_exposure_material_delta_cents,
    }
}

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_strategy_bundle(
) -> PricingStrategyBundle {
    let scenario = generated_pricing_scenario();
    let mut hold_unprofitable_count = 0usize;
    let mut partial_absorb_unprofitable_count = 0usize;
    let mut targeted_reprice_positive_delta_count = 0usize;
    let mut targeted_reprice_total_delta_cents = 0i64;
    let mut hold_total_margin_delta_cents = 0i64;
    let mut partial_absorb_total_margin_delta_cents = 0i64;
    let mut targeted_reprice_margin_recovery_cents = 0i64;

    for (main_entry, crisis_entry) in scenario
        .main_portfolio
        .iter()
        .zip(scenario.crisis_portfolio.iter())
    {
        let hold_margin_cents = main_entry.retail_price_cents - crisis_entry.landed_cost_cents;
        hold_total_margin_delta_cents += hold_margin_cents - main_entry.margin_cents;
        if hold_margin_cents < 0 {
            hold_unprofitable_count += 1;
        }

        let partial_absorb_retail_cents = main_entry.retail_price_cents
            + ((crisis_entry.retail_price_cents - main_entry.retail_price_cents) / 2);
        let partial_absorb_margin_cents =
            partial_absorb_retail_cents - crisis_entry.landed_cost_cents;
        partial_absorb_total_margin_delta_cents +=
            partial_absorb_margin_cents - main_entry.margin_cents;
        if partial_absorb_margin_cents < 0 {
            partial_absorb_unprofitable_count += 1;
        }

        let retail_delta_cents = crisis_entry.retail_price_cents - main_entry.retail_price_cents;
        if retail_delta_cents > 0 {
            targeted_reprice_positive_delta_count += 1;
            targeted_reprice_total_delta_cents += retail_delta_cents;
            targeted_reprice_margin_recovery_cents += crisis_entry.margin_cents - hold_margin_cents;
        }
    }

    let (recommended_strategy, recommendation_reason) = if hold_unprofitable_count > 0 {
        (
            "targeted-reprice".to_owned(),
            "hold strategy leaves part of the portfolio underwater under the crisis cost basis"
                .to_owned(),
        )
    } else if partial_absorb_unprofitable_count > 0 {
        (
            "partial-absorb".to_owned(),
            "full hold remains too aggressive, but partial absorption protects more portfolio than broad stasis"
                .to_owned(),
        )
    } else {
        (
            "hold".to_owned(),
            "the portfolio remains profitable without emergency repricing under this crisis basis"
                .to_owned(),
        )
    };
    let promotion_strategy = if recommended_strategy == "hold" {
        "discard-speculative-strategy".to_owned()
    } else {
        "promote-speculative-strategy".to_owned()
    };

    PricingStrategyBundle {
        hold_unprofitable_count,
        partial_absorb_unprofitable_count,
        targeted_reprice_positive_delta_count,
        targeted_reprice_total_delta_cents,
        hold_total_margin_delta_cents,
        partial_absorb_total_margin_delta_cents,
        targeted_reprice_margin_recovery_cents,
        recommended_strategy,
        recommendation_reason,
        promotion_strategy,
    }
}
