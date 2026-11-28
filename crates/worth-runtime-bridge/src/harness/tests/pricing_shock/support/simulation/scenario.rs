use super::*;

pub(in crate::harness::tests::pricing_shock) fn writeback_effect_intent(
    effect_class: BridgeWritebackEffectClass,
    marker: impl Into<String>,
) -> BridgeWritebackEffectIntent {
    let aspect_key = match effect_class {
        BridgeWritebackEffectClass::ProjectedStateDiff => "bridge.writeback.projected-state-diff",
        BridgeWritebackEffectClass::AspectReconciliation => {
            "bridge.writeback.aspect-reconciliation"
        }
    };
    BridgeWritebackEffectIntent::validated_scalar_patch(
        effect_class,
        AspectKey::new(aspect_key).expect("static writeback effect aspect key is valid"),
        AspectValue::String(marker.into().into()),
    )
    .expect("pricing writeback effect intent should validate as a foundational scalar patch")
}

#[derive(Clone)]
pub(in crate::harness::tests::pricing_shock) struct GeneratedPricingScenario {
    pub(in crate::harness::tests::pricing_shock) main_steel_cost: i64,
    pub(in crate::harness::tests::pricing_shock) main_rubber_cost: i64,
    pub(in crate::harness::tests::pricing_shock) speculative_rubber_cost: i64,
    pub(in crate::harness::tests::pricing_shock) live_main_steel_cost: i64,
    pub(in crate::harness::tests::pricing_shock) interleaved_main_rubber_cost: i64,
    pub(in crate::harness::tests::pricing_shock) fanout_second_steel_cost: i64,
    pub(in crate::harness::tests::pricing_shock) main_portfolio: Vec<ProductPriceBreakdown>,
    pub(in crate::harness::tests::pricing_shock) speculative_portfolio: Vec<ProductPriceBreakdown>,
    pub(in crate::harness::tests::pricing_shock) crisis_portfolio: Vec<ProductPriceBreakdown>,
    pub(in crate::harness::tests::pricing_shock) main_material_prices:
        BTreeMap<PricingMaterial, i64>,
    pub(in crate::harness::tests::pricing_shock) crisis_overrides: BTreeMap<PricingMaterial, i64>,
    pub(in crate::harness::tests::pricing_shock) crisis_family_tariff_bps: BTreeMap<String, i64>,
    pub(in crate::harness::tests::pricing_shock) commit_attributions:
        BTreeMap<String, PricingCommitAttribution>,
}

