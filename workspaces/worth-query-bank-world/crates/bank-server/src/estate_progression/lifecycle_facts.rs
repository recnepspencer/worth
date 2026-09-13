use bank_domain::{
    estate::{EmergencyAccessId, MandatoryReviewId},
    schema::{
        ApproveEstateEmergencyAccessOperation, BankSchema, CompleteEstateMandatoryReviewOperation,
        EmergencyAccess, EmergencyAccessExpiresAtField, EmergencyAccessIdBinding,
        EmergencyAccessIdentityField, EmergencyAccessIssuedAtField, EmergencyAccessReasonField,
        EmergencyAccessStatusField, EmergencyApprover, EmergencyEstate, EmergencyGrant,
        EmergencyRequester, EmergencyReview, EstateCase, MandatoryReview, MandatoryReviewIdBinding,
        MandatoryReviewIdentityField, MandatoryReviewKindField, MandatoryReviewStatusField,
        ReviewEstate, ReviewPrincipal, RevokeEstateEmergencyAccessOperation,
    },
};
use worth_query_host::facade::{
    declaration::application_schema::{ApplicationReadableScalarValueBinding, OperationReads},
    primary_graph::{
        WorthQueryApplicationOperationInvariantProjectionReader, WorthQueryInvariantEntityIdentity,
        WorthQueryRequestedElevation,
    },
};

use super::BankEstateLifecycleProjectionDenial;

type ElevationIdentity = WorthQueryInvariantEntityIdentity<BankSchema, EmergencyAccess>;
type ReviewIdentity = WorthQueryInvariantEntityIdentity<BankSchema, MandatoryReview>;
type EstateIdentity = WorthQueryInvariantEntityIdentity<BankSchema, EstateCase>;

pub(super) fn approval_lifecycle_identities(
    requested: &WorthQueryRequestedElevation,
) -> Result<(EmergencyAccessId, MandatoryReviewId), BankEstateLifecycleProjectionDenial> {
    let access = EmergencyAccessIdBinding::decode(requested.elevation_identity())
        .map_err(|_| BankEstateLifecycleProjectionDenial::ReceiptIdentity("emergency access"))?;
    let review = MandatoryReviewIdBinding::decode(requested.review_identity())
        .map_err(|_| BankEstateLifecycleProjectionDenial::ReceiptIdentity("mandatory review"))?;
    Ok((access, review))
}

pub(super) fn seal_approval_lifecycle_facts(
    reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
        BankSchema,
        ApproveEstateEmergencyAccessOperation,
    >,
    access: EmergencyAccessId,
    review: MandatoryReviewId,
    estate: &EstateIdentity,
) -> Result<(), BankEstateLifecycleProjectionDenial> {
    seal_selected_lifecycle_facts(reader, access, review, estate)
}

pub(super) fn seal_close_lifecycle_facts(
    reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
        BankSchema,
        RevokeEstateEmergencyAccessOperation,
    >,
    access: EmergencyAccessId,
    estate: &EstateIdentity,
) -> Result<(), BankEstateLifecycleProjectionDenial> {
    let elevation = reader.resolve_entity(EmergencyAccessIdentityField::reference(), access)?;
    let review_relations =
        reader.decision_relations_from(EmergencyReview::reference(), &elevation)?;
    let [review_relation] = review_relations.as_slice() else {
        return Err(BankEstateLifecycleProjectionDenial::RelationCardinality {
            relation: "EmergencyReview",
            expected: 1,
            observed: review_relations.len(),
        });
    };
    let review = review_relation.to().clone();
    seal_lifecycle_fields(reader, &elevation, &review)?;
    seal_remaining_lifecycle_relations(reader, &elevation, &review, estate)
}

pub(super) fn seal_review_lifecycle_facts(
    reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<
        BankSchema,
        CompleteEstateMandatoryReviewOperation,
    >,
    access: EmergencyAccessId,
    review: MandatoryReviewId,
    estate: &EstateIdentity,
) -> Result<(), BankEstateLifecycleProjectionDenial> {
    seal_selected_lifecycle_facts(reader, access, review, estate)
}

