use std::borrow::Cow;
use std::marker::PhantomData;

use worth_query_declaration::facade::application_operation::ApplicationMutationOutputRoleCardinality;

use super::WorthQueryApplicationOutputPosture;

pub(in crate::domain_computation::primary_graph) mod action {
    pub trait Sealed {}
}

pub(in crate::domain_computation::primary_graph) mod fixed {
    pub trait Sealed {}
}

/// Output posture marker: the role names a record the mutation keeps, neither
/// creating nor retiring it. Use it as the action of a
/// [`WorthQueryApplicationOutputRole`].
pub struct Preserve;
/// Output posture marker: the role names a record the mutation creates. Use it as
/// the action of a [`WorthQueryApplicationOutputRole`].
pub struct Create;
/// Output posture marker: the role names a record the mutation retires. Use it as
/// the action of a [`WorthQueryApplicationOutputRole`].
pub struct Retire;

/// Marker implemented by Query's sealed output postures.
pub trait WorthQueryApplicationOutputAction: action::Sealed {
    const POSTURE: WorthQueryApplicationOutputPosture;
}

impl action::Sealed for Preserve {}
impl WorthQueryApplicationOutputAction for Preserve {
    const POSTURE: WorthQueryApplicationOutputPosture =
        WorthQueryApplicationOutputPosture::Preserve;
}

impl action::Sealed for Create {}
impl WorthQueryApplicationOutputAction for Create {
    const POSTURE: WorthQueryApplicationOutputPosture = WorthQueryApplicationOutputPosture::Create;
}

impl action::Sealed for Retire {}
impl WorthQueryApplicationOutputAction for Retire {
    const POSTURE: WorthQueryApplicationOutputPosture = WorthQueryApplicationOutputPosture::Retire;
}

const MAXIMUM_OUTPUT_ROLE_NAME_BYTES: usize = 256;

/// Why a runtime semantic output-role name cannot enter an application candidate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationOutputRoleNameDenial {
    Empty,
    SurroundingWhitespace,
    ControlCharacter,
    RepresentationTooLarge {
        maximum_bytes: usize,
        required_bytes: usize,
    },
}

impl std::fmt::Display for WorthQueryApplicationOutputRoleNameDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => formatter.write_str("an application output role name cannot be empty"),
            Self::SurroundingWhitespace => formatter
                .write_str("an application output role name cannot contain surrounding whitespace"),
            Self::ControlCharacter => formatter
                .write_str("an application output role name cannot contain control characters"),
            Self::RepresentationTooLarge {
                maximum_bytes,
                required_bytes,
            } => write!(
                formatter,
                "application output role name requires {required_bytes} bytes but the maximum is {maximum_bytes}"
            ),
        }
    }
}

impl std::error::Error for WorthQueryApplicationOutputRoleNameDenial {}

/// A binding-owned semantic result role that every commit binds exactly once:
/// a fixed role declared with `for_entity`, or one member of a declared role
/// family. The name describes correspondence; it never supplies or
/// reconstructs an entity identity. Reading through it yields the entity itself.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationOutputRole<Binding, Entity, Action> {
    name: Cow<'static, str>,
    _marker: PhantomData<fn() -> (Binding, Entity, Action)>,
}

/// A binding-owned fixed result role declared with `optional_for_entity`: a
/// commit binds it at most once. Reading through it yields `Option`, so an
/// absent role is a value, never a denial.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationOptionalOutputRole<Binding, Entity, Action> {
    name: Cow<'static, str>,
    _marker: PhantomData<fn() -> (Binding, Entity, Action)>,
}

/// A typed output-role token: [`WorthQueryApplicationOutputRole`] for a role
/// every commit binds, [`WorthQueryApplicationOptionalOutputRole`] for a role
/// a commit may leave unbound.
///
/// The token's cardinality decides what a read returns, so absence is typed:
/// `Read<T>` is `T` for an exactly-one token and `Option<T>` for an at-most-one
/// token. Query refuses a token whose cardinality differs from the one the
/// binding declares for that role.
pub trait WorthQueryApplicationFixedOutputRole: fixed::Sealed {
    type Binding: 'static;
    type Entity: 'static;
    type Action: WorthQueryApplicationOutputAction;
    /// What reading this role yields when the value read is `T`.
    type Read<T>;
    const CARDINALITY: ApplicationMutationOutputRoleCardinality;

    fn name(&self) -> &str;

    /// Shape one role lookup. An exactly-one token yields `None` when the role
    /// is unbound, which callers refuse; an at-most-one token always yields a
    /// read, carrying absence as its `None`.
    fn read<T>(bound: Option<T>) -> Option<Self::Read<T>>;
}

