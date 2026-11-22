//! Ordinary schedule commands carry native carriers as plain input members.
pub use super::super::{
    binding::{BodyDenialBinding, BodyResultBinding},
    schema::*,
};
use super::declaration::{AssignForgedSchedule, AssignSchedule};
use serde::{ser::SerializeStruct, Serialize, Serializer};
use std::marker::PhantomData;
use worth_foundational::facade::{CanonicalDate, CanonicalTime, InlineBytes};
pub use worth_query_host::facade::{
    declaration::{application_operation::*, application_schema::*},
    primary_graph::{WorthQueryOperationBodyDecision, WorthQueryOperationBodyDenial},
};

/// Each member is a one-level Option: Some sets its field, None clears it.
#[derive(Clone, Debug, Serialize)]
pub struct ScheduleInput<Day> {
    pub identity: String,
    pub due: Option<Day>,
    pub opens: Option<CanonicalTime>,
    pub token: Option<[u8; 16]>,
    pub payload: Option<InlineBytes>,
}

/// A day count outside `CanonicalDate`, serialized in that carrier's exact
/// shape, as an untrusted client could send it.
#[derive(Clone, Copy, Debug)]
pub struct ForgedDay(pub i64);
impl Serialize for ForgedDay {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut carrier = serializer.serialize_struct("CanonicalDate", 1)?;
        carrier.serialize_field("days_from_unix_epoch", &self.0)?;
        carrier.end()
    }
}

pub trait ScheduleDay: Serialize + Clone + std::fmt::Debug + 'static {
    const INPUT_IDENTITY: &'static str;
}
impl ScheduleDay for CanonicalDate {
    const INPUT_IDENTITY: &'static str = "worth.query.body-court.schedule-input.v1";
}
impl ScheduleDay for ForgedDay {
    const INPUT_IDENTITY: &'static str = "worth.query.body-court.forged-schedule-input.v1";
}

pub struct ScheduleInputBinding<Day>(PhantomData<Day>);
impl<Day: ScheduleDay> ApplicationStructuredValueBinding for ScheduleInputBinding<Day> {
    type Value = ScheduleInput<Day>;
    const IDENTITY_NAME: &'static str = Day::INPUT_IDENTITY;
    const RETAIN_CANONICAL_INPUT: bool = true;
    fn validate(_: &ScheduleInput<Day>) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}

macro_rules! schedule_binding {
    ($Binding:ident, $Intent:ident, $Operation:ident, $Day:ty, $name:literal) => {
        pub struct $Binding<const MODE: u64>;
        impl<const MODE: u64> ApplicationMutationBinding<DocumentRetentionSchema<MODE>>
            for $Binding<MODE>
        {
            fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
                ApplicationWorkflowExecutionRequirements::not_applicable()
            }
            type Input = ScheduleInput<$Day>;
            type InputBinding = ScheduleInputBinding<$Day>;
            type Result = ();
            type ResultBinding = BodyResultBinding;
            type IdempotencyKey = u64;
            type Operation = $Operation;
            type Decision =
                WorthQueryOperationBodyDecision<DocumentRetentionSchema<MODE>, Document>;
            type Denial = WorthQueryOperationBodyDenial;
            type DenialBinding = BodyDenialBinding;
            type Output = NoApplicationMutationOutputs;
            type ScopeBinding = ApplicationMutationFieldScope<DocumentIdentityField>;
            type PrincipalBinding = DocumentPrincipalBinding;
            type Mapping = ExternalMapping;
            type Principal = Principal;
            type PrincipalIdentity = u64;
            type PrincipalIdentityBinding = U64ApplicationValueBinding;
            type SourceExpectation = NoApplicationMutationSource;
            const IDENTITY: &'static str = concat!("worth.query.body-court.", $name, ".v1");
            const HANDLER_IDENTITY: &'static str =
                concat!("worth.query.body-court.", $name, "-body.v1");
            const IDEMPOTENCY_IDENTITY: &'static str =
                concat!("worth.query.body-court.", $name, "-command.v1");
            const REQUIRES_APPLICATION_PROGRAM: bool = true;

            fn scope_field() -> DocumentIdentityField {
                DocumentIdentityField::at("DocumentIdentityField")
            }
            fn principal_binding() -> ApplicationPrincipalBindingRef<
                DocumentRetentionSchema<MODE>,
                DocumentPrincipalBinding,
                ExternalMapping,
                Principal,
                u64,
                U64ApplicationValueBinding,
            > {
                DocumentPrincipalBinding::reference()
            }
        }

        #[derive(Clone)]
        pub struct $Intent(pub ScheduleInput<$Day>);
        impl<const MODE: u64> ApplicationMutationIntent<DocumentRetentionSchema<MODE>> for $Intent {
            type Binding = $Binding<MODE>;
            fn input(&self) -> &ScheduleInput<$Day> {
                &self.0
            }
            fn scope_binding(&self) -> ApplicationMutationFieldScope<DocumentIdentityField> {
                ApplicationMutationFieldScope::new(
                    DocumentIdentityField::at("DocumentIdentityField"),
                    self.0.identity.clone(),
                )
            }
        }
    };
}

schedule_binding!(
    ScheduleBinding,
    ScheduleIntent,
    AssignSchedule,
    CanonicalDate,
    "assign-schedule"
);
schedule_binding!(
    ForgedScheduleBinding,
    ForgedScheduleIntent,
    AssignForgedSchedule,
    ForgedDay,
    "assign-forged-schedule"
);
