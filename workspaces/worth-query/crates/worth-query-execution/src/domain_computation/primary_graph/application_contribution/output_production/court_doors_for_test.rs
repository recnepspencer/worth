//! Court doors on output production's own state.
use super::{WorthQueryInstalledOutputProducerRoutes, WorthQueryOutputProduction};

impl<Schema> WorthQueryOutputProduction<Schema> {
    #[doc(hidden)]
    pub fn producer_routes_mut_for_test(&mut self) -> &mut WorthQueryInstalledOutputProducerRoutes {
        &mut self.producer_routes
    }
}
