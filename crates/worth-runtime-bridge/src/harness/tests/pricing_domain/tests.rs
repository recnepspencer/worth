use std::collections::BTreeMap;

use super::{PricingDomainWorld, PricingMaterial, PricingProduct};
use worth_harness::facade::FeedStreamEventKind;

#[test]
fn pricing_domain_reference_catalog_is_large_shared_and_fuel_coupled() {
    let world = PricingDomainWorld::new(101);
    let products = world.products();

    assert_eq!(products.len(), 100);
    assert!(products.iter().any(|product| product.family == "bicycle"));
    assert!(products.iter().any(|product| product.family == "washer"));
    assert!(products
        .iter()
        .all(|product| product.shipping.fuel_burn_microliters_per_kg_km > 0));
    assert!(products
        .iter()
        .all(|product| product.tolerance_gate.repricing_threshold_bps > 0));
    assert!(products.iter().all(|product| {
        product
            .materials
            .iter()
            .any(|requirement| requirement.material == PricingMaterial::Labor)
    }));
}

#[test]
fn pricing_domain_hidden_streams_advance_all_material_prices() {
    let mut world = PricingDomainWorld::new(202);
    let first_wave = world.advance_material_streams();
    let second_wave = world.advance_material_streams();
    assert_eq!(first_wave.sequence, 1);
    assert_eq!(second_wave.sequence, 2);
    assert_eq!(first_wave.changed_materials.len(), 9);
    assert_eq!(second_wave.changed_materials.len(), 9);
    let materials = second_wave
        .changed_materials
        .iter()
        .map(|tick| tick.material)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(materials.len(), 9);
    assert!(materials.contains(&PricingMaterial::Fuel));
    assert!(materials
        .into_iter()
        .all(|material| world.current_material_price_microunits(material) > 0));
    assert!(first_wave
        .changed_materials
        .iter()
        .any(|tick| tick.event_kind != FeedStreamEventKind::Stable));
}

#[test]
fn pricing_domain_price_matrix_reflects_fuel_driven_shipping_and_tolerance_gates() {
    let mut world = PricingDomainWorld::new(303);
    world.advance_material_streams();
    let matrix = world.price_matrix();
    let bicycle = matrix
        .iter()
        .find(|breakdown| breakdown.sku.starts_with("bicycle-"))
        .expect("bicycle should exist in reference matrix");

    assert!(bicycle.material_cost_cents > 0);
    assert!(bicycle.shipping_cost_cents > 0);
    assert!(bicycle.baseline_landed_cost_cents > 0);
    assert!(bicycle.landed_cost_cents >= bicycle.material_cost_cents);
    assert_eq!(
        bicycle.landed_cost_delta_cents,
        (bicycle.landed_cost_cents - bicycle.baseline_landed_cost_cents).abs()
    );
    assert!(bicycle.retail_price_cents > bicycle.landed_cost_cents);
    assert_eq!(
        bicycle.repricing_triggered,
        (bicycle.repricing_threshold_cents > 0
            && bicycle.landed_cost_delta_cents >= bicycle.repricing_threshold_cents)
            || bicycle.margin_floor_breached
    );
    assert!(!bicycle.margin_floor_breached);
}

#[test]
fn pricing_domain_product_breakdown_matches_independent_oracle_math() {
    let mut world = PricingDomainWorld::new(404);
    world.advance_material_streams();
    let product = world
        .products()
        .iter()
        .find(|product| product.sku.starts_with("bicycle-"))
        .expect("bicycle should exist in reference catalog");
    let mut overrides = BTreeMap::new();
    overrides.insert(
        PricingMaterial::Steel,
        world.current_material_price_microunits(PricingMaterial::Steel) + 12_500,
    );
    overrides.insert(
        PricingMaterial::Fuel,
        world.current_material_price_microunits(PricingMaterial::Fuel) + 9_500,
    );
    let breakdown = world.price_product_with_scenario(product, overrides.clone(), BTreeMap::new());
    let oracle = independent_breakdown_oracle(&world, product, &overrides);

    assert_eq!(breakdown.material_cost_cents, oracle.material_cost_cents);
    assert_eq!(breakdown.shipping_cost_cents, oracle.shipping_cost_cents);
    assert_eq!(
        breakdown.baseline_landed_cost_cents,
        oracle.baseline_landed_cost_cents
    );
    assert_eq!(breakdown.landed_cost_cents, oracle.landed_cost_cents);
    assert_eq!(
        breakdown.landed_cost_delta_cents,
        oracle.landed_cost_delta_cents
    );
    assert_eq!(
        breakdown.repricing_threshold_cents,
        oracle.repricing_threshold_cents
    );
    assert_eq!(breakdown.repricing_triggered, oracle.repricing_triggered);
}

