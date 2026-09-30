use crate::WorthServerSurfaceFamily;

use super::{WorthServerRequestContext, WorthServerTransportClass};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerResolvedRequestContext {
    request_context: WorthServerRequestContext,
    surface_family: WorthServerSurfaceFamily,
    transport_class: WorthServerTransportClass,
}

impl WorthServerResolvedRequestContext {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        self.request_context.owned_allocation_capacity_bytes()
    }

    pub(crate) fn new(
        request_context: WorthServerRequestContext,
        surface_family: WorthServerSurfaceFamily,
        transport_class: WorthServerTransportClass,
    ) -> Self {
        Self {
            request_context,
            surface_family,
            transport_class,
        }
    }

    pub fn request_context(&self) -> &WorthServerRequestContext {
        &self.request_context
    }

    pub fn surface_family(&self) -> WorthServerSurfaceFamily {
        self.surface_family
    }

    pub fn transport_class(&self) -> WorthServerTransportClass {
        self.transport_class
    }
}
