use super::FintechEvaluationView;
use crate::data::error::SignalError;
use crate::facade::{AspectVersion, NodeEvaluationResult};
use crate::logic::evaluation::EvaluationOutput;

use super::super::aspects::{ALERT, PRICE, RISK};

impl super::FintechEvaluationShape {
    pub(super) fn evaluate_portfolio_aggregation<V: FintechEvaluationView>(
        &self,
        view: &mut V,
    ) -> Result<Option<EvaluationOutput>, SignalError> {
        let node = view.node();
        if let Some(book_index) = self
            .book_aggregates
            .iter()
            .position(|candidate| *candidate == node)
        {
            let mut risk_total = view
                .read_aspect_version(self.aggregate_sources[book_index].book_state, RISK)?
                .get(RISK);
            let mut alert_total = view
                .read_aspect_version(self.aggregate_sources[book_index].book_state, ALERT)?
                .get(ALERT);
            let fx = view.read_aspect_version(self.fx.eur_jpy, PRICE)?.get(PRICE);
            for instrument in &self.instruments {
                if instrument.book_index == book_index {
                    risk_total += view
                        .read_aspect_version(instrument.core.risk, RISK)?
                        .get(RISK);
                    alert_total += view
                        .read_aspect_version(instrument.core.alert, ALERT)?
                        .get(ALERT);
                }
            }
            risk_total += fx / 100;
            let aggregate_alert = u64::from(alert_total > 0);
            return Ok(Some(
                view.finish(
                    NodeEvaluationResult::from_version(AspectVersion::from_updates([
                        (RISK, risk_total),
                        (ALERT, aggregate_alert),
                    ]))
                    .with_output_identity(format!(
                        "book-{book_index}-{risk_total}-{aggregate_alert}"
                    ))
                    .with_continuity_token("book-aggregate"),
                ),
            ));
        }

        if let Some(desk_index) = self
            .desk_aggregates
            .iter()
            .position(|candidate| *candidate == node)
        {
            let mut risk_total = view
                .read_aspect_version(self.aggregate_sources[desk_index].desk_limit, RISK)?
                .get(RISK);
            let mut alert_total = view
                .read_aspect_version(self.aggregate_sources[desk_index].desk_limit, ALERT)?
                .get(ALERT);
            for (book_index, book_node) in self.book_aggregates.iter().enumerate() {
                if book_index % self.desk_aggregates.len() == desk_index {
                    risk_total += view.read_aspect_version(*book_node, RISK)?.get(RISK);
                    alert_total += view.read_aspect_version(*book_node, ALERT)?.get(ALERT);
                }
            }
            let aggregate_alert = u64::from(alert_total > 0 || risk_total > 25_000);
            return Ok(Some(
                view.finish(
                    NodeEvaluationResult::from_version(AspectVersion::from_updates([
                        (RISK, risk_total),
                        (ALERT, aggregate_alert),
                    ]))
                    .with_output_identity(format!(
                        "desk-{desk_index}-{risk_total}-{aggregate_alert}"
                    ))
                    .with_continuity_token("desk-aggregate"),
                ),
            ));
        }

        if let Some(scenario_index) = self
            .scenario_aggregates
            .iter()
            .position(|candidate| *candidate == node)
        {
            let mut total = 0_u64;
            for instrument in &self.instruments {
                total += view
                    .read_aspect_version(instrument.scenarios[scenario_index], RISK)?
                    .get(RISK);
            }
            return Ok(Some(
                view.finish(
                    NodeEvaluationResult::from_version(AspectVersion::from_updates([(
                        RISK, total,
                    )]))
                    .with_output_identity(format!("scenario-agg-{scenario_index}-{total}"))
                    .with_continuity_token("scenario-aggregate"),
                ),
            ));
        }

        if let Some(bucket_index) = self
            .bucket_aggregates
            .iter()
            .position(|candidate| *candidate == node)
        {
            let mut total = 0_u64;
            for instrument in &self.instruments {
                total += view
                    .read_aspect_version(instrument.buckets[bucket_index], RISK)?
                    .get(RISK);
            }
            return Ok(Some(
                view.finish(
                    NodeEvaluationResult::from_version(AspectVersion::from_updates([(
                        RISK, total,
                    )]))
                    .with_output_identity(format!("bucket-agg-{bucket_index}-{total}"))
                    .with_continuity_token("bucket-aggregate"),
                ),
            ));
        }
        Ok(None)
    }
}
