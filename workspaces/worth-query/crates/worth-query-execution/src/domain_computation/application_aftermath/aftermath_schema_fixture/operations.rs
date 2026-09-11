//! Aftermath fixture operation bindings through the production declaration door.

use worth_query_declaration::facade::application_aftermath::{
    DeclaredAftermathPostcondition, DeclaredApplicationAftermathContract, DeclaredCompensation,
    DeclaredCorrectionMechanism, DeclaredLoweringCorrespondenceRef, DeclaredPreImageDemand,
    DeclaredPreImageLocus, DeclaredReconciliationProcedure, DeclaredRecordedInverse,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationAbilityRef, ApplicationOperationMarkerIdentity, ApplicationOperationRef,
    ApplicationSchemaDeclaration, ApplicationSchemaDeclarationBuilder,
    ApplicationStructuredValueBinding, WorthQueryExternalEffectCorrelationFamily,
};

use super::declaration::{
    principal_read, AftermathFixtureSchema, ApproveEmergencyAccess, AuditRetention, BalanceRead,
    Charge, DeathNoticeEffect, FixtureAbility, FixtureEntity, FixtureInput, FreezeAccount,
    FreezeAccountFields, FreezeBalance, FreezeNote, FrozenRead, LegalHold, NoteRead, NotifyDeath,
    ReleaseEstate, Transfer, TransferLarge, TransferSmall, WireInstructionEffect,
    WireTransferFinal,
};

/// The rail the fixture's death notices escape through.
pub(super) const DEATH_NOTICE_RAIL: &str = "estate-death-notice-rail";
/// The rail the fixture's wire instructions escape through.
pub(super) const WIRE_RAIL: &str = "escaped-rail";

type FixtureBuilder = ApplicationSchemaDeclarationBuilder<AftermathFixtureSchema>;
type FixtureAbilityRef =
    ApplicationAbilityRef<AftermathFixtureSchema, FixtureAbility, FixtureEntity>;

/// The non-principal reads the inverse-bearing operations demand.
pub(super) struct FixtureReads {
    pub(super) frozen: FrozenRead,
    pub(super) note: NoteRead,
    pub(super) balance: BalanceRead,
}

fn op<Operation>() -> ApplicationOperationRef<AftermathFixtureSchema, Operation, FixtureInput>
where
    Operation: ApplicationOperationMarkerIdentity<AftermathFixtureSchema>,
    Operation::InputBinding: ApplicationStructuredValueBinding<Value = FixtureInput>,
{
    ApplicationOperationRef::from_declaration()
}

fn not_correctable() -> DeclaredApplicationAftermathContract<AftermathFixtureSchema> {
    DeclaredApplicationAftermathContract::not_correctable()
}

fn compensation(
    slot: &str,
    identity: &str,
) -> DeclaredApplicationAftermathContract<AftermathFixtureSchema> {
    DeclaredApplicationAftermathContract::runtime_alone(DeclaredCorrectionMechanism::Compensation(
        DeclaredCompensation::new(
            slot,
            DeclaredAftermathPostcondition::BusinessPostcondition {
                identity: identity.into(),
            },
        )
        .unwrap(),
    ))
}

fn inverse(
    loci: impl IntoIterator<Item = DeclaredPreImageLocus<AftermathFixtureSchema>>,
    bound: usize,
    lowering: &str,
) -> DeclaredApplicationAftermathContract<AftermathFixtureSchema> {
    DeclaredApplicationAftermathContract::runtime_alone(
        DeclaredCorrectionMechanism::RecordedInverse(
            DeclaredRecordedInverse::new(
                "unfreeze",
                DeclaredLoweringCorrespondenceRef::new(lowering).unwrap(),
                DeclaredAftermathPostcondition::ExactPriorTruth,
                DeclaredPreImageDemand::new(loci, bound).unwrap(),
            )
            .unwrap(),
        ),
    )
}

