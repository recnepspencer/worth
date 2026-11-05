//! What `record_reservations` needs from the runtime that composes it.

/// The runtime's identity plus what `record_reservations` reads or calls above itself.
pub trait RecordReservationsHost<Schema>:
    crate::domain_computation::primary_graph::runtime_host::WorthQueryRuntimeIdentityHost<Schema>
{
    /// This host as the trait object `record_reservations` is handed.
    fn record_reservations_host(&self) -> &dyn RecordReservationsHost<Schema>;
}
