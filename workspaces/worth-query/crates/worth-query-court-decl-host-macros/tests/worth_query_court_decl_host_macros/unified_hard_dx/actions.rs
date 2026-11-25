//! Derived actions and `mode = authored_body` correction for the Daymark court.
use super::program::{DaymarkProgram, RestoreTaskStateAction, SetTaskStateAction};
use super::queries::ReadTaskDetailsBinding;
use super::schema::*;
use worth_foundational::facade::WorthValue;
use worth_query_decl::facade::application_aftermath::*;
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::{
    worth_query_identity_namespace, WorthQueryAction, WorthQueryCorrection,
};

pub const SET_STATE_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("daymark.unified.set-state.v1");
pub const RESCHEDULE_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("daymark.unified.reschedule.v1");
pub const RENAME_PROJECT_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("daymark.unified.rename-project.v1");
pub const RESTORE_STATE_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("daymark.unified.restore-state.v1");
pub const CORRECT_STATE_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("daymark.unified.correct-state.v1");
pub const PROBE_KEY_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("daymark.unified.probe-occupied-key.v1");
pub const EXHAUST_KEY_NS: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("daymark.unified.exhaust-auto-key.v1");

thread_local! {
    static THREAD_CAPTURED_SEED: std::cell::Cell<Option<[u8; 32]>> =
        const { std::cell::Cell::new(None) };
}

/// Takes the probe-0 seed captured on the current test thread only, keeping
/// seed observation isolated from parallel test threads.
pub fn take_captured_auto_key_seed() -> Option<[u8; 32]> {
    THREAD_CAPTURED_SEED.with(|slot| slot.take())
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct SequencedAutoKey(pub u64);

impl worth_query_host::facade::WorthQueryAutoIdempotencyKey for SequencedAutoKey {
    fn derive_auto_key(seed: [u8; 32], probe: u64) -> Self {
        if probe == 0 {
            THREAD_CAPTURED_SEED.with(|slot| slot.set(Some(seed)));
        }
        Self(probe + 1)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct ConstantAutoKey(pub u64);

impl worth_query_host::facade::WorthQueryAutoIdempotencyKey for ConstantAutoKey {
    fn derive_auto_key(_seed: [u8; 32], _probe: u64) -> Self {
        Self(1)
    }
}

pub struct ConstantKeyContext;
impl ApplicationBindingContext<DaymarkSchema> for ConstantKeyContext {
    type PrincipalBinding = DaymarkPrincipal;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type IdempotencyKey = ConstantAutoKey;
    type DenialBinding = BodyDenialBinding;
    const IDEMPOTENCY_IDENTITY: &'static str =
        worth_query_decl::facade::worth_query_identity!(EXHAUST_KEY_NS, idempotency);
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        DaymarkSchema,
        DaymarkPrincipal,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DaymarkPrincipal::reference()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, WorthValue, WorthQueryAction)]
#[worth_query(
    schema = DaymarkSchema,
    context = ConstantKeyContext,
    namespace = EXHAUST_KEY_NS,
    input = ExhaustedKeyTaskStateInput,
    operation = ExhaustedKeyTaskStateOperation,
    binding = ExhaustedKeyTaskStateBinding,
    delivery = ExhaustedKeyTaskStateDelivery,
    body,
    program,
    reads(Task),
)]
pub struct ExhaustedKeyTaskState {
    #[worth_query(scope = TaskKey)]
    pub task: u64,
    pub state: String,
}

pub fn exhausted_key_task_state_definition() -> ApplicationOperationDefinition<
    DaymarkSchema,
    ExhaustedKeyTaskStateOperation,
    ExhaustedKeyTaskState,
> {
    ExhaustedKeyTaskStateOperation::reference()
        .typed_body()
        .assign_input(TaskFacts::state(), ExhaustedKeyTaskState::state_input())
        .finish_pure_definition()
        .expect("exhausted_key_task_state body admits")
}

pub struct SequencedKeyContext;
impl ApplicationBindingContext<DaymarkSchema> for SequencedKeyContext {
    type PrincipalBinding = DaymarkPrincipal;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type IdempotencyKey = SequencedAutoKey;
    type DenialBinding = BodyDenialBinding;
    const IDEMPOTENCY_IDENTITY: &'static str =
        worth_query_decl::facade::worth_query_identity!(PROBE_KEY_NS, idempotency);
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        DaymarkSchema,
        DaymarkPrincipal,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        DaymarkPrincipal::reference()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, WorthValue, WorthQueryAction)]
#[worth_query(
    schema = DaymarkSchema,
    context = SequencedKeyContext,
    namespace = PROBE_KEY_NS,
    input = ProbeOccupiedTaskStateInput,
    operation = ProbeOccupiedTaskStateOperation,
    binding = ProbeOccupiedTaskStateBinding,
    delivery = ProbeOccupiedTaskStateDelivery,
    body,
    program,
    reads(Task),
)]
pub struct ProbeOccupiedTaskState {
    #[worth_query(scope = TaskKey)]
    pub task: u64,
    pub state: String,
}

pub fn probe_occupied_task_state_definition() -> ApplicationOperationDefinition<
    DaymarkSchema,
    ProbeOccupiedTaskStateOperation,
    ProbeOccupiedTaskState,
> {
    ProbeOccupiedTaskStateOperation::reference()
        .typed_body()
        .assign_input(TaskFacts::state(), ProbeOccupiedTaskState::state_input())
        .finish_pure_definition()
        .expect("probe_occupied_task_state body admits")
}

