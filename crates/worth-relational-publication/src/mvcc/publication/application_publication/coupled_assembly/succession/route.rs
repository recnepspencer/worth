//! A successor round continues exactly its chain's route.
use super::super::producer_posture::RelationalCoupledProducerPosture;
use super::super::refusals::reject;
use contracts::identity::data::record_ids::EntityId;
use contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaningOwnerPort as _;
use contracts::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal as Cause;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_relational_contracts::internal_port as contracts;

/// The same route, producer and posture kind as the chain's head, or
/// `RepeatedCoupledOwner`. Returns the route source.
pub fn require_same_route(
    head: &RelationalCoupledProducerPosture,
    publication: &RelationalCoupledProducerPosture,
) -> Result<EntityId, TransactionCommitError> {
    let meaning = head.producing_meaning().expect("admitted producer route");
    let source = meaning.source_scope;
    let same = match (
        publication.producing_meaning(),
        head.owned(),
        publication.owned(),
    ) {
        (Some(next), Some(prior), Some(owned)) => {
            next.same_route(meaning)
                && publication
                    .lineage_attribution()
                    .map(|(producer, _)| producer)
                    == head.lineage_attribution().map(|(producer, _)| producer)
                && std::mem::discriminant(prior) == std::mem::discriminant(owned)
        }
        _ => false,
    };
    if !same {
        return Err(reject(Cause::RepeatedCoupledOwner, source));
    }
    Ok(source)
}
