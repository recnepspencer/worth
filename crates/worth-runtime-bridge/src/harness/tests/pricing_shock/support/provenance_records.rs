use super::*;

pub(in crate::harness::tests::pricing_shock) fn snapshot_with_provenance(
    snapshot: &SnapshotFixture,
    attributions: &[&PricingCommitAttribution],
) -> SnapshotFixture {
    let mut records = snapshot.records().to_vec();
    for attribution in attributions {
        let component = attribution.material.key();
        records.push(SnapshotReadRecord::for_request(
            &pricing_provenance_read_request(component, "regime"),
            worth_foundational::facade::AspectValue::String(
                (format!("{:?}", attribution.material_attribution.regime)).into(),
            ),
        ));
        records.push(SnapshotReadRecord::for_request(
            &pricing_provenance_read_request(component, "external-factor"),
            worth_foundational::facade::AspectValue::String(
                (attribution
                    .material_attribution
                    .external_factor_microunits
                    .to_string())
                .into(),
            ),
        ));
        records.push(SnapshotReadRecord::for_request(
            &pricing_provenance_read_request(component, "factor-delta"),
            worth_foundational::facade::AspectValue::String(
                (attribution
                    .material_attribution
                    .factor_delta_microunits
                    .to_string())
                .into(),
            ),
        ));
        records.push(SnapshotReadRecord::for_request(
            &pricing_provenance_read_request(component, "trend-delta"),
            worth_foundational::facade::AspectValue::String(
                (attribution
                    .material_attribution
                    .trend_delta_microunits
                    .to_string())
                .into(),
            ),
        ));
        records.push(SnapshotReadRecord::for_request(
            &pricing_provenance_read_request(component, "jump-delta"),
            worth_foundational::facade::AspectValue::String(
                (attribution
                    .material_attribution
                    .jump_delta_microunits
                    .to_string())
                .into(),
            ),
        ));
        records.push(SnapshotReadRecord::for_request(
            &pricing_provenance_read_request(component, "shock-delta"),
            worth_foundational::facade::AspectValue::String(
                (attribution.shock_delta_microunits.to_string()).into(),
            ),
        ));
        records.push(SnapshotReadRecord::for_request(
            &pricing_provenance_read_request(component, "shock-multiplier"),
            worth_foundational::facade::AspectValue::String(
                (attribution.shock_multiplier_per_mille.to_string()).into(),
            ),
        ));
    }
    SnapshotFixture::new(snapshot.identity().clone(), records)
        .with_read_result_identity(snapshot.read_result_identity().clone())
}

pub(in crate::harness::tests::pricing_shock) fn snapshot_with_corrupted_provenance_field(
    snapshot: &SnapshotFixture,
    component: &str,
    field: &str,
    content: impl Into<String>,
) -> SnapshotFixture {
    let target_request = pricing_provenance_read_request(component, field);
    let target_correlation_id = target_request.correlation_id().clone();
    let replacement_content = content.into();
    let mut replaced = false;
    let mut records = Vec::with_capacity(snapshot.records().len());
    for record in snapshot.records() {
        if record.correlation_id() == &target_correlation_id {
            records.push(SnapshotReadRecord::for_request(
                &target_request,
                worth_foundational::facade::AspectValue::String(replacement_content.clone().into()),
            ));
            replaced = true;
        } else {
            records.push(record.clone());
        }
    }
    assert!(
        replaced,
        "provenance correlation `{}` should exist before corruption",
        target_correlation_id.as_str()
    );
    SnapshotFixture::new(snapshot.identity().clone(), records)
        .with_read_result_identity(snapshot.read_result_identity().clone())
}

pub(in crate::harness::tests::pricing_shock) fn snapshot_with_identity(
    snapshot: &SnapshotFixture,
    snapshot_identity: TruthSnapshotIdentity,
) -> SnapshotFixture {
    SnapshotFixture::new(snapshot_identity.clone(), snapshot.records().to_vec())
        .with_read_result_identity(snapshot_identity)
}

pub(in crate::harness::tests::pricing_shock) fn read_single_aspect_value_text(
    evaluation: &crate::facade::BridgeTruthViewEvaluation,
) -> String {
    let reads = evaluation
        .observation()
        .read_planned_packet(worth_execution::ExecutionRequest::serial(
            &crate::snapshot::test_serial_request(),
        ))
        .expect("truth-view read packet should materialize");
    pricing_aspect_value_text(
        reads.records()[0]
            .scalar_aspect_value()
            .expect("pricing snapshot read should return a scalar aspect value"),
    )
}

pub(in crate::harness::tests::pricing_shock) fn read_single_money_cents(
    evaluation: &crate::facade::BridgeTruthViewEvaluation,
) -> i64 {
    read_single_aspect_value_text(evaluation)
        .parse::<i64>()
        .expect("pricing aspect value should be parseable as integer cents")
}

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
            worth_foundational::facade::InternedString::Symbol(symbol) => {
                format!("symbol:{}", symbol.0)
            }
        },
        worth_foundational::facade::AspectValue::Int64(value) => value.to_string(),
        other => panic!("pricing snapshot expected string or integer aspect value, got {other:?}"),
    }
}
