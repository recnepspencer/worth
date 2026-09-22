//! The program support an installation admits before it publishes anything.
//!
//! Program support has to exist earlier than the program runtime that carries
//! it: the invariant adapters lowered for this installation select by rostered
//! program, and bootstrap seeds the branch program activation record before any
//! ordinary row. Both happen while the installed schema is being published, so
//! admission is handed in as a step the publication runs at that exact moment
//! rather than bolted on afterwards.

use std::sync::Arc;

use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationSchema, WorthQueryProgramSupportRoster,
};

use super::WorthQueryInMemoryApplicationDenial;

/// The complete, immutable program support one installation admitted, together
/// with the program its first occurrence activates.
pub(in crate::domain_computation::primary_graph) struct WorthQueryAdmittedProgramSupport<Schema> {
    pub(in crate::domain_computation::primary_graph) roster:
        Arc<WorthQueryProgramSupportRoster<Schema>>,
    pub(in crate::domain_computation::primary_graph) initial_revision: ApplicationProgramRevision,
}

/// The admission step a program-hosted installation runs against its own
/// installed schema, before invariant lowering and before bootstrap.
pub(in crate::domain_computation::primary_graph) type WorthQueryProgramAdmissionStep<
    'authoring,
    Schema,
> = Box<
    dyn FnOnce(
            &WorthQueryInstalledApplicationSchema<Schema>,
        ) -> Result<
            WorthQueryAdmittedProgramSupport<Schema>,
            WorthQueryInMemoryApplicationDenial,
        > + 'authoring,
>;