pub(in crate::harness::tests::pricing_shock) fn generated_pricing_scenario(
) -> GeneratedPricingScenario {
    let mut world = PricingDomainWorld::new(1_337);
    let _warmup_wave = world.advance_material_streams();
    let main_wave = world.advance_material_streams();

    let main_steel_cost = world.current_material_price_microunits(PricingMaterial::Steel);
    let main_rubber_cost = world.current_material_price_microunits(PricingMaterial::Rubber);
    let speculative_rubber_cost =
        world.shocked_material_price_microunits(PricingMaterial::Rubber, 4_000);
    let crisis_overrides = BTreeMap::from([
        (
            PricingMaterial::Steel,
            world.shocked_material_price_microunits(PricingMaterial::Steel, 1_450),
        ),
        (
            PricingMaterial::Rubber,
            world.shocked_material_price_microunits(PricingMaterial::Rubber, 4_000),
        ),
        (
            PricingMaterial::Fuel,
            world.shocked_material_price_microunits(PricingMaterial::Fuel, 2_800),
        ),
        (
            PricingMaterial::Copper,
            world.shocked_material_price_microunits(PricingMaterial::Copper, 1_650),
        ),
        (
            PricingMaterial::Electronics,
            world.shocked_material_price_microunits(PricingMaterial::Electronics, 1_250),
        ),
    ]);
    let crisis_family_tariff_bps = BTreeMap::from([
        ("washer".to_owned(), 650),
        ("dryer".to_owned(), 650),
        ("e-bike".to_owned(), 420),
    ]);
    let main_portfolio = world.price_matrix();
    let main_material_prices = BTreeMap::from([
        (
            PricingMaterial::Steel,
            world.current_material_price_microunits(PricingMaterial::Steel),
        ),
        (
            PricingMaterial::Aluminum,
            world.current_material_price_microunits(PricingMaterial::Aluminum),
        ),
        (
            PricingMaterial::Copper,
            world.current_material_price_microunits(PricingMaterial::Copper),
        ),
        (
            PricingMaterial::Rubber,
            world.current_material_price_microunits(PricingMaterial::Rubber),
        ),
        (
            PricingMaterial::PlasticResin,
            world.current_material_price_microunits(PricingMaterial::PlasticResin),
        ),
        (
            PricingMaterial::Electronics,
            world.current_material_price_microunits(PricingMaterial::Electronics),
        ),
        (
            PricingMaterial::Packaging,
            world.current_material_price_microunits(PricingMaterial::Packaging),
        ),
        (
            PricingMaterial::Labor,
            world.current_material_price_microunits(PricingMaterial::Labor),
        ),
        (
            PricingMaterial::Fuel,
            world.current_material_price_microunits(PricingMaterial::Fuel),
        ),
    ]);
    let speculative_portfolio =
        world.price_matrix_with_overrides([(PricingMaterial::Rubber, speculative_rubber_cost)]);
    let crisis_portfolio = world
        .price_matrix_with_scenario(crisis_overrides.clone(), crisis_family_tariff_bps.clone());

    let mut commit_attributions = BTreeMap::new();
    commit_attributions.insert(
        "commit:steel-main".to_owned(),
        pricing_commit_attribution(
            &world,
            crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-main"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-main"),
            crate::truth_identity_fixtures::truth_branch_fixture("main"),
            PricingMaterial::Steel,
            attribution_for(&main_wave, PricingMaterial::Steel),
            0,
            1000,
            "bicycle-000",
        ),
    );
    commit_attributions.insert(
        "commit:rubber-main".to_owned(),
        pricing_commit_attribution(
            &world,
            crate::truth_identity_fixtures::truth_commit_fixture("commit:rubber-main"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-main"),
            crate::truth_identity_fixtures::truth_branch_fixture("main"),
            PricingMaterial::Rubber,
            attribution_for(&main_wave, PricingMaterial::Rubber),
            0,
            1000,
            "scooter-001",
        ),
    );
    commit_attributions.insert(
        "commit:rubber-shock".to_owned(),
        pricing_commit_attribution(
            &world,
            crate::truth_identity_fixtures::truth_commit_fixture("commit:rubber-shock"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-shock"),
            crate::truth_identity_fixtures::truth_branch_fixture("pricing-shock"),
            PricingMaterial::Rubber,
            attribution_for(&main_wave, PricingMaterial::Rubber),
            speculative_rubber_cost - main_rubber_cost,
            4000,
            "scooter-001",
        ),
    );

    let interleaved_wave = world.advance_material_streams();
    let live_main_steel_cost = world.current_material_price_microunits(PricingMaterial::Steel);
    let interleaved_main_rubber_cost =
        world.current_material_price_microunits(PricingMaterial::Rubber);
    let fanout_second_steel_cost = live_main_steel_cost;
    commit_attributions.insert(
        "commit:steel-main-live".to_owned(),
        pricing_commit_attribution(
            &world,
            crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-main-live"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-main-live"),
            crate::truth_identity_fixtures::truth_branch_fixture("main"),
            PricingMaterial::Steel,
            attribution_for(&interleaved_wave, PricingMaterial::Steel),
            0,
            1000,
            "bicycle-000",
        ),
    );
    commit_attributions.insert(
        "commit:rubber-main-interleaved".to_owned(),
        pricing_commit_attribution(
            &world,
            crate::truth_identity_fixtures::truth_commit_fixture("commit:rubber-main-interleaved"),
            crate::truth_identity_fixtures::truth_snapshot_fixture(
                "snapshot:pricing-main-interleaved",
            ),
            crate::truth_identity_fixtures::truth_branch_fixture("main"),
            PricingMaterial::Rubber,
            attribution_for(&interleaved_wave, PricingMaterial::Rubber),
            0,
            1000,
            "scooter-001",
        ),
    );
    commit_attributions.insert(
        "commit:steel-fanout-b".to_owned(),
        pricing_commit_attribution(
            &world,
            crate::truth_identity_fixtures::truth_commit_fixture("commit:steel-fanout-b"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot:pricing-fanout-b"),
            crate::truth_identity_fixtures::truth_branch_fixture("main"),
            PricingMaterial::Steel,
            attribution_for(&interleaved_wave, PricingMaterial::Steel),
            0,
            1000,
            "bicycle-000",
        ),
    );

    GeneratedPricingScenario {
        main_steel_cost,
        main_rubber_cost,
        speculative_rubber_cost,
        live_main_steel_cost,
        interleaved_main_rubber_cost,
        fanout_second_steel_cost,
        main_portfolio,
        speculative_portfolio,
        crisis_portfolio,
        main_material_prices,
        crisis_overrides,
        crisis_family_tariff_bps,
        commit_attributions,
    }
}

pub(in crate::harness::tests::pricing_shock) fn attribution_for(
    wave: &MaterialTickWave,
    material: PricingMaterial,
) -> MaterialPriceAttribution {
    wave.changed_materials
        .iter()
        .find(|tick| tick.material == material)
        .expect("requested material attribution should exist in generated wave")
        .attribution
        .clone()
}

pub(in crate::harness::tests::pricing_shock) fn pricing_commit_attribution(
    world: &PricingDomainWorld,
    commit_identity: TruthCommitIdentity,
    snapshot_identity: TruthSnapshotIdentity,
    branch_identity: TruthBranchIdentity,
    material: PricingMaterial,
    material_attribution: MaterialPriceAttribution,
    shock_delta_microunits: i64,
    shock_multiplier_per_mille: i64,
    representative_sku: &str,
) -> PricingCommitAttribution {
    PricingCommitAttribution {
        commit_identity,
        snapshot_identity,
        branch_identity,
        material,
        material_attribution,
        shock_delta_microunits,
        shock_multiplier_per_mille,
        representative_product: world.explain_product_price(representative_sku),
    }
}
