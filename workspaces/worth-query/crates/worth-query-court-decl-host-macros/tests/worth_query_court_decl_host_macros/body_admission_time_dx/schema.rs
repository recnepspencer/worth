//! A help desk whose tickets are born by a declarative body that stamps
//! `created_at` from trusted admission time, and whose desk records the
//! instant a handler last noted it.
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

pub const DESK: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("body-admission-time.dx.v1");

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    schema,
    owner = "worth.body-admission-time.dx",
    name = "body_admission_time_dx",
    members(super::program::DeskContribution)
)]
pub struct DeskSchema;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    fragment,
    name = "body_admission_time_dx",
    bindings = Codecs,
    members(Owner, Mapping, Desk, Ticket)
)]
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
#[worth_query(entity, aspects(OwnerFacts), reference(DeskSchema, Records))]
pub struct Owner;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(
    entity,
    aspects(MappingFacts),
    relations(MappingOwner),
    reference(DeskSchema, Records)
)]
pub struct Mapping;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(entity, aspects(DeskFacts), reference(DeskSchema, Records))]
pub struct Desk;

#[derive(Clone, Copy, WorthQuerySchema)]
#[worth_query(entity, aspects(TicketFacts), reference(DeskSchema, Records))]
pub struct Ticket;

#[derive(WorthQueryRelation)]
#[worth_query(to = Owner, from = Mapping, reference(DeskSchema, Records))]
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

/// `noted_at` is written only by the noting handler from `admitted_at()`.
#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = DeskFacts, fragment = Records, entity = Desk)]
pub struct DeskRow {
    #[worth_query(equality, unique, read_only)]
    pub desk_id: u64,
    pub noted_at: CanonicalTimestamp,
}

/// `created_at` is read-only: only the birth body initializes it, from
/// trusted admission time. No caller input carries it.
#[derive(WorthQueryRecord)]
#[worth_query(bindings = Codecs, aspect = TicketFacts, fragment = Records, entity = Ticket)]
pub struct TicketRow {
    #[worth_query(equality, unique, read_only)]
    pub ticket_id: u64,
    #[worth_query(read_only)]
    pub created_at: CanonicalTimestamp,
}

worth_query_structured_value_binding!(pub BodyDenial for WorthQueryOperationBodyDenial {
    identity: worth_query_identity!(DESK, denial)
});

worth_query_principal_binding! {
    pub OwnerBinding in DeskSchema, mapping Mapping => Owner {
        identity: MappingFacts::external_identity(),
        status: MappingFacts::mapping_status(),
        target: MappingOwner,
        principal: OwnerId = OwnerFacts::owner_id(),
    }
    context pub Desktop {
        idempotency: u64 = worth_query_identity!(DESK, idempotency),
        denial: BodyDenial,
    }
}

#[derive(Clone, Debug, Eq, PartialEq, WorthQueryRow)]
#[worth_query(schema = DeskSchema, root = Ticket, namespace = DESK, member)]
pub struct TicketView {
    #[worth_query(field = TicketId)]
    pub id: u64,
    #[worth_query(field = CreatedAt)]
    pub created_at: CanonicalTimestamp,
}

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = DeskSchema, context = Desktop, namespace = DESK, member,
    result = TicketView, root = Ticket, scope = TicketId, cardinality = ExactlyOne, limits = 1,
)]
pub struct ReadTicket {
    #[worth_query(scope)]
    pub ticket: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, WorthQueryRow)]
#[worth_query(schema = DeskSchema, root = Desk, namespace = DESK, member)]
pub struct DeskView {
    #[worth_query(field = DeskId)]
    pub id: u64,
    #[worth_query(field = NotedAt)]
    pub noted_at: CanonicalTimestamp,
}

#[derive(Clone, Debug, WorthQuery)]
#[worth_query(
    schema = DeskSchema, context = Desktop, namespace = DESK, member,
    result = DeskView, root = Desk, scope = DeskId, cardinality = ExactlyOne, limits = 1,
)]
pub struct ReadDesk {
    #[worth_query(scope)]
    pub desk: u64,
}

/// Opens a ticket on a desk. The caller names the ticket; the body stamps
/// its birth instant.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, WorthQueryAction)]
#[worth_query(
    schema = DeskSchema, context = Desktop, namespace = DESK, member,
    input = OpenTicketInput, operation = OpenTicketOp, binding = OpenTicketBinding,
    delivery = TicketOpened, body, program, reads(Desk), creates(Ticket),
)]
pub struct OpenTicket {
    #[worth_query(scope = DeskId)]
    pub desk: u64,
    pub ticket: u64,
}

/// One creation, two initializers: the caller's ticket number and the
/// trusted admission instant.
pub fn open_ticket_operation(
) -> ApplicationOperationDefinition<DeskSchema, OpenTicketOp, OpenTicket> {
    OpenTicketOp::reference()
        .typed_body()
        .in_scope(Desk::reference())
        .create_in_scope(Ticket::reference(), "ticket")
        .initialize_input(TicketFacts::ticket_id(), OpenTicket::ticket_input())
        .initialize_from_admission_time(TicketFacts::created_at())
        .finish_pure_definition()
        .expect("open ticket body admits")
}

/// Notes a desk at the handler's trusted admission instant.
#[derive(Clone, Debug, Serialize, Deserialize, WorthQueryAction)]
#[worth_query(
    schema = DeskSchema, context = Desktop, namespace = DESK, member,
    input = NoteDeskInput, operation = NoteDeskOp, binding = NoteDeskBinding,
    delivery = DeskNoted, result = UnitApplicationValueBinding, decision = NoteDecision,
    program,
    access(
        read_entities: [Desk];
        read_fields: [DeskFacts::desk_id(), DeskFacts::noted_at()];
        writes: [DeskFacts::noted_at()];
    )
)]
pub struct NoteDesk {
    #[worth_query(scope = DeskId)]
    pub desk: u64,
}

/// The desk to write and the admission instant the handler observed.
pub struct NoteDecision {
    pub target: WorthQueryInvariantMutationTarget<DeskSchema, Desk>,
    pub noted_at: CanonicalTimestamp,
}
