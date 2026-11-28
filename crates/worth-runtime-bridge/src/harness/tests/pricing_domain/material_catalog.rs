use super::world_state::PricingDomainWorld;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::harness::tests) enum PricingMaterial {
    Steel,
    Aluminum,
    Copper,
    Rubber,
    PlasticResin,
    Electronics,
    Packaging,
    Labor,
    Fuel,
}

impl PricingMaterial {
    pub(in crate::harness::tests) fn key(self) -> &'static str {
        match self {
            Self::Steel => "steel",
            Self::Aluminum => "aluminum",
            Self::Copper => "copper",
            Self::Rubber => "rubber",
            Self::PlasticResin => "plastic-resin",
            Self::Electronics => "electronics",
            Self::Packaging => "packaging",
            Self::Labor => "labor",
            Self::Fuel => "fuel",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::harness::tests) struct MaterialRequirement {
    pub(in crate::harness::tests) material: PricingMaterial,
    pub(in crate::harness::tests) quantity_milliunits: i64,
}

impl MaterialRequirement {
    pub(in crate::harness::tests) fn new(
        material: PricingMaterial,
        quantity_milliunits: i64,
    ) -> Self {
        Self {
            material,
            quantity_milliunits,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::harness::tests) struct ShippingSpec {
    pub(in crate::harness::tests) route_class: String,
    pub(in crate::harness::tests) route_distance_km: i64,
    pub(in crate::harness::tests) shipment_weight_grams: i64,
    pub(in crate::harness::tests) packaging_volume_cc: i64,
    pub(in crate::harness::tests) base_shipping_cents: i64,
    pub(in crate::harness::tests) fuel_burn_microliters_per_kg_km: i64,
}

impl ShippingSpec {
    pub(in crate::harness::tests) fn new(
        route_class: impl Into<String>,
        route_distance_km: i64,
        shipment_weight_grams: i64,
        packaging_volume_cc: i64,
        base_shipping_cents: i64,
        fuel_burn_microliters_per_kg_km: i64,
    ) -> Self {
        Self {
            route_class: route_class.into(),
            route_distance_km,
            shipment_weight_grams,
            packaging_volume_cc,
            base_shipping_cents,
            fuel_burn_microliters_per_kg_km,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::harness::tests) struct ToleranceGate {
    pub(in crate::harness::tests) repricing_threshold_bps: i64,
    pub(in crate::harness::tests) margin_floor_bps: i64,
}

impl ToleranceGate {
    pub(in crate::harness::tests) fn new(
        repricing_threshold_bps: i64,
        margin_floor_bps: i64,
    ) -> Self {
        Self {
            repricing_threshold_bps,
            margin_floor_bps,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::harness::tests) struct PricingProduct {
    pub(in crate::harness::tests) sku: String,
    pub(in crate::harness::tests) family: String,
    pub(in crate::harness::tests) materials: Vec<MaterialRequirement>,
    pub(in crate::harness::tests) shipping: ShippingSpec,
    pub(in crate::harness::tests) tolerance_gate: ToleranceGate,
    pub(in crate::harness::tests) margin_bps: i64,
}

impl PricingDomainWorld {
    pub(in crate::harness::tests) fn reference_catalog() -> Vec<PricingProduct> {
        let templates = [
            (
                "bicycle",
                vec![
                    MaterialRequirement::new(PricingMaterial::Steel, 18_000),
                    MaterialRequirement::new(PricingMaterial::Rubber, 5_200),
                    MaterialRequirement::new(PricingMaterial::Aluminum, 2_400),
                    MaterialRequirement::new(PricingMaterial::Labor, 6_000),
                    MaterialRequirement::new(PricingMaterial::Packaging, 1_000),
                ],
                ShippingSpec::new("regional-ground", 820, 15_000, 140_000, 2_800, 130),
                ToleranceGate::new(45, 1_800),
                2_800,
            ),
            (
                "scooter",
                vec![
                    MaterialRequirement::new(PricingMaterial::Steel, 9_500),
                    MaterialRequirement::new(PricingMaterial::Rubber, 3_600),
                    MaterialRequirement::new(PricingMaterial::PlasticResin, 1_400),
                    MaterialRequirement::new(PricingMaterial::Labor, 4_500),
                    MaterialRequirement::new(PricingMaterial::Packaging, 900),
                ],
                ShippingSpec::new("metro-ground", 410, 9_000, 82_000, 1_900, 150),
                ToleranceGate::new(40, 1_600),
                2_700,
            ),
            (
                "wheelbarrow",
                vec![
                    MaterialRequirement::new(PricingMaterial::Steel, 11_000),
                    MaterialRequirement::new(PricingMaterial::Rubber, 2_500),
                    MaterialRequirement::new(PricingMaterial::PlasticResin, 1_800),
                    MaterialRequirement::new(PricingMaterial::Labor, 3_400),
                    MaterialRequirement::new(PricingMaterial::Packaging, 700),
                ],
                ShippingSpec::new("regional-ground", 760, 12_500, 120_000, 2_200, 128),
                ToleranceGate::new(35, 1_500),
                2_400,
            ),
            (
                "washer",
                vec![
                    MaterialRequirement::new(PricingMaterial::Steel, 24_000),
                    MaterialRequirement::new(PricingMaterial::Copper, 4_500),
                    MaterialRequirement::new(PricingMaterial::PlasticResin, 3_400),
                    MaterialRequirement::new(PricingMaterial::Electronics, 6_500),
                    MaterialRequirement::new(PricingMaterial::Labor, 8_500),
                    MaterialRequirement::new(PricingMaterial::Packaging, 1_400),
                ],
                ShippingSpec::new("appliance-truck", 1_150, 68_000, 380_000, 8_200, 170),
                ToleranceGate::new(30, 1_300),
                2_200,
            ),
            (
                "dryer",
                vec![
                    MaterialRequirement::new(PricingMaterial::Steel, 22_000),
                    MaterialRequirement::new(PricingMaterial::Copper, 3_900),
                    MaterialRequirement::new(PricingMaterial::PlasticResin, 2_800),
                    MaterialRequirement::new(PricingMaterial::Electronics, 5_400),
                    MaterialRequirement::new(PricingMaterial::Labor, 7_900),
                    MaterialRequirement::new(PricingMaterial::Packaging, 1_300),
                ],
                ShippingSpec::new("appliance-truck", 1_050, 62_000, 360_000, 7_800, 166),
                ToleranceGate::new(30, 1_300),
                2_100,
            ),
            (
                "e-bike",
                vec![
                    MaterialRequirement::new(PricingMaterial::Steel, 10_000),
                    MaterialRequirement::new(PricingMaterial::Aluminum, 6_000),
                    MaterialRequirement::new(PricingMaterial::Rubber, 4_800),
                    MaterialRequirement::new(PricingMaterial::Electronics, 9_000),
                    MaterialRequirement::new(PricingMaterial::Labor, 7_200),
                    MaterialRequirement::new(PricingMaterial::Packaging, 1_100),
                ],
                ShippingSpec::new("regional-ground", 910, 24_000, 180_000, 3_900, 136),
                ToleranceGate::new(35, 1_700),
                2_900,
            ),
        ];

        let mut products = Vec::new();
        for product_idx in 0..100 {
            let template = &templates[product_idx % templates.len()];
            products.push(PricingProduct {
                sku: format!("{}-{product_idx:03}", template.0),
                family: template.0.to_owned(),
                materials: template.1.clone(),
                shipping: template.2.clone(),
                tolerance_gate: template.3.clone(),
                margin_bps: template.4,
            });
        }
        products
    }
}