impl<Binding, Entity, Action> fixed::Sealed
    for WorthQueryApplicationOutputRole<Binding, Entity, Action>
{
}

impl<Binding, Entity, Action> WorthQueryApplicationFixedOutputRole
    for WorthQueryApplicationOutputRole<Binding, Entity, Action>
where
    Binding: 'static,
    Entity: 'static,
    Action: WorthQueryApplicationOutputAction,
{
    type Binding = Binding;
    type Entity = Entity;
    type Action = Action;
    type Read<T> = T;
    const CARDINALITY: ApplicationMutationOutputRoleCardinality =
        ApplicationMutationOutputRoleCardinality::ExactlyOne;

    fn name(&self) -> &str {
        &self.name
    }

    fn read<T>(bound: Option<T>) -> Option<T> {
        bound
    }
}

impl<Binding, Entity, Action> fixed::Sealed
    for WorthQueryApplicationOptionalOutputRole<Binding, Entity, Action>
{
}

impl<Binding, Entity, Action> WorthQueryApplicationFixedOutputRole
    for WorthQueryApplicationOptionalOutputRole<Binding, Entity, Action>
where
    Binding: 'static,
    Entity: 'static,
    Action: WorthQueryApplicationOutputAction,
{
    type Binding = Binding;
    type Entity = Entity;
    type Action = Action;
    type Read<T> = Option<T>;
    const CARDINALITY: ApplicationMutationOutputRoleCardinality =
        ApplicationMutationOutputRoleCardinality::AtMostOne;

    fn name(&self) -> &str {
        &self.name
    }

    fn read<T>(bound: Option<T>) -> Option<Option<T>> {
        Some(bound)
    }
}

/// One declared generated-role family from a prior mutation binding.
///
/// The prefix selects installed correspondence meaning. Query validates it
/// against `Binding::Output` before exposing any prior identities.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationOutputRoleFamily<Binding, Entity> {
    prefix: &'static str,
    _marker: PhantomData<fn() -> (Binding, Entity)>,
}

impl<Binding, Entity> WorthQueryApplicationOutputRoleFamily<Binding, Entity> {
    pub const fn from_static(prefix: &'static str) -> Self {
        Self {
            prefix,
            _marker: PhantomData,
        }
    }

    pub const fn prefix(&self) -> &'static str {
        self.prefix
    }
}

impl<Binding, Entity, Action> WorthQueryApplicationOutputRole<Binding, Entity, Action> {
    /// Construct a statically named exact role. Query validates it against the
    /// installed binding before candidate effects can be admitted.
    pub const fn from_static(name: &'static str) -> Self {
        Self {
            name: Cow::Borrowed(name),
            _marker: PhantomData,
        }
    }

    /// Construct a topology-derived role without leaking an untyped string into
    /// effect authoring.
    pub fn try_new(
        name: impl Into<String>,
    ) -> Result<Self, WorthQueryApplicationOutputRoleNameDenial> {
        let name = name.into();
        validate_output_role_name(&name)?;
        Ok(Self {
            name: Cow::Owned(name),
            _marker: PhantomData,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl<Binding, Entity, Action> WorthQueryApplicationOptionalOutputRole<Binding, Entity, Action> {
    /// Construct a statically named at-most-one role. Query validates it
    /// against the installed binding before candidate effects can be admitted.
    pub const fn from_static(name: &'static str) -> Self {
        Self {
            name: Cow::Borrowed(name),
            _marker: PhantomData,
        }
    }

    /// Construct a runtime-named at-most-one role without leaking an untyped
    /// string into effect authoring.
    pub fn try_new(
        name: impl Into<String>,
    ) -> Result<Self, WorthQueryApplicationOutputRoleNameDenial> {
        let name = name.into();
        validate_output_role_name(&name)?;
        Ok(Self {
            name: Cow::Owned(name),
            _marker: PhantomData,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

pub(super) fn validate_output_role_name(
    name: &str,
) -> Result<(), WorthQueryApplicationOutputRoleNameDenial> {
    if name.is_empty() {
        return Err(WorthQueryApplicationOutputRoleNameDenial::Empty);
    }
    if name.trim() != name {
        return Err(WorthQueryApplicationOutputRoleNameDenial::SurroundingWhitespace);
    }
    if name.chars().any(char::is_control) {
        return Err(WorthQueryApplicationOutputRoleNameDenial::ControlCharacter);
    }
    if name.len() > MAXIMUM_OUTPUT_ROLE_NAME_BYTES {
        return Err(
            WorthQueryApplicationOutputRoleNameDenial::RepresentationTooLarge {
                maximum_bytes: MAXIMUM_OUTPUT_ROLE_NAME_BYTES,
                required_bytes: name.len(),
            },
        );
    }
    Ok(())
}
