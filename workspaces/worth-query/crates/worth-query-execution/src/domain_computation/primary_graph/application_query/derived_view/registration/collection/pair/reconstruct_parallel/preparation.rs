//! Owner-side admission mints the concrete read capability and raw scope binding.

use super::super::Denial;
#[cfg(test)]
use crate::domain_computation::primary_graph::application_query::DispatchWitness;
use crate::domain_computation::primary_graph::application_query::{
    authorized_read::{
        execute_authorized_read, refresh_governed_authorization,
        WorthQueryAuthorizedApplicationReadDenial,
    },
    one_shot::{
        map_authorized_read_denial, pair_plans::WorthQueryDerivedPairReadPlans,
        plan_admission::validate_one_shot_plan,
    },
    read_execution::{
        prepared_pair::{batch::admit_batch_read, PreparedPair, PreparedRead},
        read_plan::ReadPlan,
    },
    WorthQueryApplicationAuthorizationWorkEvidence,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use worth_execution::MapKernelContext;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryAuthorization;
use worth_relational::facade::identity::EntityId;

impl<'a, S: ApplicationSchema, FQ, FP, FR, SQ, SP, SR, A, I, C>
    WorthQueryDerivedPairReadPlans<'a, S, FQ, FP, FR, SQ, SP, SR, A, I, C>
{
    pub(super) fn admit(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        root: EntityId,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<
        (
            WorthQueryApplicationAuthorizationWorkEvidence,
            WorthQueryApplicationAuthorizationWorkEvidence,
        ),
        Denial,
    > {
        self.admit_batch_reads(root)?;
        if self.first.scope.entity_id() != root
            || self.second.scope.entity_id() != root
            || (self.first.basis.identity().descriptor()
                != self.second.basis.identity().descriptor()
                || self.first.basis.identity().selection()
                    != self.second.basis.identity().selection())
        {
            return Err(Denial::IncompleteDependencies);
        }
        for authorization in [
            self.first.query.authorization(),
            self.second.query.authorization(),
        ] {
            if !matches!(
                authorization,
                WorthQueryInstalledApplicationQueryAuthorization::Public
            ) {
                return Err(Denial::AuthorizationRequired);
            }
        }
        let work = self
            .first
            .query
            .validation_work_bound()
            .and_then(|first| {
                self.second
                    .query
                    .validation_work_bound()
                    .and_then(|second| first.checked_add(second))
            })
            .ok_or(Denial::WorkCounterOverflow)?;
        context
            .checkpoint(work)
            .map_err(|stop| super::denial::kernel(stop, None))?;
        validate_one_shot_plan(execution, application, &self.first)
            .map_err(|denial| Denial::ReadDenied { root, denial })?;
        validate_one_shot_plan(execution, application, &self.second)
            .map_err(|denial| Denial::ReadDenied { root, denial })?;
        refresh_governed_authorization(execution, application, &mut self.first)
            .map_err(|read| Self::read_denial(root, read, self.first.query.name()))?;
        refresh_governed_authorization(execution, application, &mut self.second)
            .map_err(|read| Self::read_denial(root, read, self.second.query.name()))?;
        let (_, first, _) =
            execute_authorized_read(execution, application, &self.first, |_, _, _| Ok(()))
                .map_err(|read| Self::read_denial(root, read, self.first.query.name()))?;
        let (_, second, _) =
            execute_authorized_read(execution, application, &self.second, |_, _, _| Ok(()))
                .map_err(|read| Self::read_denial(root, read, self.second.query.name()))?;
        Ok((first, second))
    }

    fn read_denial(
        root: EntityId,
        read: WorthQueryAuthorizedApplicationReadDenial,
        name: &str,
    ) -> Denial {
        Denial::ReadDenied {
            root,
            denial: map_authorized_read_denial(read, name),
        }
    }

    #[cfg(test)]
    pub(super) fn worker(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,

        root: EntityId,
    ) -> Result<PreparedPair<'_>, Denial> {
        self.worker_with_pins(execution, root, &mut Default::default())
    }
    pub(super) fn worker_with_pins(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,

        root: EntityId,
        pins: &mut crate::domain_computation::primary_graph::application_query::one_shot::pair_plans::WorthQueryPairReadPins,
    ) -> Result<PreparedPair<'_>, Denial> {
        let first_batch = self.first_batch.take();
        let second_batch = self.second_batch.take();
        let locator = self.second.scope.identity_locator();
        let slot = self
            .first
            .query
            .read_family_binding()
            .planning_contract()
            .projections()
            .iter()
            .find(|field| {
                field.parent_path() == "root"
                    && field.aspect_key() == locator.aspect().aspect_key()
                    && *locator.field_path()
                        == worth_foundational::facade::CanonicalFieldPath::single(
                            field.field_key().clone(),
                        )
            })
            .ok_or(Denial::IncompleteDependencies)?;
        let first = match pins.first.as_ref() {
            Some(pin) => self.first.graph_work.prepare_query_read_from_pin(
                self.first.basis.identity(),
                execution,
                self.first.basis.snapshot_handle(),
                pin,
                |graph| ReadPlan::public_worker(&self.first).selected_indexes(graph),
            ),
            None => self.first.graph_work.prepare_query_read(
                self.first.basis.identity(),
                execution,
                self.first.basis.snapshot_handle(),
                |graph| ReadPlan::public_worker(&self.first).selected_indexes(graph),
            ),
        }
        .map_err(|cause| {
            super::denial::session(cause, root, &ReadPlan::public_worker(&self.first))
        })?;
        let second = match pins.second.as_ref() {
            Some(pin) => self.second.graph_work.prepare_query_read_from_pin(
                self.second.basis.identity(),
                execution,
                self.second.basis.snapshot_handle(),
                pin,
                |graph| ReadPlan::public_worker(&self.second).selected_indexes(graph),
            ),
            None => self.second.graph_work.prepare_query_read(
                self.second.basis.identity(),
                execution,
                self.second.basis.snapshot_handle(),
                |graph| ReadPlan::public_worker(&self.second).selected_indexes(graph),
            ),
        }
        .map_err(|cause| {
            super::denial::session(cause, root, &ReadPlan::public_worker(&self.second))
        })?;
        pins.first = Some(first.retained_pin());
        pins.second = Some(second.retained_pin());
        Ok(PreparedPair {
            root,
            #[cfg(test)]
            witness: DispatchWitness::carried(),
            first: PreparedRead {
                plan: ReadPlan::public_worker(&self.first),
                session: first,

                batch: first_batch,
            },
            second: PreparedRead {
                plan: ReadPlan::public_worker(&self.second),
                session: second,

                batch: second_batch,
            },
            binding_slot: slot.slot_type(),
            binding_value: self.second.scope.identity_value(),
        })
    }

    pub(super) fn retained_input_bound(&self) -> Result<u64, Denial> {
        let parameters = self
            .first
            .parameters
            .retained_owned_capacity_bytes()
            .and_then(|first| {
                self.second
                    .parameters
                    .retained_owned_capacity_bytes()
                    .and_then(|second| first.checked_add(second))
            })
            .ok_or(Denial::ChargedBytesOverflow)?;
        let mut bytes = parameters;
        // Shared installed contracts and pinned truth remain borrowed. Only
        // already-owned parameter and name backing accompanies these plans.
        for (graph, provider) in [
            (
                &self.first.graph_authority_identity,
                &self.first.provider_identity,
            ),
            (
                &self.second.graph_authority_identity,
                &self.second.provider_identity,
            ),
        ] {
            bytes = bytes
                .checked_add(graph.capacity())
                .and_then(|bytes| bytes.checked_add(provider.capacity()))
                .ok_or(Denial::ChargedBytesOverflow)?;
        }
        u64::try_from(bytes).map_err(|_| Denial::ChargedBytesOverflow)
    }

    fn admit_batch_reads(&mut self, root: EntityId) -> Result<(), Denial> {
        use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryBatchResourceDenial as Resource;
        let denied = |denial| Denial::BatchResource { root, denial };
        if self.first_batch.is_some() || self.second_batch.is_some() {
            return Err(denied(Resource::ForeignPlan));
        }
        if let Some(batch) = &self.batch {
            self.first_batch = Some(admit_batch_read(&mut self.first, batch).map_err(denied)?);
            self.second_batch = Some(admit_batch_read(&mut self.second, batch).map_err(denied)?);
        } else if self.first.planned_batch_item.is_some()
            || self.second.planned_batch_item.is_some()
        {
            return Err(denied(Resource::ForeignPlan));
        }
        Ok(())
    }
}
