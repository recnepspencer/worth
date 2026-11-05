//! The program support an installation admits before it publishes anything.
//!
//! Program support has to exist earlier than the program runtime that carries
//! it: the invariant adapters lowered for this installation select by rostered
//! program, and bootstrap seeds the branch program activation record before any
//! ordinary row. Both happen while the installed schema is being published, so
//! admission is handed in as a step the publication runs at that exact moment
//! rather than bolted on afterwards.

use worth_query_installation::facade::WorthQueryInstalledApplicationSchema;

use super::WorthQueryInMemoryApplicationDenial;
use crate::domain_computation::primary_graph::vocabulary::application_installation::program_admission::WorthQueryAdmittedProgramSupport;

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
