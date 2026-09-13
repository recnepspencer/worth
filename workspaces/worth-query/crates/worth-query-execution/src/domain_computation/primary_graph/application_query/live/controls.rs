use std::num::NonZeroUsize;

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

use crate::domain_computation::primary_graph::live_delivery::{
    WorthQueryLiveDeliveryControlDenial, WorthQueryLiveDeliveryControls,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationLiveControlDenial {
    Delivery(WorthQueryLiveDeliveryControlDenial),
    ZeroMaterializedRecordLimit,
    ZeroWorkLimit,
}

#[derive(Clone)]
pub struct WorthQueryApplicationLiveControls {
    delivery: WorthQueryLiveDeliveryControls,
    maximum_materialized_record_count: NonZeroUsize,
    maximum_work_per_delivery: NonZeroUsize,
}

impl WorthQueryApplicationLiveControls {
    pub fn bounded(
        request: WorthQueryRequestScope,
        buffer_capacity: usize,
        maximum_materialized_record_count: usize,
        maximum_work_per_delivery: usize,
    ) -> Result<Self, WorthQueryApplicationLiveControlDenial> {
        let delivery = WorthQueryLiveDeliveryControls::bounded(request, buffer_capacity)
            .map_err(WorthQueryApplicationLiveControlDenial::Delivery)?;
        let maximum_materialized_record_count =
            NonZeroUsize::new(maximum_materialized_record_count)
                .ok_or(WorthQueryApplicationLiveControlDenial::ZeroMaterializedRecordLimit)?;
        let maximum_work_per_delivery = NonZeroUsize::new(maximum_work_per_delivery)
            .ok_or(WorthQueryApplicationLiveControlDenial::ZeroWorkLimit)?;
        Ok(Self {
            delivery,
            maximum_materialized_record_count,
            maximum_work_per_delivery,
        })
    }

    pub const fn request(&self) -> &WorthQueryRequestScope {
        self.delivery.request()
    }

    pub const fn buffer_capacity(&self) -> usize {
        self.delivery.buffer_capacity()
    }

    pub const fn maximum_materialized_record_count(&self) -> NonZeroUsize {
        self.maximum_materialized_record_count
    }

    pub const fn maximum_work_per_delivery(&self) -> NonZeroUsize {
        self.maximum_work_per_delivery
    }

    pub(crate) fn replace_request(&mut self, request: WorthQueryRequestScope) {
        self.delivery.replace_request(request);
    }
}
