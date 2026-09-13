use std::num::NonZeroUsize;

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryLiveDeliveryControlDenial {
    ZeroBufferCapacity,
}

#[derive(Clone)]
pub struct WorthQueryLiveDeliveryControls {
    request: WorthQueryRequestScope,
    buffer_capacity: NonZeroUsize,
}

impl WorthQueryLiveDeliveryControls {
    pub fn bounded(
        request: WorthQueryRequestScope,
        buffer_capacity: usize,
    ) -> Result<Self, WorthQueryLiveDeliveryControlDenial> {
        let buffer_capacity = NonZeroUsize::new(buffer_capacity)
            .ok_or(WorthQueryLiveDeliveryControlDenial::ZeroBufferCapacity)?;
        Ok(Self {
            request,
            buffer_capacity,
        })
    }

    pub const fn request(&self) -> &WorthQueryRequestScope {
        &self.request
    }

    pub const fn buffer_capacity(&self) -> usize {
        self.buffer_capacity.get()
    }

    pub(crate) fn replace_request(&mut self, request: WorthQueryRequestScope) {
        self.request = request;
    }
}
