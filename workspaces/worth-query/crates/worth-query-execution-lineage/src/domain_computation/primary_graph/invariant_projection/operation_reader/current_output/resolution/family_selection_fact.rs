//! Record a `current_output` selection as a verifiable `FamilySelection`
//! fact (public #429): Missing, Unique and Ambiguous alike.

use super::*;
use crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding;
use crate::domain_computation::primary_graph::application_attempt::FamilySelectionRead;
use crate::domain_computation::primary_graph::output_lineage::family_selection::{
    FamilySelectionProducer, ObservedFamilySelection,
};
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::mvcc::CompanionPreflightStop;

impl<'reader, 'runtime, Schema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'reader, 'runtime, Schema, Operation>
where
    Schema: ApplicationSchema,
{
    /// `outcome` holds the committed members the read selected and `applied`
    /// the members its applied overlay supplied, masking `replaced`.
    pub(super) fn record_family_selection<Family, Producer>(
        &mut self,
        producer: &WorthQueryInvariantEntityIdentity<Schema, Producer>,
        sources: Vec<SemanticSource>,
        outcome: Vec<EntityId>,
        applied: Vec<EntityId>,
        replaced: &[(Option<TypeId>, [u8; 32])],
    ) -> Result<(), WorthQueryCurrentOutputDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        Producer: ApplicationEntityMarkerIdentity<Schema>,
    {
        let unavailable = || {
            WorthQueryCurrentOutputDenial::new(
                WorthQueryCurrentOutputDenialKind::OutputUnavailable,
                Family::IDENTITY,
            )
        };
        let admission = self.admission_identity.ok_or_else(unavailable)?;
        let observation = ObservedFamilySelection::new(
            FamilySelectionProducer {
                runtime_authority: self.runtime_authority.as_u64(),
                schema: self.binding_identity.clone(),
                scope: WorthQueryOperationScopeEntityBinding::from_entity(producer.entity_id()),
                family: Family::IDENTITY,
                entity_type: TypeId::of::<Family::Entity>(),
                entity_identifier:
                    <Family::Entity as ApplicationEntityMarkerIdentity<Schema>>::IDENTIFIER,
                producer: producer.entity_id(),
            },
            sources,
            outcome,
            applied,
            replaced,
            &self.reader.output_lineage,
            &self.reader.invalidation_owner,
            self.reader.layout,
        );
        self.decision_facts
            .record_family_selection(
                FamilySelectionRead::new(admission, Family::IDENTITY, producer.entity_id()),
                std::sync::Arc::new(observation),
            )
            .map_err(|stop| match stop {
                CompanionPreflightStop::Execution(cause) => WorthQueryCurrentOutputDenial::new(
                    WorthQueryCurrentOutputDenialKind::ExecutionRequest(cause),
                    Family::IDENTITY,
                ),
                _ => unavailable(),
            })
    }
}
