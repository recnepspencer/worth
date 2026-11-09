use bank_domain::{
    estate::{EstateCaseId, EstateCaseStatus, EstateDisbursement, LegalAuthorityId},
    model::BankPrincipalId,
    proposals::BankDecisionSnapshot,
    schema::*,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryEntityResolutionDenial, WorthQueryInvariantAggregateDenial,
    WorthQueryInvariantDecisionPlanDenial, WorthQueryInvariantEntityIdentity,
    WorthQueryInvariantProjectionTraversalDenial,
};

use super::{
    account_balance::validated_account_balance,
    bounded::{BoundedProjectionState, ProjectionReader},
    BankProjectionDenial,
};

#[derive(Debug)]
pub enum BankEstateDisbursementProjectionDenial {
    EstateIdentityMissing,
    EstateIdentityMismatch,
    EstateStatusMissing,
    EstateNotOpen(EstateCaseStatus),
    EstateAccountCardinality(usize),
    EstateAccountMismatch,
    EstateBeneficiaryRelationMissing,
    EstateJointOwnerRelationMissing,
    RecognizedExecutorAuthorityMissing,
    LegalAuthorityIdentityMissing,
    LegalAuthorityIdentityMismatch,
    LegalAuthorityRecognitionMissing,
    LegalAuthorityEstateCardinality(usize),
    LegalAuthorityEstateMismatch,
    LegalAuthorityHolderCardinality(usize),
    LegalAuthorityHolderIdentityMissing,
    LegalAuthorityHolderIdentityMismatch,
    WitnessRelationCardinality {
        relation: &'static str,
        observed: usize,
    },
    Projection(BankProjectionDenial),
}

pub(crate) struct BankEstateDisbursementDecision {
    snapshot: BankDecisionSnapshot,
    authority: LegalAuthorityId,
    executor: BankPrincipalId,
}

impl BankEstateDisbursementDecision {
    pub(crate) fn into_parts(self) -> (BankDecisionSnapshot, LegalAuthorityId, BankPrincipalId) {
        (self.snapshot, self.authority, self.executor)
    }
}

pub(crate) fn project_estate_disbursement(
    reader: &mut ProjectionReader<'_, '_, DisburseEstateOperation>,
    estate: &WorthQueryInvariantEntityIdentity<BankSchema, EstateCase>,
    input: &EstateDisbursement,
) -> Result<BankEstateDisbursementDecision, BankEstateDisbursementProjectionDenial> {
    project_estate(reader, estate, input.estate)?;
    let source = project_estate_source(reader, estate, input)?;
    let destination = reader.resolve_entity(
        AccountIdentity::at("AccountIdentity"),
        input.destination_account,
    )?;
    let mut state = BoundedProjectionState::new(reader)?;
    let source_revision = state.project_admitted_account(reader, &source, input.source_account)?;
    let destination_revision =
        state.project_admitted_account(reader, &destination, input.destination_account)?;
    project_beneficiary(reader, &mut state, estate, &destination, input)?;
    let (authority, executor) = require_recognized_executor_authority(reader, estate)?;
    let source_balance = validated_account_balance(
        input.source_account,
        source_revision,
        reader.summarize_exclusive_incoming(
            PostingAccount::reference(),
            PostingAmount::at("PostingAmount"),
            &source,
        )?,
    )?;
    let destination_balance = validated_account_balance(
        input.destination_account,
        destination_revision,
        reader.summarize_exclusive_incoming(
            PostingAccount::reference(),
            PostingAmount::at("PostingAmount"),
            &destination,
        )?,
    )?;
    let snapshot = state
        .finish()
        .build_decision_projection_with_balances(
            [input.source_account],
            [
                (input.source_account, source_balance),
                (input.destination_account, destination_balance),
            ],
        )
        .map_err(BankProjectionDenial::InvalidDomainState)?;
    Ok(BankEstateDisbursementDecision {
        snapshot,
        authority,
        executor,
    })
}

