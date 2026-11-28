pub(in crate::harness::tests::pricing_shock) fn read_pricing_provenance_aspect_text_packet(
    evaluation: &crate::facade::BridgeTruthViewEvaluation,
) -> PricingProvenanceAspectTextPacket {
    PricingProvenanceAspectTextPacket::from_evaluation(evaluation)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::harness::tests::pricing_shock) struct PricingProvenanceAspectTextPacket {
    regime: String,
    external_factor: String,
    factor_delta: String,
    trend_delta: String,
    jump_delta: String,
    shock_delta: String,
    shock_multiplier: String,
}

impl PricingProvenanceAspectTextPacket {
    fn from_evaluation(evaluation: &crate::facade::BridgeTruthViewEvaluation) -> Self {
        let reads = evaluation
            .observation()
            .read_planned_packet(worth_execution::ExecutionRequest::serial(
                &crate::snapshot::test_serial_request(),
            ))
            .expect("truth-view read packet should materialize");
        let mut values = reads.records().iter().map(|record| {
            pricing_aspect_value_text(
                record
                    .scalar_aspect_value()
                    .expect("pricing snapshot read should return scalar aspect values"),
            )
        });
        let packet = Self {
            regime: values.next().expect("provenance regime should be present"),
            external_factor: values
                .next()
                .expect("provenance external factor should be present"),
            factor_delta: values
                .next()
                .expect("provenance factor delta should be present"),
            trend_delta: values
                .next()
                .expect("provenance trend delta should be present"),
            jump_delta: values
                .next()
                .expect("provenance jump delta should be present"),
            shock_delta: values
                .next()
                .expect("provenance shock delta should be present"),
            shock_multiplier: values
                .next()
                .expect("provenance shock multiplier should be present"),
        };
        assert!(
            values.next().is_none(),
            "pricing provenance packet should expose exactly seven aspect values"
        );
        packet
    }

    pub(in crate::harness::tests::pricing_shock) fn regime_text(&self) -> &str {
        self.regime.as_str()
    }

    pub(in crate::harness::tests::pricing_shock) fn external_factor_text(&self) -> &str {
        self.external_factor.as_str()
    }

    pub(in crate::harness::tests::pricing_shock) fn factor_delta_text(&self) -> &str {
        self.factor_delta.as_str()
    }

    pub(in crate::harness::tests::pricing_shock) fn trend_delta_text(&self) -> &str {
        self.trend_delta.as_str()
    }

    pub(in crate::harness::tests::pricing_shock) fn jump_delta_text(&self) -> &str {
        self.jump_delta.as_str()
    }

    pub(in crate::harness::tests::pricing_shock) fn shock_delta_text(&self) -> &str {
        self.shock_delta.as_str()
    }

    pub(in crate::harness::tests::pricing_shock) fn shock_multiplier_text(&self) -> &str {
        self.shock_multiplier.as_str()
    }

    pub(in crate::harness::tests::pricing_shock) fn field_text(&self, field: &str) -> &str {
        match field {
            "external-factor" => self.external_factor_text(),
            "factor-delta" => self.factor_delta_text(),
            "trend-delta" => self.trend_delta_text(),
            "jump-delta" => self.jump_delta_text(),
            "shock-delta" => self.shock_delta_text(),
            "shock-multiplier" => self.shock_multiplier_text(),
            _ => panic!("unexpected provenance field `{field}`"),
        }
    }
}

pub(in crate::harness::tests::pricing_shock) fn pricing_aspect_value_text(
    value: &worth_foundational::facade::AspectValue,
) -> String {
    match value {
        worth_foundational::facade::AspectValue::String(value) => match value {
            worth_foundational::facade::InternedString::Raw(text) => text.clone(),
            worth_foundational::facade::InternedString::Stored(text) => text.as_str().to_owned(),
            worth_foundational::facade::InternedString::Symbol(symbol) => {
                format!("symbol:{}", symbol.0)
            }
        },
        worth_foundational::facade::AspectValue::Int64(value) => value.to_string(),
        other => panic!("pricing snapshot expected string or integer aspect value, got {other:?}"),
    }
}

pub(in crate::harness::tests::pricing_shock) fn read_cost(
    evaluation: &crate::facade::BridgeTruthViewEvaluation,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> String {
    let result = evaluation
        .observation()
        .read_planned_packet(execution)
        .unwrap();
    let value = result.records()[0].validated_value_posture().unwrap();
    let worth_foundational::facade::ContractValidatedAspectValueView::Struct(cost) =
        value.payload().view()
    else {
        panic!("the native cost read must preserve its struct contract")
    };
    pricing_aspect_value_text(
        cost.get(&crate::relational_source::relational_test_support::field_key("usd"))
            .expect("declared cost field"),
    )
}
