//! Select publication succession before projecting a genuine live prior role.
use super::*;
use crate::domain_computation::primary_graph::HandlerExecutionDenial;
use worth_query_declaration::facade::application_operation::{
    WorthQueryCreateOutput, WorthQueryExactlyOneOutput, WorthQueryPreserveOutput,
};
use worth_relational::facade::mvcc::CompanionPreflightStop;

/// Local error context, not another publication or currentness authority.
#[derive(Debug)]
struct PriorPublicationSelectionStop(CompanionPreflightStop);
impl std::fmt::Display for PriorPublicationSelectionStop {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(output, "prior publication selection: {:?}", self.0)
    }
}
impl std::error::Error for PriorPublicationSelectionStop {}

impl<Schema: ApplicationSchema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    pub(in crate::domain_computation::primary_graph) fn prior_output_preserved_or_created<
        Preserve,
        Preserved,
        Create,
        Created,
    >(
        &mut self,
    ) -> Result<WorthQueryInvariantEntityIdentity<Schema, Preserved::Entity>, HandlerExecutionDenial>
    where
        Preserve: ApplicationMutationBinding<Schema>,
        Preserved: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Contract = Preserve::Output,
            Action = WorthQueryPreserveOutput,
            Cardinality = WorthQueryExactlyOneOutput,
        >,
        Preserved::Entity: OperationReads<Operation>,
        Create: ApplicationMutationBinding<Schema>,
        Created: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Contract = Create::Output,
            Entity = Preserved::Entity,
            Action = WorthQueryCreateOutput,
            Cardinality = WorthQueryExactlyOneOutput,
        >,
    {
        let unavailable = || {
            HandlerExecutionDenial::new(WorthQueryPriorOutputDenial::new(
                WorthQueryPriorOutputDenialKind::Unavailable,
                Preserved::NAME,
            ))
        };
        let scope = self.operation_scope.as_ref().ok_or_else(unavailable)?;
        let occurrence = self
            .reader
            .selected_product_occurrence
            .ok_or_else(unavailable)?;
        let generation = self
            .reader
            .selected_product_generation
            .ok_or_else(unavailable)?;
        let partition = self
            .reader
            .selected_source_partition_identity
            .ok_or_else(unavailable)?;
        let selected = self
            .reader
            .output_lineage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .resolve_binding_succession::<Preserve, Create>(
                self.reader.execution,
                scope,
                occurrence,
                generation,
                partition,
            );
        let selected = selected.map_err(|stop| {
            // The enclosing projection's ordinary completion projects this
            // exact native stop before it can return any handler result.
            let cause = match stop {
                CompanionPreflightStop::Execution(cause) => Some(cause),
                CompanionPreflightStop::WorkCounterOverflow => Some(
                    worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkCounterOverflow),
                _ => None,
            };
            if let Some(cause) = cause { self.reader.native_read_stop.get_or_insert(cause); }
            HandlerExecutionDenial::new(PriorPublicationSelectionStop(stop))
        })?;
        let correspondence = selected.correspondence.ok_or_else(unavailable)?;
        if correspondence.binding_type() == Some(TypeId::of::<Preserve>()) {
            self.project_succession_role::<Preserved>(&correspondence)
        } else if correspondence.binding_type() == Some(TypeId::of::<Create>()) {
            self.project_succession_role::<Created>(&correspondence)
        } else {
            Err(unavailable())
        }
    }

    fn project_succession_role<Role>(
        &mut self,
        correspondence: &WorthQueryApplicationOutputCorrespondence,
    ) -> Result<WorthQueryInvariantEntityIdentity<Schema, Role::Entity>, HandlerExecutionDenial>
    where
        Role: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Cardinality = WorthQueryExactlyOneOutput,
        >,
        Role::Entity: OperationReads<Operation>,
    {
        self.admit_prior_entity::<Role::Entity>(Role::NAME)
            .map_err(HandlerExecutionDenial::new)?;
        self.charge_role_work(Role::NAME)
            .map_err(HandlerExecutionDenial::new)?;
        let id = correspondence
            .bound_entity(&OutputRoleUse::fixed::<Role>())
            .map_err(|denial| {
                HandlerExecutionDenial::new(WorthQueryPriorOutputDenial::projection(
                    Role::NAME,
                    denial,
                ))
            })?
            .ok_or_else(|| {
                HandlerExecutionDenial::new(WorthQueryPriorOutputDenial::new(
                    WorthQueryPriorOutputDenialKind::MissingRole,
                    Role::NAME,
                ))
            })?;
        // Never fall back from an unavailable selected full generation.
        self.live_prior_identity::<Role::Entity>(self.reader.execution, Role::NAME, id)
            .map_err(HandlerExecutionDenial::new)
    }
}