#[test]
fn pricing_domain_independent_oracle_holds_across_multiple_seeds_and_families() {
    for seed in [11_u64, 29, 47, 83, 131] {
        let mut world = PricingDomainWorld::new(seed);
        world.advance_material_streams();
        world.advance_material_streams();

        for family_prefix in ["bicycle-", "washer-", "e-bike-"] {
            let product = world
                .products()
                .iter()
                .find(|product| product.sku.starts_with(family_prefix))
                .expect("reference family should exist");
            let mut overrides = BTreeMap::new();
            overrides.insert(
                PricingMaterial::Steel,
                world.current_material_price_microunits(PricingMaterial::Steel) + 7_500,
            );
            overrides.insert(
                PricingMaterial::Fuel,
                world.current_material_price_microunits(PricingMaterial::Fuel) + 5_500,
            );
            if family_prefix == "washer-" || family_prefix == "e-bike-" {
                overrides.insert(
                    PricingMaterial::Electronics,
                    world.current_material_price_microunits(PricingMaterial::Electronics) + 4_000,
                );
            }

            let breakdown =
                world.price_product_with_scenario(product, overrides.clone(), BTreeMap::new());
            let oracle = independent_breakdown_oracle(&world, product, &overrides);

            assert_eq!(breakdown.material_cost_cents, oracle.material_cost_cents);
            assert_eq!(breakdown.shipping_cost_cents, oracle.shipping_cost_cents);
            assert_eq!(
                breakdown.baseline_landed_cost_cents,
                oracle.baseline_landed_cost_cents
            );
            assert_eq!(breakdown.landed_cost_cents, oracle.landed_cost_cents);
            assert_eq!(
                breakdown.landed_cost_delta_cents,
                oracle.landed_cost_delta_cents
            );
            assert_eq!(
                breakdown.repricing_threshold_cents,
                oracle.repricing_threshold_cents
            );
            assert_eq!(breakdown.repricing_triggered, oracle.repricing_triggered);
        }
    }
}

struct OracleBreakdown {
    material_cost_cents: i64,
    shipping_cost_cents: i64,
    baseline_landed_cost_cents: i64,
    landed_cost_cents: i64,
    landed_cost_delta_cents: i64,
    repricing_threshold_cents: i64,
    repricing_triggered: bool,
}

fn independent_breakdown_oracle(
    world: &PricingDomainWorld,
    product: &PricingProduct,
    overrides: &BTreeMap<PricingMaterial, i64>,
) -> OracleBreakdown {
    let material_cost_cents = product
        .materials
        .iter()
        .map(|requirement| {
            overrides
                .get(&requirement.material)
                .copied()
                .unwrap_or_else(|| world.current_material_price_microunits(requirement.material))
                * requirement.quantity_milliunits
                / 1_000_000
        })
        .sum::<i64>();
    let baseline_material_cost_cents = product
        .materials
        .iter()
        .map(|requirement| {
            world.baseline_material_price_microunits(requirement.material)
                * requirement.quantity_milliunits
                / 1_000_000
        })
        .sum::<i64>();
    let weight_kg = product.shipping.shipment_weight_grams.max(0) / 1_000;
    let packaging_surcharge_cents = product.shipping.packaging_volume_cc / 25_000;
    let fuel_price_microunits = overrides
        .get(&PricingMaterial::Fuel)
        .copied()
        .unwrap_or_else(|| world.current_material_price_microunits(PricingMaterial::Fuel));
    let shipping_cost_cents = product.shipping.base_shipping_cents
        + fuel_price_microunits
            * product.shipping.fuel_burn_microliters_per_kg_km
            * weight_kg
            * product.shipping.route_distance_km
            / 1_000_000_000
        + packaging_surcharge_cents;
    let baseline_shipping_cost_cents = product.shipping.base_shipping_cents
        + world.baseline_material_price_microunits(PricingMaterial::Fuel)
            * product.shipping.fuel_burn_microliters_per_kg_km
            * weight_kg
            * product.shipping.route_distance_km
            / 1_000_000_000
        + packaging_surcharge_cents;
    let baseline_landed_cost_cents = baseline_material_cost_cents + baseline_shipping_cost_cents;
    let landed_cost_cents = material_cost_cents + shipping_cost_cents;
    let landed_cost_delta_cents = (landed_cost_cents - baseline_landed_cost_cents).abs();
    let repricing_threshold_cents =
        baseline_landed_cost_cents * product.tolerance_gate.repricing_threshold_bps / 10_000;
    let margin_floor_breached = product.margin_bps < product.tolerance_gate.margin_floor_bps;

    OracleBreakdown {
        material_cost_cents,
        shipping_cost_cents,
        baseline_landed_cost_cents,
        landed_cost_cents,
        landed_cost_delta_cents,
        repricing_threshold_cents,
        repricing_triggered: (repricing_threshold_cents > 0
            && landed_cost_delta_cents >= repricing_threshold_cents)
            || margin_floor_breached,
    }
}