fn project_estate(
    reader: &mut ProjectionReader<'_, '_, DisburseEstateOperation>,
    estate: &WorthQueryInvariantEntityIdentity<BankSchema, EstateCase>,
    expected: EstateCaseId,
) -> Result<(), BankEstateDisbursementProjectionDenial> {
    let identity = reader
        .decision_field(
            estate,
            EstateCaseIdentityField::at("EstateCaseIdentityField"),
        )?
        .ok_or(BankEstateDisbursementProjectionDenial::EstateIdentityMissing)?;
    if identity != expected {
        return Err(BankEstateDisbursementProjectionDenial::EstateIdentityMismatch);
    }
    let status = reader
        .decision_field(estate, EstateCaseStatusField::at("EstateCaseStatusField"))?
        .ok_or(BankEstateDisbursementProjectionDenial::EstateStatusMissing)?;
    if status != EstateCaseStatus::Open {
        return Err(BankEstateDisbursementProjectionDenial::EstateNotOpen(
            status,
        ));
    }
    Ok(())
}

fn project_estate_source(
    reader: &mut ProjectionReader<'_, '_, DisburseEstateOperation>,
    estate: &WorthQueryInvariantEntityIdentity<BankSchema, EstateCase>,
    input: &EstateDisbursement,
) -> Result<super::bounded::AccountEntity, BankEstateDisbursementProjectionDenial> {
    let relations = reader.decision_relations_from(EstateAccount::reference(), estate)?;
    let [relation] = relations.as_slice() else {
        return Err(
            BankEstateDisbursementProjectionDenial::EstateAccountCardinality(relations.len()),
        );
    };
    let source =
        reader.resolve_entity(AccountIdentity::at("AccountIdentity"), input.source_account)?;
    if relation.to() != &source {
        return Err(BankEstateDisbursementProjectionDenial::EstateAccountMismatch);
    }
    reader.require_decision_relation(EstateAccount::reference(), estate, &source)?;
    Ok(source)
}

fn project_beneficiary(
    reader: &mut ProjectionReader<'_, '_, DisburseEstateOperation>,
    state: &mut BoundedProjectionState,
    estate: &WorthQueryInvariantEntityIdentity<BankSchema, EstateCase>,
    destination: &super::bounded::AccountEntity,
    input: &EstateDisbursement,
) -> Result<(), BankEstateDisbursementProjectionDenial> {
    let beneficiary = state.project_principal(reader, input.beneficiary)?;
    if !reader
        .relations_from(EstateBeneficiary::reference(), &beneficiary)?
        .iter()
        .any(|relation| relation.to() == estate)
    {
        return Err(BankEstateDisbursementProjectionDenial::EstateBeneficiaryRelationMissing);
    }
    if !reader
        .relations_from(EstateJointOwner::reference(), &beneficiary)?
        .iter()
        .any(|relation| relation.to() == destination)
    {
        return Err(BankEstateDisbursementProjectionDenial::EstateJointOwnerRelationMissing);
    }
    reader.require_decision_relation(EstateBeneficiary::reference(), &beneficiary, estate)?;
    reader.require_decision_relation(EstateJointOwner::reference(), &beneficiary, destination)?;
    Ok(())
}

fn require_recognized_executor_authority(
    reader: &mut ProjectionReader<'_, '_, DisburseEstateOperation>,
    estate: &WorthQueryInvariantEntityIdentity<BankSchema, EstateCase>,
) -> Result<(LegalAuthorityId, BankPrincipalId), BankEstateDisbursementProjectionDenial> {
    let authorities = reader.decision_relations_to(LegalAuthorityEstate::reference(), estate)?;
    let mut selected = None;
    for relation in authorities {
        let authority = relation.from();
        let authority_id = validate_authority_identity(reader, authority)?;
        let recognized = reader
            .decision_field(
                authority,
                LegalAuthorityRecognizedField::at("LegalAuthorityRecognizedField"),
            )?
            .ok_or(BankEstateDisbursementProjectionDenial::LegalAuthorityRecognitionMissing)?;
        if !recognized {
            continue;
        }
        validate_authority_estate(reader, authority, estate)?;
        let (holder, executor_id) = exact_authority_holder(reader, authority)?;
        let executor_relations = reader.relations_from(EstateExecutor::reference(), &holder)?;
        if executor_relations
            .iter()
            .any(|executor| executor.to() == estate)
            && selected.is_none_or(|(selected_authority, _)| authority_id < selected_authority)
        {
            selected = Some((authority_id, executor_id));
        }
    }
    let (authority_id, executor_id) = selected
        .ok_or(BankEstateDisbursementProjectionDenial::RecognizedExecutorAuthorityMissing)?;
    let authority = reader.resolve_entity(
        LegalAuthorityIdentityField::at("LegalAuthorityIdentityField"),
        authority_id,
    )?;
    let executor = reader.resolve_entity(
        PrincipalIdentityField::at("PrincipalIdentityField"),
        executor_id,
    )?;
    reader.require_decision_relation(LegalAuthorityEstate::reference(), &authority, estate)?;
    reader.require_decision_relation(LegalAuthorityHolder::reference(), &authority, &executor)?;
    reader.require_decision_relation(EstateExecutor::reference(), &executor, estate)?;
    Ok((authority_id, executor_id))
}