#[derive(Clone, Debug, PartialEq, Eq, WorthValue, serde::Deserialize, WorthQueryAction)]
#[worth_query(
    schema = DaymarkSchema,
    context = DaymarkContext,
    namespace = SET_STATE_NS,
    input = SetTaskStateInput,
    operation = SetTaskStateOperation,
    binding = SetTaskStateBinding,
    delivery = SetTaskStateDelivery,
    body,
    program,
    reads(Task),
)]
pub struct SetTaskState {
    #[worth_query(scope = TaskKey)]
    pub task: u64,
    pub state: String,
}

pub fn set_task_state_body(
) -> ApplicationOperationBody<DaymarkSchema, SetTaskStateOperation, SetTaskState> {
    SetTaskStateOperation::reference()
        .typed_body()
        .assign_input(TaskFacts::state(), SetTaskState::state_input())
        .finish()
        .expect("set_task_state body admits")
}

#[derive(Clone, Debug, serde::Serialize, WorthQueryAction)]
#[worth_query(
    schema = DaymarkSchema,
    context = DaymarkContext,
    namespace = RESCHEDULE_NS,
    input = RescheduleTaskInput,
    operation = RescheduleTaskOperation,
    binding = RescheduleTaskBinding,
    delivery = RescheduleTaskDelivery,
    body,
    program,
    reads(Task),
)]
pub struct RescheduleTask {
    #[worth_query(scope = TaskKey)]
    pub task: u64,
    pub due_date: Option<CanonicalDate>,
}

pub fn reschedule_task_definition(
) -> ApplicationOperationDefinition<DaymarkSchema, RescheduleTaskOperation, RescheduleTask> {
    RescheduleTaskOperation::reference()
        .typed_body()
        .assign_optional_input(
            TaskFactsValue::select_due_date(),
            RescheduleTask::due_date_input(),
        )
        .finish_pure_definition()
        .expect("reschedule_task body admits")
}

#[derive(Clone, Debug, WorthValue, WorthQueryAction)]
#[worth_query(
    schema = DaymarkSchema,
    context = DaymarkContext,
    namespace = RENAME_PROJECT_NS,
    input = RenameProjectInput,
    operation = RenameProjectOperation,
    binding = RenameProjectBinding,
    delivery = RenameProjectDelivery,
    body,
    program,
    reads(Project),
    validate = label_on_one_line,
)]
pub struct RenameProject {
    #[worth_query(scope = ProjectKey)]
    pub project: u64,
    pub label: String,
}

/// The action's own end-user validation reason.
pub const LABEL_REASON: &str = "Enter a project label on one line.";

fn label_on_one_line(input: &RenameProject) -> Result<(), &'static str> {
    let label = input.label.trim();
    (!label.is_empty() && !label.chars().any(char::is_control))
        .then_some(())
        .ok_or(LABEL_REASON)
}

pub fn rename_project_definition(
) -> ApplicationOperationDefinition<DaymarkSchema, RenameProjectOperation, RenameProject> {
    RenameProjectOperation::reference()
        .typed_body()
        .assign_input(ProjectFacts::label(), RenameProject::label_input())
        .finish_pure_definition()
        .expect("rename_project body admits")
}

#[derive(Clone, Debug, PartialEq, Eq, WorthValue, serde::Deserialize, WorthQueryAction)]
#[worth_query(
    schema = DaymarkSchema,
    context = DaymarkContext,
    namespace = RESTORE_STATE_NS,
    input = RestoreTaskStateInput,
    operation = RestoreTaskStateOperation,
    binding = RestoreTaskStateBinding,
    delivery = RestoreTaskStateDelivery,
    body,
    program,
    reads(Task),
)]
pub struct RestoreTaskState {
    #[worth_query(scope = TaskKey)]
    pub task: u64,
    pub expected_state: String,
    pub restored_state: String,
}

pub fn restore_task_state_definition(
) -> ApplicationOperationDefinition<DaymarkSchema, RestoreTaskStateOperation, RestoreTaskState> {
    RestoreTaskStateOperation::reference()
        .typed_body()
        .expect_input(TaskFacts::state(), RestoreTaskState::expected_state_input())
        .assign_input(TaskFacts::state(), RestoreTaskState::restored_state_input())
        .finish_pure_definition()
        .expect("restore_task_state body admits")
}

/// Target DX #4: `#[derive(WorthQueryCorrection)]` with `mode = authored_body`,
/// wiring both `home.undo_authored::<CorrectTaskState>` and
/// `home.redo_authored::<CorrectTaskState>`!
#[derive(WorthQueryCorrection)]
#[worth_query(
    schema = DaymarkSchema,
    namespace = CORRECT_STATE_NS,
    mode = authored_body,
    source = SetTaskStateBinding,
    inverse = RestoreTaskState,
    disclosure = ReadTaskDetailsBinding,
    source_body = set_task_state_body,
    preimage_fields(TaskFacts::state()),
    preimage_bytes = 64,
    derive = derive_restore_task_state,
    reapply,
    program = DaymarkProgram,
    inverse_action = RestoreTaskStateAction,
    source_action = SetTaskStateAction,
)]
pub struct CorrectTaskState;

fn derive_restore_task_state(
    original: &SetTaskState,
    preimage: &dyn ApplicationCorrectionPreImage<DaymarkSchema>,
) -> Result<RestoreTaskState, ApplicationCorrectionDerivationDenial> {
    Ok(RestoreTaskState {
        task: original.task,
        expected_state: original.state.clone(),
        restored_state: decode_preimage_field(preimage, TaskFacts::state())?,
    })
}
