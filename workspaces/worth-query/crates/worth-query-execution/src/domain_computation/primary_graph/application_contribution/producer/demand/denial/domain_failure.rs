//! Encode the original terminal decision as data without retaining cleanup.
use super::*;

impl WorthQueryOutputDemandDenial {
    pub(in crate::domain_computation::primary_graph) fn record_domain_failure(
        &self,
    ) -> Result<
        Vec<u8>,
        crate::domain_computation::primary_graph::query_obligations::QueryObligationDenial,
    > {
        let data = (
            self.subject().to_owned(),
            self.domain_reason().map(str::to_owned),
            self.producer_denial_value().map(
                super::super::super::denial_value::WorthQueryRecordedProducerDenialValue::capture,
            ),
        );
        rmp_serde::to_vec_named(&data).map_err(|_| crate::domain_computation::primary_graph::query_obligations::QueryObligationDenial::MalformedFrame)
    }
    pub(in crate::domain_computation::primary_graph) fn readmit_domain_failure(
        bytes: &[u8],
    ) -> Result<
        Self,
        crate::domain_computation::primary_graph::query_obligations::QueryObligationDenial,
    > {
        let (subject,reason,value): (String,Option<String>,Option<super::super::super::denial_value::WorthQueryRecordedProducerDenialValue>) = rmp_serde::from_slice(bytes).map_err(|_| crate::domain_computation::primary_graph::query_obligations::QueryObligationDenial::MalformedFrame)?;
        let mut denial = Self::new(
            WorthQueryOutputDemandDenialKind::ProducerDomainDenied,
            subject,
        );
        if let Some(value) = value {
            denial.domain = Some(Box::new(producer_value::ProducerDomainDenial {
                reason: reason.map(Into::into),
                value: super::super::super::WorthQueryProducerDenialValue::Recorded(Box::new(
                    value,
                )),
            }));
        }
        Ok(denial)
    }
}