fn validate_authority_identity(
    reader: &mut ProjectionReader<'_, '_, DisburseEstateOperation>,
    authority: &WorthQueryInvariantEntityIdentity<BankSchema, LegalAuthority>,
) -> Result<LegalAuthorityId, BankEstateDisbursementProjectionDenial> {
    let identity = reader
        .decision_field(
            authority,
            LegalAuthorityIdentityField::at("LegalAuthorityIdentityField"),
        )?
        .ok_or(BankEstateDisbursementProjectionDenial::LegalAuthorityIdentityMissing)?;
    let canonical = reader.resolve_entity(
        LegalAuthorityIdentityField::at("LegalAuthorityIdentityField"),
        identity,
    )?;
    if &canonical != authority {
        return Err(BankEstateDisbursementProjectionDenial::LegalAuthorityIdentityMismatch);
    }
    Ok(identity)
}

fn validate_authority_estate(
    reader: &mut ProjectionReader<'_, '_, DisburseEstateOperation>,
    authority: &WorthQueryInvariantEntityIdentity<BankSchema, LegalAuthority>,
    estate: &WorthQueryInvariantEntityIdentity<BankSchema, EstateCase>,
) -> Result<(), BankEstateDisbursementProjectionDenial> {
    let estates = reader.decision_relations_from(LegalAuthorityEstate::reference(), authority)?;
    let [relation] = estates.as_slice() else {
        return Err(
            BankEstateDisbursementProjectionDenial::LegalAuthorityEstateCardinality(estates.len()),
        );
    };
    if relation.to() != estate {
        return Err(BankEstateDisbursementProjectionDenial::LegalAuthorityEstateMismatch);
    }
    Ok(())
}

fn exact_authority_holder(
    reader: &mut ProjectionReader<'_, '_, DisburseEstateOperation>,
    authority: &WorthQueryInvariantEntityIdentity<BankSchema, LegalAuthority>,
) -> Result<
    (
        WorthQueryInvariantEntityIdentity<BankSchema, Principal>,
        BankPrincipalId,
    ),
    BankEstateDisbursementProjectionDenial,
> {
    let holders = reader.decision_relations_from(LegalAuthorityHolder::reference(), authority)?;
    let [holder] = holders.as_slice() else {
        return Err(
            BankEstateDisbursementProjectionDenial::LegalAuthorityHolderCardinality(holders.len()),
        );
    };
    let holder = holder.to().clone();
    let identity = reader
        .decision_field(
            &holder,
            PrincipalIdentityField::at("PrincipalIdentityField"),
        )?
        .ok_or(BankEstateDisbursementProjectionDenial::LegalAuthorityHolderIdentityMissing)?;
    let canonical = reader.resolve_entity(
        PrincipalIdentityField::at("PrincipalIdentityField"),
        identity,
    )?;
    if canonical != holder {
        return Err(BankEstateDisbursementProjectionDenial::LegalAuthorityHolderIdentityMismatch);
    }
    Ok((canonical, identity))
}

impl From<BankProjectionDenial> for BankEstateDisbursementProjectionDenial {
    fn from(denial: BankProjectionDenial) -> Self {
        Self::Projection(denial)
    }
}

macro_rules! projection_conversion {
    ($denial:ty) => {
        impl From<$denial> for BankEstateDisbursementProjectionDenial {
            fn from(denial: $denial) -> Self {
                Self::Projection(denial.into())
            }
        }
    };
}

projection_conversion!(WorthQueryEntityResolutionDenial);
projection_conversion!(WorthQueryInvariantAggregateDenial);
projection_conversion!(WorthQueryInvariantDecisionPlanDenial);
projection_conversion!(WorthQueryInvariantProjectionTraversalDenial);

impl std::fmt::Display for BankEstateDisbursementProjectionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for BankEstateDisbursementProjectionDenial {}
