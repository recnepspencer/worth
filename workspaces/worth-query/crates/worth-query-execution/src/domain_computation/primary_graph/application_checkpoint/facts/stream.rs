//! Bounded visitation over the same current fact codec, without a retained Vec.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl;

pub(in crate::domain_computation::primary_graph) struct FactDecoder<'bytes, 'scope, 'authority> {
    cursor: CheckpointCursor<'bytes>,
    control: StorageControl<'scope, 'authority>,
    remaining: usize,
    count: usize,
    funding: Arc<CodecPayloadReservation>,
}
impl<'bytes, 'scope, 'authority> FactDecoder<'bytes, 'scope, 'authority> {
    pub(in crate::domain_computation::primary_graph) fn new(
        bytes: &'bytes [u8],
        wire_version: u16,
        request: Option<
            &'scope worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        >,
        policy: worth_execution::ExecutionAllocationPolicy<'scope, 'authority>,
    ) -> Result<Self, FactDecodeDenial> {
        if wire_version != WIRE_VERSION {
            return Err("checkpoint producer fact wire version is unsupported"
                .to_owned()
                .into());
        }
        if bytes.len() < 5 || bytes.len() > MAXIMUM_FACT_BYTES {
            return Err("checkpoint producer fact payload length is invalid"
                .to_owned()
                .into());
        }
        let mut cursor = CheckpointCursor::new(bytes);
        let count = usize::try_from(cursor.next_u32()?)
            .map_err(|_| "checkpoint producer fact count exceeds host".to_owned())?;
        if count == 0 || count > cursor.remaining.len() / MINIMUM_FACT_WIRE_BYTES {
            return Err("checkpoint producer fact count is invalid"
                .to_owned()
                .into());
        }
        let control = StorageControl::new(policy, request);
        control.check_live().map_err(FactDecodeDenial::Retention)?;
        let quote = super::payload::parse_quote(bytes.len(), count)
            .ok_or(FactDecodeDenial::Retention(
                crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial::Representability))?;
        let funding = CodecPayloadReservation::admit(control, quote)?;
        Ok(Self {
            funding,
            cursor,
            control,
            remaining: count,
            count,
        })
    }
    pub(in crate::domain_computation::primary_graph) fn funding(
        &self,
    ) -> Arc<CodecPayloadReservation> {
        Arc::clone(&self.funding)
    }
    pub(in crate::domain_computation::primary_graph) fn count(&self) -> usize {
        self.count
    }
    pub(in crate::domain_computation::primary_graph) fn next_fact(
        &mut self,
    ) -> Result<Option<Fact>, FactDecodeDenial> {
        self.control
            .check_live()
            .map_err(FactDecodeDenial::Retention)?;
        if self.remaining == 0 {
            if !self.cursor.is_empty() {
                return Err("checkpoint producer fact payload length differs"
                    .to_owned()
                    .into());
            }
            return Ok(None);
        }
        let fact = super::record::decode(&mut self.cursor, self.control)?;
        self.remaining -= 1;
        Ok(Some(fact))
    }
}
