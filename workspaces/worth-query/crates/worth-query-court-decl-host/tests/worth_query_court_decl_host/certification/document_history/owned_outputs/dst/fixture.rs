//! The DST's bindings on the owned_outputs world: the model's two entities
//! are members `e1` and `e2` of model `a` (`e2` owned under `e1` by the
//! model's dependency rule, enforced in the handlers), each with one binding
//! per stage kind and the two corrections the model names.
//!
//! A `Set` stage on an absent entity creates it; on a live one it writes the
//! value. A `Retire` stage retires the entity and its derived outputs.
use super::super::*;
use super::reference::{push_order, Label, Stage, Stages, E1, E2};
pub use super::reference::{ENTITIES, NAMES};
use worth_query_decl::facade::application_aftermath::*;
use worth_query_host::facade::application_entry::WorthQueryApplicationHistoryDenial;
use worth_query_decl::facade::application_operation::{
    ApplicationEncodedInput, ApplicationRetainedInputDenial,
};
use worth_query_decl::facade::application_schema::{
    CanonicalDigestId, CanonicalValueSeed, ValueDecodeBounds,
};

/// `$body` with `$intent` bound to `$call`'s binding intent and, when
/// named, `$action` to its program action type.
macro_rules! dispatch {
    (@ $module:ident, $value:expr, $intent:ident, $action:ident, $body:expr) => {{
        #[allow(dead_code)]
        type $action = Action<$module::Binding>;
        let $intent = $value;
        $body
    }};
    ($call:expr, $intent:ident => $body:expr) => {
        dispatch!($call, $intent, _Unused => $body)
    };
    ($call:expr, $intent:ident, $action:ident => $body:expr) => {
        match $call {
            Call::Create(E1, value) => dispatch!(@ create_e1, create_e1::Intent { value }, $intent, $action, $body),
            Call::Create(_, value) => dispatch!(@ create_e2, create_e2::Intent { value }, $intent, $action, $body),
            Call::Set(E1, value) => dispatch!(@ set_e1, set_e1::Intent { value }, $intent, $action, $body),
            Call::Set(_, value) => dispatch!(@ set_e2, set_e2::Intent { value }, $intent, $action, $body),
            Call::Retire(E1) => dispatch!(@ retire_e1, retire_e1::Intent, $intent, $action, $body),
            Call::Retire(_) => dispatch!(@ retire_e2, retire_e2::Intent, $intent, $action, $body),
        }
    };
}

#[macro_use]
mod birth;
#[macro_use]
mod correction;
#[macro_use]
mod death;
#[macro_use]
mod set;
pub mod coupled;
pub mod envelope;
pub mod program;
pub mod reads;
pub mod refusals;
pub mod writers;

/// Why a stage's handler refused, in the model's `DepsSatisfied` terms.
#[derive(Clone, Debug)]
pub enum Refusal {
    /// Creating an entity that is live.
    Present,
    /// Setting or retiring an entity that is not live.
    Absent,
    /// Creating `e2` without a live `e1`.
    OwnerAbsent,
    /// Retiring `e1` while `e2` is live.
    OwnedLive,
}
worth_query_structured_value_binding!(pub RefusalBinding for Refusal { identity: "owned-outputs.dst.refusal.v1" });

birth_binding!(create_e1, DstCreateE1, "e1", None, "owned-outputs.dst.create-e1.v1");
birth_binding!(create_e2, DstCreateE2, "e2", Some("e1"), "owned-outputs.dst.create-e2.v1");
birth_binding!(restore_e1, DstRestoreE1, "e1", None, "owned-outputs.dst.restore-e1.v1");
birth_binding!(restore_e2, DstRestoreE2, "e2", Some("e1"), "owned-outputs.dst.restore-e2.v1");
set_binding!(set_e1, DstSetE1, "e1", "owned-outputs.dst.set-e1.v1");
set_binding!(set_e2, DstSetE2, "e2", "owned-outputs.dst.set-e2.v1");
death_binding!(retire_e1, DstRetireE1, "e1", Some("e2"), "owned-outputs.dst.retire-e1.v1");
death_binding!(retire_e2, DstRetireE2, "e2", None, "owned-outputs.dst.retire-e2.v1");
death_binding!(despawn_e1, DstDespawnE1, "e1", Some("e2"), "owned-outputs.dst.despawn-e1.v1");
death_binding!(despawn_e2, DstDespawnE2, "e2", None, "owned-outputs.dst.despawn-e2.v1");
undo_create!(UndoCreateE1, create_e1, despawn_e1, "owned-outputs.dst.undo-create-e1.v1");
undo_create!(UndoCreateE2, create_e2, despawn_e2, "owned-outputs.dst.undo-create-e2.v1");
undo_retire!(UndoRetireE1, retire_e1, restore_e1, "owned-outputs.dst.undo-retire-e1.v1");
undo_retire!(UndoRetireE2, retire_e2, restore_e2, "owned-outputs.dst.undo-retire-e2.v1");