fn seal_selected_lifecycle_facts<Operation>(
    reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<BankSchema, Operation>,
    access: EmergencyAccessId,
    review: MandatoryReviewId,
    estate: &EstateIdentity,
) -> Result<(), BankEstateLifecycleProjectionDenial>
where
    EmergencyAccessIdentityField: OperationReads<Operation>,
    EmergencyAccessReasonField: OperationReads<Operation>,
    EmergencyAccessStatusField: OperationReads<Operation>,
    EmergencyAccessIssuedAtField: OperationReads<Operation>,
    EmergencyAccessExpiresAtField: OperationReads<Operation>,
    MandatoryReviewIdentityField: OperationReads<Operation>,
    MandatoryReviewKindField: OperationReads<Operation>,
    MandatoryReviewStatusField: OperationReads<Operation>,
    EmergencyRequester: OperationReads<Operation>,
    EmergencyApprover: OperationReads<Operation>,
    EmergencyGrant: OperationReads<Operation>,
    EmergencyEstate: OperationReads<Operation>,
    EmergencyReview: OperationReads<Operation>,
    ReviewEstate: OperationReads<Operation>,
    ReviewPrincipal: OperationReads<Operation>,
{
    let elevation = reader.resolve_entity(EmergencyAccessIdentityField::reference(), access)?;
    let review = reader.resolve_entity(MandatoryReviewIdentityField::reference(), review)?;
    seal_lifecycle_fields(reader, &elevation, &review)?;
    reader.decision_relations_from(EmergencyReview::reference(), &elevation)?;
    seal_remaining_lifecycle_relations(reader, &elevation, &review, estate)
}

fn seal_lifecycle_fields<Operation>(
    reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<BankSchema, Operation>,
    elevation: &ElevationIdentity,
    review: &ReviewIdentity,
) -> Result<(), BankEstateLifecycleProjectionDenial>
where
    EmergencyAccessIdentityField: OperationReads<Operation>,
    EmergencyAccessReasonField: OperationReads<Operation>,
    EmergencyAccessStatusField: OperationReads<Operation>,
    EmergencyAccessIssuedAtField: OperationReads<Operation>,
    EmergencyAccessExpiresAtField: OperationReads<Operation>,
    MandatoryReviewIdentityField: OperationReads<Operation>,
    MandatoryReviewKindField: OperationReads<Operation>,
    MandatoryReviewStatusField: OperationReads<Operation>,
{
    reader.require_decision_field(elevation, EmergencyAccessIdentityField::reference())?;
    reader.require_decision_field(elevation, EmergencyAccessReasonField::reference())?;
    reader.require_decision_field(elevation, EmergencyAccessStatusField::reference())?;
    reader.require_decision_field(elevation, EmergencyAccessIssuedAtField::reference())?;
    reader.require_decision_field(elevation, EmergencyAccessExpiresAtField::reference())?;
    reader.require_decision_field(review, MandatoryReviewIdentityField::reference())?;
    reader.require_decision_field(review, MandatoryReviewKindField::reference())?;
    reader.require_decision_field(review, MandatoryReviewStatusField::reference())?;
    Ok(())
}

fn seal_remaining_lifecycle_relations<Operation>(
    reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<BankSchema, Operation>,
    elevation: &ElevationIdentity,
    review: &ReviewIdentity,
    estate: &EstateIdentity,
) -> Result<(), BankEstateLifecycleProjectionDenial>
where
    EmergencyRequester: OperationReads<Operation>,
    EmergencyApprover: OperationReads<Operation>,
    EmergencyGrant: OperationReads<Operation>,
    EmergencyEstate: OperationReads<Operation>,
    ReviewEstate: OperationReads<Operation>,
    ReviewPrincipal: OperationReads<Operation>,
{
    reader.decision_relations_to(EmergencyRequester::reference(), elevation)?;
    reader.decision_relations_to(EmergencyApprover::reference(), elevation)?;
    reader.decision_relations_from(EmergencyGrant::reference(), elevation)?;
    seal_resource_relation(reader, elevation, estate)?;
    reader.decision_relations_from(ReviewEstate::reference(), review)?;
    reader.decision_relations_to(ReviewPrincipal::reference(), review)?;
    Ok(())
}

fn seal_resource_relation<Operation>(
    reader: &mut WorthQueryApplicationOperationInvariantProjectionReader<BankSchema, Operation>,
    elevation: &ElevationIdentity,
    estate: &EstateIdentity,
) -> Result<(), BankEstateLifecycleProjectionDenial>
where
    EmergencyEstate: OperationReads<Operation>,
{
    let relations = reader.decision_relations_from(EmergencyEstate::reference(), elevation)?;
    let [relation] = relations.as_slice() else {
        return Err(BankEstateLifecycleProjectionDenial::RelationCardinality {
            relation: "EmergencyEstate",
            expected: 1,
            observed: relations.len(),
        });
    };
    if relation.to() == estate {
        Ok(())
    } else {
        Err(
            BankEstateLifecycleProjectionDenial::RelationTargetMismatch {
                relation: "EmergencyEstate",
            },
        )
    }
}
