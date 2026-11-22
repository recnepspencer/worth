//! One producer's declared old-read intent and exact Native succession selection.
use super::*;
use crate::domain_computation::primary_graph::{
    application_contribution::producer::{
        InstalledProducerEdition, WorthQueryApplicationProducerBinding,
    },
    RequiredOutputDemandContext, WorthQueryPrimaryGraphApplicationRuntime,
};
use std::sync::Arc;
use worth_query_execution_lineage::facade::ports::PrimaryGraphInternalPort as _;
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    identity::EntityId,
    mvcc::{
        RelationalPredecessorInput, RelationalRequiredOutputPublication,
        RequiredOutputProducingImplementation, RequiredOutputPublicationMeaning,
    },
    runtime::RelationalRuntime,
};

/// Current admission supplies authority separately. This carries only declared
/// operation membership and the running producer's authentic route coordinates.
pub struct ProducerPredecessorReadAccess {
    pub state: Arc<ProducerPredecessorReadState>,
    require: fn(&str, &str, EqualityTypeId) -> Result<(), WorthQueryProducerObservationReadDenial>,
    producer: &'static str,
    family: &'static str,
    epoch: [u8; 32],
    implementation: RequiredOutputProducingImplementation,
}
impl ProducerPredecessorReadAccess {
    pub fn prepare<S, B>(
        required: &mut RequiredOutputDemandContext,
        edition: InstalledProducerEdition,
    ) -> Option<Self>
    where
        S: ApplicationSchema + 'static,
        B: WorthQueryApplicationProducerBinding<S>,
    {
        let access = Self::for_route::<S, B>(
            required.publication_family(),
            required.key().source_epoch().checkpoint_identity().bytes(),
            edition,
        )?;
        required.bind_predecessor_reads(Arc::clone(&access.state));
        Some(access)
    }
    pub fn prepare_coupled<S, B>(
        handler: &mut crate::domain_computation::primary_graph::output_lineage::ProducerHandlerPreparation,
        epoch: [u8; 32],
        edition: InstalledProducerEdition,
    ) -> Option<Self>
    where
        S: ApplicationSchema + 'static,
        B: WorthQueryApplicationProducerBinding<S>,
    {
        let access = Self::for_route::<S, B>(
            <B::OutputFamily as super::super::WorthQueryProducerOutputFamily<S>>::IDENTITY,
            epoch,
            edition,
        )?;
        handler.bind_predecessor_reads(Arc::clone(&access.state));
        Some(access)
    }
    fn for_route<S, B>(
        family: &'static str,
        epoch: [u8; 32],
        edition: InstalledProducerEdition,
    ) -> Option<Self>
    where
        S: ApplicationSchema + 'static,
        B: WorthQueryApplicationProducerBinding<S>,
    {
        B::PREDECESSOR_READS?;
        Some(Self {
            state: Arc::new(ProducerPredecessorReadState::default()),
            require: require::<S, B>,
            producer: B::IDENTITY,
            family,
            epoch,
            implementation: RequiredOutputProducingImplementation::new(
                edition.build_identity(),
                edition.digest(),
            ),
        })
    }
    pub fn require<S, O>(&self) -> Result<(), WorthQueryProducerObservationReadDenial>
    where
        S: ApplicationSchema,
        O: ApplicationOperationMarkerIdentity<S> + 'static,
    {
        (self.require)(
            O::IDENTIFIER,
            O::InputBinding::IDENTITY_NAME,
            EqualityTypeId::of::<O>(),
        )
    }
    pub fn select<S: ApplicationSchema>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        application: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        native: &RelationalRuntime,
        selected: &AdmittedRelationalBranchBasis,
        scope: EntityId,
        partition: [u8; 32],
    ) -> Result<
        (
            RelationalRequiredOutputPublication,
            RelationalPredecessorInput,
        ),
        WorthQueryProducerObservationReadDenial,
    > {
        use WorthQueryProducerObservationReadDenial as Denial;
        let graph = application
            .runtime
            .primary_graph()
            .ok_or(Denial::PredecessorMismatch)?;
        let publisher = graph
            .application_publication_ref()
            .ok_or(Denial::PredecessorBasisUnavailable)?;
        let meaning = RequiredOutputPublicationMeaning {
            implementation: self.implementation,
            family: self.family.into(),
            source_scope: scope,
            source_partition: worth_foundational::CanonicalDigestId::new(partition),
            source_epoch: worth_foundational::CanonicalDigestId::new(self.epoch),
        };
        let prior = native
            .readmit_current_owned_output_successor(
                execution,
                &publisher.native,
                selected,
                self.producer,
                &meaning,
            )
            .map_err(|cause| {
                Denial::Native(
                    worth_relational::facade::mvcc::RelationalPredecessorBasisDenial::Selection(
                        cause,
                    ),
                )
            })?
            .ok_or(Denial::PredecessorBasisUnavailable)?;
        let expected = worth_relational::facade::history::CommitId(
            prior.authority().publication_coordinate().0,
        );
        let basis = native.predecessor_publication_input(
            execution,
            selected,
            Some(&prior),
            Some(expected),
        )?;
        let mut previous = self
            .state
            .selected
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if previous
            .as_ref()
            .is_some_and(|previous| !previous.same_origin(&prior))
        {
            return Err(Denial::PredecessorMismatch);
        }
        *previous = Some(prior.clone());
        Ok((prior, basis))
    }
}
fn require<S, B>(
    name: &str,
    input: &str,
    operation: EqualityTypeId,
) -> Result<(), WorthQueryProducerObservationReadDenial>
where
    S: ApplicationSchema + 'static,
    B: WorthQueryApplicationProducerBinding<S>,
{
    B::PREDECESSOR_READS
        .ok_or(WorthQueryProducerObservationReadDenial::UndeclaredObservationRead)?
        .require_identity(name, input, operation)
}