macro_rules! bind_principal_op {
    ($schema:expr, $ability:expr, $op:ty, $name:expr, $aftermath:expr) => {{
        let operation = op::<$op>();
        let read = principal_read();
        $schema
            .operation(
                operation
                    .definition()
                    .no_external_effect()
                    .aftermath($aftermath)
                    .finish(),
            )
            .operation_decision_fact_budget(operation, 1)
            .operation_projection_work_budget(operation, 16)
            .operation_requires_ability(operation, $ability)
            .operation_read_field(operation, read)
    }};
}

macro_rules! bind_escaping_principal_op {
    ($schema:expr, $ability:expr, $op:ty, $name:expr, $effect:expr, $rail:expr, $aftermath:expr) => {{
        let operation = op::<$op>();
        let read = principal_read();
        $schema
            .operation(
                operation
                    .definition()
                    .external_effect(
                        $effect,
                        WorthQueryExternalEffectCorrelationFamily::new($rail).unwrap(),
                    )
                    .aftermath($aftermath)
                    .finish(),
            )
            .operation_decision_fact_budget(operation, 1)
            .operation_projection_work_budget(operation, 16)
            .operation_requires_ability(operation, $ability)
            .operation_read_field(operation, read)
            .operation_emit(operation, $effect)
    }};
}

pub(super) fn bind_operations(
    schema: FixtureBuilder,
    ability: FixtureAbilityRef,
    reads: FixtureReads,
) -> Result<
    ApplicationSchemaDeclaration<AftermathFixtureSchema>,
    worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
> {
    let schema = bind_compensating_ops(schema, ability);
    let schema = bind_uncorrectable_ops(schema, ability);
    let schema = bind_escaping_ops(schema, ability);
    let schema = bind_inverse_ops(schema, ability, reads);
    schema.build()
}

/// Contained operations whose correction is a compensating action.
fn bind_compensating_ops(schema: FixtureBuilder, ability: FixtureAbilityRef) -> FixtureBuilder {
    let schema = bind_principal_op!(
        schema,
        ability,
        Transfer,
        "transfer",
        compensation("compensate-transfer", "balances-restored")
    );
    let schema = bind_principal_op!(
        schema,
        ability,
        TransferSmall,
        "transfer-small",
        compensation("compensate-transfer", "balances-restored")
    );
    let schema = bind_principal_op!(
        schema,
        ability,
        TransferLarge,
        "transfer-large",
        compensation("compensate-transfer-large", "balances-restored-large")
    );
    bind_principal_op!(
        schema,
        ability,
        Charge,
        "charge",
        DeclaredApplicationAftermathContract::runtime_alone(
            DeclaredCorrectionMechanism::Compensation(
                DeclaredCompensation::new(
                    "undo-charge",
                    DeclaredAftermathPostcondition::InvariantRestored {
                        invariant: "balanced-ledger".into(),
                    },
                )
                .unwrap(),
            ),
        )
    )
}

/// Contained operations the runtime publishes as irreversible.
fn bind_uncorrectable_ops(schema: FixtureBuilder, ability: FixtureAbilityRef) -> FixtureBuilder {
    let schema = bind_principal_op!(
        schema,
        ability,
        ReleaseEstate,
        "ReleaseEstate",
        not_correctable()
    );
    let schema = bind_principal_op!(schema, ability, LegalHold, "legal-hold", not_correctable());
    let schema = bind_principal_op!(
        schema,
        ability,
        AuditRetention,
        "audit-retention",
        not_correctable()
    );
    bind_principal_op!(
        schema,
        ability,
        ApproveEmergencyAccess,
        "ApproveEmergencyAccess",
        not_correctable()
    )
}