/// Every binding: the correction sources (create, retire) need only their
/// binding; the others declare their operation.
pub fn declare(schema: ApplicationSchemaDeclarationBuilder<Schema>) -> ApplicationSchemaDeclarationBuilder<Schema> {
    macro_rules! declared {
        ($schema:expr, $($module:ident),*) => {
            $schema$(
                .declared_operation::<$module::Operation>(
                    $module::Operation::reference().definition().no_external_effect().no_aftermath().finish(),
                )
                .application_mutation_binding::<$module::Binding>()
            )*
        };
    }
    let schema = declared!(schema, set_e1, set_e2, despawn_e1, despawn_e2, restore_e1, restore_e2);
    let schema = family::dst::dst_use::declare(family::dst::dst_b::declare(family::dst::dst_a::declare(schema)));
    schema
        .application_mutation_binding::<create_e1::Binding>()
        .application_mutation_binding::<create_e2::Binding>()
        .application_mutation_binding::<retire_e1::Binding>()
        .application_mutation_binding::<retire_e2::Binding>()
}

/// History labels: one per stage kind per entity, parsed back by [`label`].
pub fn history(
    definition: ApplicationHistoryDefinition<Schema, History>,
) -> ApplicationHistoryDefinition<Schema, History> {
    definition
        .operation::<create_e1::Binding>("dst set e1")
        .operation::<create_e2::Binding>("dst set e2")
        .operation::<set_e1::Binding>("dst set e1")
        .operation::<set_e2::Binding>("dst set e2")
        .operation::<retire_e1::Binding>("dst retire e1")
        .operation::<retire_e2::Binding>("dst retire e2")
        .operation::<despawn_e1::Binding>("dst undo-create e1")
        .operation::<despawn_e2::Binding>("dst undo-create e2")
        .operation::<restore_e1::Binding>("dst undo-retire e1")
        .operation::<restore_e2::Binding>("dst undo-retire e2")
}

/// A step's label in the model's terms; `None` for one the DST never writes.
pub fn label(text: &str) -> Option<Label> {
    let (kind, entity) = text.strip_prefix("dst ")?.rsplit_once(' ')?;
    let entity = match entity {
        "e1" => E1,
        "e2" => E2,
        _ => return None,
    };
    Some(match kind {
        "set" => Label::Set(entity),
        "retire" => Label::Retire(entity),
        "undo-create" => Label::UndoCreate(entity),
        "undo-retire" => Label::UndoRetire(entity),
        _ => return None,
    })
}

pub fn contracts(
    contracts: &mut WorthQueryApplicationContributionContracts<Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    family::dst::dst_a::contracts(contracts)?;
    family::dst::dst_b::contracts(contracts)?;
    family::dst::dst_use::contracts(contracts)
}

pub fn install(
    setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    macro_rules! handlers {
        ($($module:ident),*) => {
            $(setup.handler::<$module::Binding, _>($module::Handler)?;)*
        };
    }
    handlers!(create_e1, create_e2, restore_e1, restore_e2, set_e1, set_e2);
    handlers!(retire_e1, retire_e2, despawn_e1, despawn_e2);
    family::dst::dst_a::install(setup)?;
    family::dst::dst_b::install(setup)?;
    family::dst::dst_use::install(setup)
}

/// The program actions the writers (`.edit`, `.gesture`) need.
pub fn actions(
    entry: ApplicationFeatureSpecBuilder<Schema, ApplicationRootComposition, super::super::program::Entry>,
) -> ApplicationFeatureSpecBuilder<Schema, ApplicationRootComposition, super::super::program::Entry> {
    macro_rules! actions {
        ($entry:expr, $($module:ident),*) => {
            $entry$(.action::<$module::Binding, Completion>().finish_action())*
        };
    }
    actions!(entry, create_e1, create_e2, restore_e1, restore_e2, set_e1, set_e2, retire_e1, retire_e2, despawn_e1, despawn_e2)
}

/// The program action of one binding.
pub type Action<B> =
    ApplicationActionReference<Schema, ApplicationRootComposition, super::super::program::Entry, B, Completion>;

/// One stage resolved to the binding it runs, given what is live before it.
#[derive(Clone, Copy, Debug)]
pub enum Call {
    Create(usize, u64),
    Set(usize, u64),
    Retire(usize),
}

impl Call {
    pub fn entity(self) -> usize {
        match self {
            Self::Create(entity, _) | Self::Set(entity, _) | Self::Retire(entity) => entity,
        }
    }
}

/// `stages` in push order, each resolved against `live` as the stages
/// before it leave it.
pub fn calls(stages: &Stages, mut live: [bool; 2]) -> Vec<Call> {
    let mut calls = Vec::new();
    for entity in push_order(stages) {
        match stages[entity] {
            Stage::Skip => {}
            Stage::Retire => {
                calls.push(Call::Retire(entity));
                live[entity] = false;
            }
            Stage::Set(value) if live[entity] => calls.push(Call::Set(entity, value)),
            Stage::Set(value) => {
                calls.push(Call::Create(entity, value));
                live[entity] = true;
            }
        }
    }
    calls
}
