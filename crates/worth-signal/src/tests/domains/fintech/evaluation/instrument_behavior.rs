use super::FintechEvaluationView;
use crate::data::error::SignalError;
use crate::facade::{AspectVersion, NodeEvaluationResult};
use crate::logic::evaluation::EvaluationOutput;

use super::super::aspects::{ALERT, CURVE, LIQUIDITY, PRICE, RISK, VOL};

impl super::FintechEvaluationShape {
    pub(super) fn evaluate_instrument_node<V: FintechEvaluationView>(
        &self,
        view: &mut V,
    ) -> Result<Option<EvaluationOutput>, SignalError> {
        let node = view.node();
        if node == self.fx.eur_jpy {
            let eur_usd = view.read_aspect_version(self.fx.eur_usd, PRICE)?.get(PRICE);
            let usd_jpy = view.read_aspect_version(self.fx.usd_jpy, PRICE)?.get(PRICE);
            let eur_jpy = eur_usd.saturating_mul(usd_jpy) / 10_000;
            return Ok(Some(
                view.finish(
                    NodeEvaluationResult::from_version(AspectVersion::from_updates([(
                        PRICE, eur_jpy,
                    )]))
                    .with_output_identity(format!("eur-jpy-{eur_jpy}"))
                    .with_continuity_token("fx-cross"),
                ),
            ));
        }

        for instrument in &self.instruments {
            if node == instrument.core.normalized {
                let price = view
                    .read_aspect_version(instrument.core.market, PRICE)?
                    .get(PRICE);
                let vol = view
                    .read_aspect_version(instrument.core.market, VOL)?
                    .get(VOL);
                let curve = view
                    .read_aspect_version(instrument.core.market, CURVE)?
                    .get(CURVE);
                let liquidity = view
                    .read_aspect_version(instrument.core.market, LIQUIDITY)?
                    .get(LIQUIDITY);
                let risk_hint = price / 24 + vol + curve / 20 + liquidity / 12;
                return Ok(Some(
                    view.finish(
                        NodeEvaluationResult::from_version(AspectVersion::from_updates([
                            (PRICE, price),
                            (VOL, vol),
                            (CURVE, curve),
                            (LIQUIDITY, liquidity),
                            (RISK, risk_hint),
                        ]))
                        .with_output_identity(format!(
                            "normalized-{price}-{vol}-{curve}-{liquidity}"
                        ))
                        .with_continuity_token("normalized"),
                    ),
                ));
            }

            if node == instrument.core.price {
                let price = view
                    .read_aspect_version(instrument.core.normalized, PRICE)?
                    .get(PRICE);
                let vol = view
                    .read_aspect_version(instrument.core.normalized, VOL)?
                    .get(VOL);
                let curve = view
                    .read_aspect_version(instrument.core.normalized, CURVE)?
                    .get(CURVE);
                let priced = price + vol / 8 + curve / 32;
                let priced_risk = priced / 3 + vol / 2 + curve / 6;
                return Ok(Some(
                    view.finish(
                        NodeEvaluationResult::from_version(AspectVersion::from_updates([
                            (PRICE, priced),
                            (RISK, priced_risk),
                        ]))
                        .with_output_identity(format!("price-{priced}-{priced_risk}"))
                        .with_continuity_token("price"),
                    ),
                ));
            }

            if node == instrument.core.risk {
                let priced_risk = view
                    .read_aspect_version(instrument.core.price, RISK)?
                    .get(RISK);
                let liquidity = view
                    .read_aspect_version(instrument.core.normalized, LIQUIDITY)?
                    .get(LIQUIDITY);
                let risk = priced_risk + liquidity / 3;
                let alert = u64::from(risk > 1_600);
                return Ok(Some(
                    view.finish(
                        NodeEvaluationResult::from_version(AspectVersion::from_updates([
                            (RISK, risk),
                            (ALERT, alert),
                        ]))
                        .with_output_identity(format!("risk-{risk}-{alert}"))
                        .with_continuity_token("risk"),
                    ),
                ));
            }

            if node == instrument.core.alert {
                let alert = view
                    .read_aspect_version(instrument.core.risk, ALERT)?
                    .get(ALERT);
                return Ok(Some(
                    view.finish(
                        NodeEvaluationResult::from_version(AspectVersion::from_updates([(
                            ALERT, alert,
                        )]))
                        .with_output_identity(format!("alert-{alert}"))
                        .with_continuity_token("alert"),
                    ),
                ));
            }

            if node == instrument.core.threshold {
                let price = view
                    .read_aspect_version(instrument.core.price, PRICE)?
                    .get(PRICE);
                return Ok(Some(
                    view.finish(
                        NodeEvaluationResult::from_version(AspectVersion::from_updates([(
                            PRICE, price,
                        )]))
                        .with_output_identity(format!("threshold-{price}"))
                        .with_continuity_token("threshold"),
                    ),
                ));
            }

            if let Some(bucket_index) = instrument
                .buckets
                .iter()
                .position(|candidate| *candidate == node)
            {
                let risk = view
                    .read_aspect_version(instrument.core.risk, RISK)?
                    .get(RISK);
                let threshold = view
                    .read_aspect_version(instrument.core.threshold, PRICE)?
                    .get(PRICE);
                let curve = view
                    .read_aspect_version(self.curve_buckets[bucket_index], CURVE)?
                    .get(CURVE);
                let surface_vol = view
                    .read_aspect_version(self.vol_surface_buckets[bucket_index], VOL)?
                    .get(VOL);
                let bucket_risk = risk + threshold / 5 + curve / 9 + surface_vol / 7;
                return Ok(Some(
                    view.finish(
                        NodeEvaluationResult::from_version(AspectVersion::from_updates([(
                            RISK,
                            bucket_risk,
                        )]))
                        .with_output_identity(format!("bucket-{bucket_index}-{bucket_risk}"))
                        .with_continuity_token("bucket-risk"),
                    ),
                ));
            }

            if let Some(scenario_index) = instrument
                .scenarios
                .iter()
                .position(|candidate| *candidate == node)
            {
                let price = view
                    .read_aspect_version(instrument.core.price, PRICE)?
                    .get(PRICE);
                let risk = view
                    .read_aspect_version(instrument.core.risk, RISK)?
                    .get(RISK);
                let alert = view
                    .read_aspect_version(instrument.core.alert, ALERT)?
                    .get(ALERT);
                let scenario_risk = view
                    .read_aspect_version(self.scenario_sources[scenario_index], RISK)?
                    .get(RISK);
                let scenario_vol = view
                    .read_aspect_version(self.scenario_sources[scenario_index], VOL)?
                    .get(VOL);
                let aggregate = risk + scenario_risk + scenario_vol + price / 10;
                let scenario_alert = u64::from(alert == 1 || aggregate > 2_700);
                return Ok(Some(
                    view.finish(
                        NodeEvaluationResult::from_version(AspectVersion::from_updates([
                            (RISK, aggregate),
                            (ALERT, scenario_alert),
                        ]))
                        .with_output_identity(format!("scenario-{scenario_index}-{aggregate}"))
                        .with_continuity_token("scenario-risk"),
                    ),
                ));
            }
        }
        Ok(None)
    }
}