/// The three operations carrying a recorded inverse over a non-principal read.
fn bind_inverse_ops(
    schema: FixtureBuilder,
    ability: FixtureAbilityRef,
    reads: FixtureReads,
) -> FixtureBuilder {
    let freeze = op::<FreezeAccount>();
    let schema = schema
        .operation(
            freeze
                .definition()
                .no_external_effect()
                .aftermath(inverse(
                    [DeclaredPreImageLocus::from_field(reads.frozen)],
                    64,
                    "account-freeze-inverse",
                ))
                .finish(),
        )
        .operation_decision_fact_budget(freeze, 1)
        .operation_projection_work_budget(freeze, 16)
        .operation_requires_ability(freeze, ability)
        .operation_read_field(freeze, reads.frozen);
    let note_op = op::<FreezeNote>();
    let schema = schema
        .operation(
            note_op
                .definition()
                .no_external_effect()
                .aftermath(inverse(
                    [DeclaredPreImageLocus::from_field(reads.note)],
                    4,
                    "geometry-inverse",
                ))
                .finish(),
        )
        .operation_decision_fact_budget(note_op, 1)
        .operation_projection_work_budget(note_op, 16)
        .operation_requires_ability(note_op, ability)
        .operation_read_field(note_op, reads.note);
    let balance_op = op::<FreezeBalance>();
    let schema = schema
        .operation(
            balance_op
                .definition()
                .no_external_effect()
                .aftermath(inverse(
                    [DeclaredPreImageLocus::from_field(reads.balance)],
                    256,
                    "inverse-v2",
                ))
                .finish(),
        )
        .operation_decision_fact_budget(balance_op, 1)
        .operation_projection_work_budget(balance_op, 16)
        .operation_requires_ability(balance_op, ability)
        .operation_read_field(balance_op, reads.balance);
    let fields_op = op::<FreezeAccountFields>();
    schema
        .operation(
            fields_op
                .definition()
                .no_external_effect()
                .aftermath(inverse(
                    [
                        DeclaredPreImageLocus::from_field(reads.frozen),
                        DeclaredPreImageLocus::from_field(reads.note),
                    ],
                    256,
                    "account-fields-inverse",
                ))
                .finish(),
        )
        .operation_decision_fact_budget(fields_op, 2)
        .operation_projection_work_budget(fields_op, 24)
        .operation_requires_ability(fields_op, ability)
        .operation_read_field(fields_op, reads.frozen)
        .operation_read_field(fields_op, reads.note)
}

/// The two fixture operations that actually escape the runtime.
///
/// Each declares its emitted effect and the external lane it projects onto.
/// That member — not the aftermath contract — is what makes the operation
/// escaping, so the reversibility guard and the installed external posture both
/// read what is declared here (Q8.25-C1). Before slice 9A the fixture carried an
/// aftermath escaping posture with no lane behind it: it asserted a reconcilable
/// external owner for operations that would have co-committed no outbox record
/// and dispatched nothing.
fn bind_escaping_ops(schema: FixtureBuilder, ability: FixtureAbilityRef) -> FixtureBuilder {
    let schema = schema
        .effect(DeathNoticeEffect::reference())
        .effect(WireInstructionEffect::reference());
    let schema = bind_escaping_principal_op!(
        schema,
        ability,
        NotifyDeath,
        "notify-death",
        DeathNoticeEffect::reference(),
        DEATH_NOTICE_RAIL,
        DeclaredApplicationAftermathContract::runtime_with_external_owner(
            DeclaredCorrectionMechanism::Compensation(
                DeclaredCompensation::new(
                    "confirm-death-notice-with-authority",
                    DeclaredAftermathPostcondition::BusinessPostcondition {
                        identity: "death-notice-confirmed".into(),
                    },
                )
                .unwrap(),
            ),
            DeclaredReconciliationProcedure::new("confirm-death-notice-with-authority").unwrap(),
        )
    );
    bind_escaping_principal_op!(
        schema,
        ability,
        WireTransferFinal,
        "wire-transfer-final",
        WireInstructionEffect::reference(),
        WIRE_RAIL,
        not_correctable()
    )
}
