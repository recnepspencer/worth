//! The runtime composes `record_reservations`.
use worth_query_installation::facade::ApplicationSchema;

use super::super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::record_reservations::host::RecordReservationsHost;

impl<Schema: ApplicationSchema> RecordReservationsHost<Schema>
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn record_reservations_host(&self) -> &dyn RecordReservationsHost<Schema> {
        self
    }
}
