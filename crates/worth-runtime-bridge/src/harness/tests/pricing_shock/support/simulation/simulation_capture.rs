use super::*;

pub(in crate::harness::tests::pricing_shock) fn capture_pricing_simulation_suite(
) -> PricingShockSimulationSuite {
    const BRANCH_COUNT: usize = 10;
    const ITERATIONS_PER_BRANCH: usize = 10;

    let mut material_summaries = Vec::new();
    let mut iteration_traces = Vec::new();

    for &material in simulation_candidate_materials() {
        let mut branch_mean_deltas = Vec::<(String, i64)>::new();
        let mut material_total_delta = 0i64;
        let mut shipping_total_delta = 0i64;
        let mut material_cost_total_delta = 0i64;
        let mut breach_total = 0i64;
        let mut repricing_total = 0i64;

        for branch_index in 0..BRANCH_COUNT {
            let mut world =
                PricingDomainWorld::new(70_000 + (material as u64 * 1_000) + branch_index as u64);
            let mut branch_total_delta = 0i64;

            for iteration_index in 0..ITERATIONS_PER_BRANCH {
                let wave = world.advance_material_streams();
                let attribution = attribution_for(&wave, material);
                let baseline = world.price_matrix();
                let multiplier = natural_shock_multiplier_per_mille(
                    material,
                    &attribution,
                    branch_index,
                    iteration_index,
                );
                let override_map = BTreeMap::from([(
                    material,
                    world.shocked_material_price_microunits(material, multiplier),
                )]);
                let tariff_map =
                    family_tariff_bps_for_material(material, branch_index, iteration_index);
                let shocked = world.price_matrix_with_scenario(override_map, tariff_map);

                let baseline_total_retail_cents = sum_retail_cents(&baseline);
                let shocked_total_retail_cents = sum_retail_cents(&shocked);
                let total_retail_delta_cents =
                    shocked_total_retail_cents - baseline_total_retail_cents;
                let shipping_delta_cents = baseline
                    .iter()
                    .zip(shocked.iter())
                    .map(|(base, shock)| shock.shipping_cost_cents - base.shipping_cost_cents)
                    .sum::<i64>();
                let material_delta_cents = baseline
                    .iter()
                    .zip(shocked.iter())
                    .map(|(base, shock)| shock.material_cost_cents - base.material_cost_cents)
                    .sum::<i64>();
                let margin_floor_breach_count = shocked
                    .iter()
                    .filter(|entry| entry.margin_floor_breached)
                    .count();
                let repricing_count = shocked
                    .iter()
                    .filter(|entry| entry.repricing_triggered)
                    .count();

                branch_total_delta += total_retail_delta_cents;
                material_total_delta += total_retail_delta_cents;
                shipping_total_delta += shipping_delta_cents;
                material_cost_total_delta += material_delta_cents;
                breach_total += margin_floor_breach_count as i64;
                repricing_total += repricing_count as i64;

                iteration_traces.push(PricingShockSimulationIterationTrace {
                    material: material.key().to_owned(),
                    branch_identity: format!("sim:{}:branch-{branch_index:02}", material.key()),
                    iteration_index,
                    regime: format!("{:?}", attribution.regime),
                    event_kind: format!("{:?}", attribution.event_kind),
                    shock_multiplier_per_mille: multiplier,
                    baseline_total_retail_cents,
                    shocked_total_retail_cents,
                    total_retail_delta_cents,
                    shipping_delta_cents,
                    material_delta_cents,
                    margin_floor_breach_count,
                    repricing_count,
                });
            }

            branch_mean_deltas.push((
                format!("sim:{}:branch-{branch_index:02}", material.key()),
                branch_total_delta / ITERATIONS_PER_BRANCH as i64,
            ));
        }

        let (worst_branch_identity, worst_branch_mean_total_delta_cents) = branch_mean_deltas
            .iter()
            .max_by_key(|(_, delta)| *delta)
            .cloned()
            .expect("branch means should not be empty");
        let total_iterations = (BRANCH_COUNT * ITERATIONS_PER_BRANCH) as i64;
        let mean_total_retail_delta_cents = material_total_delta / total_iterations;
        let mean_shipping_delta_cents = shipping_total_delta / total_iterations;
        let mean_material_delta_cents = material_cost_total_delta / total_iterations;
        let mean_margin_floor_breach_count = breach_total / total_iterations;
        let mean_repricing_count = repricing_total / total_iterations;
        let damage_score = mean_total_retail_delta_cents
            + (mean_margin_floor_breach_count * 50)
            + mean_shipping_delta_cents.abs() / 10;

        material_summaries.push(PricingShockSimulationMaterialSummary {
            material: material.key().to_owned(),
            branch_count: BRANCH_COUNT,
            iterations_per_branch: ITERATIONS_PER_BRANCH,
            mean_total_retail_delta_cents,
            mean_shipping_delta_cents,
            mean_material_delta_cents,
            mean_margin_floor_breach_count,
            mean_repricing_count,
            worst_branch_identity,
            worst_branch_mean_total_delta_cents,
            damage_score,
        });
    }

    material_summaries.sort_by(|left, right| {
        right.damage_score.cmp(&left.damage_score).then_with(|| {
            right
                .mean_total_retail_delta_cents
                .cmp(&left.mean_total_retail_delta_cents)
        })
    });
    let ranked_materials_by_damage =
        PricingShockRankedMaterialDamageSet::from_ranked_material_summaries(
            material_summaries.iter(),
        );

    PricingShockSimulationSuite {
        branch_count: BRANCH_COUNT,
        iterations_per_branch: ITERATIONS_PER_BRANCH,
        material_summaries,
        ranked_materials_by_damage,
        iteration_traces,
    }
}
