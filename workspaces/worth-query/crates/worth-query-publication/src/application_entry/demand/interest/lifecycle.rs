use worth_query_execution::facade::application_contribution::WorthQueryApplicationOutputDemand;
use worth_query_execution::facade::primary_graph::WorthQueryOutputDemandNotifications;
use worth_query_installation::facade::ApplicationSchema;

use super::{WorthQueryApplicationOutputDemandDenial, WorthQueryApplicationOutputDemandHandle};

impl<Schema, Demand> WorthQueryApplicationOutputDemandHandle<'_, Schema, Demand>
where
    Schema: ApplicationSchema,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    pub fn close(&mut self) {
        if !self.closed {
            self.admitted.close();
        }
        self.closed = true;
    }

    pub fn notifications(
        &self,
    ) -> Result<WorthQueryOutputDemandNotifications, WorthQueryApplicationOutputDemandDenial> {
        self.admitted
            .notifications()
            .map_err(WorthQueryApplicationOutputDemandDenial::Demand)
    }
}

impl<Schema, Demand> Drop for WorthQueryApplicationOutputDemandHandle<'_, Schema, Demand>
where
    Schema: ApplicationSchema,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    fn drop(&mut self) {
        self.close();
    }
}
