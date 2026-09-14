use std::borrow::Cow;
use std::marker::PhantomData;

use super::WorthQueryApplicationOutputPosture;

pub(in crate::domain_computation::primary_graph) mod action {
    pub trait Sealed {}
}

pub struct Preserve;
pub struct Create;
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

/// A binding-owned semantic result role. The name describes correspondence;
/// it never supplies or reconstructs an entity identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationOutputRole<Binding, Entity, Action> {
    name: Cow<'static, str>,
    _marker: PhantomData<fn() -> (Binding, Entity, Action)>,
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
