//! A lot ledger whose caller-supplied dates are judged against trusted
//! admission time: lots carry an effective date, an expiry date, and the
//! admission instant their last stamping handler observed.
use serde::{Deserialize, Serialize};
use worth_query_decl::facade::{
    application_identity::ApplicationIdentityNamespace,
    application_schema::{fragment::*, *},
    authentication::*,
    worth_query_identity, worth_query_identity_namespace, worth_query_native_scalar_bindings,
    worth_query_principal_binding, worth_query_structured_value_binding, WorthQuery,
    WorthQueryAction, WorthQueryRecord, WorthQueryRelation, WorthQueryRow, WorthQuerySchema,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryInvariantMutationTarget, WorthQueryOperationBodyDenial,
};

pub const DATED: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("admission-time.dx.v1");

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    schema,
    owner = "worth.admission-time.dx",
    name = "admission_time_dx",
    members(super::program::DatedContribution)
)]
pub struct DatedSchema;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(fragment, name = "admission_time_dx", bindings = Codecs, members(Owner, Mapping, Lot))]
pub struct Records;

worth_query_native_scalar_bindings!(pub Codecs);
impl Scalar<WorthQueryExternalPrincipalIdentity> for Codecs {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const WRITABLE: bool = false;
}
impl Scalar<WorthQueryPrincipalMappingStatus> for Codecs {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
}

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(entity, aspects(OwnerFacts), reference(DatedSchema, Records))]
pub struct Owner;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    entity,
    aspects(MappingFacts),
    relations(MappingOwner),
    reference(DatedSchema, Records)
)]
pub struct Mapping;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(entity, aspects(LotFacts), reference(DatedSchema, Records))]
pub struct Lot;

#[derive(WorthQueryRelation)]
#[worth_query(to = Owner, from = Mapping, reference(DatedSchema, Records))]
pub struct MappingOwner;

#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = OwnerFacts, fragment = Records, entity = Owner)]
pub struct OwnerRow {
    #[worth_query(equality, read_only)]
    pub owner_id: u64,
}

#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = MappingFacts, fragment = Records, entity = Mapping)]
pub struct MappingRow {
    #[worth_query(equality)]
    pub external_identity: WorthQueryExternalPrincipalIdentity,
    #[worth_query(equality)]
    pub mapping_status: WorthQueryPrincipalMappingStatus,
}

/// `stamped_at` is written only by the stamping handler, from its trusted
/// `admitted_at()`; callers supply only the two dates.
#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = LotFacts, fragment = Records, entity = Lot)]
pub struct LotRow {
    #[worth_query(equality, unique, read_only)]
    pub lot_id: u64,
    pub effective_on: CanonicalDate,
    pub expires_on: CanonicalDate,
    pub stamped_at: CanonicalTimestamp,
}

worth_query_structured_value_binding!(pub BodyDenial for WorthQueryOperationBodyDenial {
    identity: worth_query_identity!(DATED, denial)
});

worth_query_principal_binding! {
    pub OwnerBinding in DatedSchema, mapping Mapping => Owner {
        identity: MappingFacts::external_identity(),
        status: MappingFacts::mapping_status(),
        target: MappingOwner,
        principal: OwnerId = OwnerFacts::owner_id(),
    }
    context pub Desktop {
        idempotency: u64 = worth_query_identity!(DATED, idempotency),
        denial: BodyDenial,
    }
}

#[derive(Clone, Debug, Eq, PartialEq, WorthQueryRow)]
#[worth_query(schema = DatedSchema, root = Lot, namespace = DATED, member)]
pub struct LotView {
    #[worth_query(field = LotId)]
    pub id: u64,
    #[worth_query(field = EffectiveOn)]
    pub effective_on: CanonicalDate,
    #[worth_query(field = ExpiresOn)]
    pub expires_on: CanonicalDate,
    #[worth_query(field = StampedAt)]
    pub stamped_at: CanonicalTimestamp,
}

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = DatedSchema, context = Desktop, namespace = DATED, member,
    result = LotView, root = Lot, scope = LotId, cardinality = ExactlyOne, limits = 1,
)]
pub struct ReadLot {
    #[worth_query(scope)]
    pub lot: u64,
}

/// Records a lot's caller-supplied effective and expiry dates. The handler
/// stamps its trusted admission instant; the commit invariant judges both
/// dates against that same instant.
#[derive(Clone, Debug, Serialize, Deserialize, WorthQueryAction)]
#[worth_query(
    schema = DatedSchema, context = Desktop, namespace = DATED, member,
    input = StampLotInput, operation = StampLotOp, binding = StampLotBinding,
    delivery = LotStamped, result = UnitApplicationValueBinding, decision = StampDecision,
    program,
    access(
        read_entities: [Lot];
        read_fields: [
            LotFacts::lot_id(), LotFacts::effective_on(), LotFacts::expires_on(),
            LotFacts::stamped_at()
        ];
        writes: [LotFacts::effective_on(), LotFacts::expires_on(), LotFacts::stamped_at()];
    )
)]
pub struct StampLot {
    #[worth_query(scope = LotId)]
    pub lot: u64,
    pub effective_on: CanonicalDate,
    pub expires_on: CanonicalDate,
}

/// The lot to write and the admission instant the handler observed.
pub struct StampDecision {
    pub target: WorthQueryInvariantMutationTarget<DatedSchema, Lot>,
    pub stamped_at: CanonicalTimestamp,
}
