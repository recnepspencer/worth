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
        map_authorized_read_denial,
        pair_plans::WorthQueryDerivedPairReadPlans,
        plan_admission::{reserve_one_shot_result_buffer, validate_one_shot_plan},
    },
    read_execution::{
        prepared_pair::{PreparedPair, PreparedRead},
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
        validate_one_shot_plan(application, &self.first)
            .map_err(|denial| Denial::ReadDenied { root, denial })?;
        validate_one_shot_plan(application, &self.second)
            .map_err(|denial| Denial::ReadDenied { root, denial })?;
        refresh_governed_authorization(application, &mut self.first)
            .map_err(|read| Self::read_denial(root, read, self.first.query.name()))?;
        refresh_governed_authorization(application, &mut self.second)
            .map_err(|read| Self::read_denial(root, read, self.second.query.name()))?;
        let (_, first, _) = execute_authorized_read(application, &self.first, |_, _, _| Ok(()))
            .map_err(|read| Self::read_denial(root, read, self.first.query.name()))?;
        let (_, second, _) =
            execute_authorized_read(application, &self.second, |_, _, _| Ok(()))
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

    pub(super) fn worker(
        &self,
        application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        root: EntityId,
    ) -> Result<PreparedPair<'_>, Denial> {
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
        let first_buffer = reserve_one_shot_result_buffer(application, &self.first)
            .map_err(|denial| Denial::ReadDenied { root, denial })?;
        let second_buffer = reserve_one_shot_result_buffer(application, &self.second)
            .map_err(|denial| Denial::ReadDenied { root, denial })?;
        Ok(PreparedPair {
            root,
            #[cfg(test)]
            witness: DispatchWitness::carried(),
            first: PreparedRead {
                plan: ReadPlan::public_worker(&self.first),
                session: self
                    .first
                    .graph_work
                    .prepare_query_read(self.first.basis.identity())
                    .map_err(super::denial::session)?,
                buffer: first_buffer,
            },
            second: PreparedRead {
                plan: ReadPlan::public_worker(&self.second),
                session: self
                    .second
                    .graph_work
                    .prepare_query_read(self.second.basis.identity())
                    .map_err(super::denial::session)?,
                buffer: second_buffer,
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
        // Native plan custody carries its admitted index allowance and proof
        // estimate. Shared installed contracts and pinned truth are borrowed.
        for (review, graph, provider) in [
            (
                self.first.graph_read_plan(),
                &self.first.graph_authority_identity,
                &self.first.provider_identity,
            ),
            (
                self.second.graph_read_plan(),
                &self.second.graph_authority_identity,
                &self.second.provider_identity,
            ),
        ] {
            bytes = bytes
                .checked_add(review.budget_check().max_inline_index_bytes())
                .and_then(|bytes| {
                    bytes.checked_add(review.cost_estimate().supported().proof_bytes())
                })
                .and_then(|bytes| bytes.checked_add(graph.capacity()))
                .and_then(|bytes| bytes.checked_add(provider.capacity()))
                .ok_or(Denial::ChargedBytesOverflow)?;
        }
        u64::try_from(bytes).map_err(|_| Denial::ChargedBytesOverflow)
    }

    pub(super) fn projection_work(&self) -> Result<u64, Denial> {
        use crate::domain_computation::primary_graph::application_query::one_shot::custody_work::RetainedCustodyWork;
        let first = RetainedCustodyWork::of(
            self.first.scope.identity_locator(),
            self.first.scope.identity_value(),
            self.first.query.name(),
            self.first.basis.identity().branch_id(),
        )
        .and_then(RetainedCustodyWork::total);
        let second = RetainedCustodyWork::of(
            self.second.scope.identity_locator(),
            self.second.scope.identity_value(),
            self.second.query.name(),
            self.second.basis.identity().branch_id(),
        )
        .and_then(RetainedCustodyWork::total);
        first
            .and_then(|first| second.and_then(|second| first.checked_add(second)))
            .and_then(|units| u64::try_from(units).ok())
            .ok_or(Denial::WorkCounterOverflow)
    }

    pub(super) fn result_capacity(&self) -> Result<u64, Denial> {
        self.first
            .graph_read_plan()
            .budget_check()
            .max_inline_result_bytes()
            .checked_add(
                self.second
                    .graph_read_plan()
                    .budget_check()
                    .max_inline_result_bytes(),
            )
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(Denial::CapacityOverflow)
    }
}
