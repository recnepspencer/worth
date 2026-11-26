//! An undoable expiry change: `ExtendLot` sets a lot's expiry, and its
//! `authored_body` correction restores the previous one, so undo and redo
//! commit through `TrustedDates` like any other operation.
use worth_query_decl::facade::application_aftermath::*;
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::{
    worth_query_identity_namespace, WorthQueryAction, WorthQueryCorrection,
};

use super::program::{DatedProgram, ExtendLotAction, RestoreLotExpiryAction};
use super::schema::*;

pub const EXTEND: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("admission-time.dx.extend.v1");
pub const RESTORE: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("admission-time.dx.restore.v1");
pub const CORRECT: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("admission-time.dx.correct.v1");

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, WorthQueryAction)]
#[worth_query(
    schema = DatedSchema, context = Desktop, namespace = EXTEND,
    input = ExtendLotInput, operation = ExtendLotOp, binding = ExtendLotBinding,
    delivery = LotExtended, body, program, reads(Lot),
)]
pub struct ExtendLot {
    #[worth_query(scope = LotId)]
    pub lot: u64,
    pub expires_on: CanonicalDate,
}

pub fn extend_lot_body() -> ApplicationOperationBody<DatedSchema, ExtendLotOp, ExtendLot> {
    ExtendLotOp::reference()
        .typed_body()
        .assign_input(LotFacts::expires_on(), ExtendLot::expires_on_input())
        .finish()
        .expect("extend lot body admits")
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, WorthQueryAction)]
#[worth_query(
    schema = DatedSchema, context = Desktop, namespace = RESTORE,
    input = RestoreLotExpiryInput, operation = RestoreLotExpiryOp,
    binding = RestoreLotExpiryBinding, delivery = LotExpiryRestored, body, program, reads(Lot),
)]
pub struct RestoreLotExpiry {
    #[worth_query(scope = LotId)]
    pub lot: u64,
    pub expected: CanonicalDate,
    pub restored: CanonicalDate,
}

pub fn restore_lot_expiry_definition(
) -> ApplicationOperationDefinition<DatedSchema, RestoreLotExpiryOp, RestoreLotExpiry> {
    RestoreLotExpiryOp::reference()
        .typed_body()
        .expect_input(LotFacts::expires_on(), RestoreLotExpiry::expected_input())
        .assign_input(LotFacts::expires_on(), RestoreLotExpiry::restored_input())
        .finish_pure_definition()
        .expect("restore lot expiry body admits")
}

#[derive(WorthQueryCorrection)]
#[worth_query(
    schema = DatedSchema,
    namespace = CORRECT,
    mode = authored_body,
    source = ExtendLotBinding,
    inverse = RestoreLotExpiry,
    disclosure = ReadLotBinding,
    source_body = extend_lot_body,
    preimage_fields(LotFacts::expires_on()),
    preimage_bytes = 64,
    derive = derive_restore_lot_expiry,
    reapply,
    program = DatedProgram,
    inverse_action = RestoreLotExpiryAction,
    source_action = ExtendLotAction,
)]
pub struct CorrectLotExpiry;

fn derive_restore_lot_expiry(
    original: &ExtendLot,
    preimage: &dyn ApplicationCorrectionPreImage<DatedSchema>,
) -> Result<RestoreLotExpiry, ApplicationCorrectionDerivationDenial> {
    Ok(RestoreLotExpiry {
        lot: original.lot,
        expected: original.expires_on,
        restored: decode_preimage_field(preimage, LotFacts::expires_on())?,
    })
}
