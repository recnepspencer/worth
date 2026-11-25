//! Retire a Home, or a refused Directory opening, under an execution policy
//! while keeping the owner across every refusal, so an application retries
//! one retirement instead of matching each close refusal itself.
use worth_execution::LeaseDenial;
use worth_foundational::facade::ExecutionRequestPolicy;
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationSchemaComposition,
};

use super::{
    WorthQueryApplicationHome, WorthQueryApplicationHomeCloseRefused,
    WorthQueryApplicationHomeOpenRefused, WorthQueryApplicationHomeOpeningCloseRefused,
    WorthQueryDirectoryOpenDenial, WorthQueryHomeClosePolicyRefused,
};

/// A Home on its way to retirement: still open, or a close Native must
/// continue. [`Self::try_retire`] returns the same retirement on refusal.
///
/// ```
/// use worth_query_host::facade::{WorthQueryHomeRetirement, WorthQueryHomeRetirementStop};
/// fn retire<S, P>(mut retiring: WorthQueryHomeRetirement<S, P>) -> Result<(), WorthQueryHomeRetirementStop>
/// where
///     S: worth_query_host::facade::declaration::application_schema::ApplicationSchema,
///     P: 'static,
/// {
///     let policy = worth_query_host::facade::DEFAULT_HOST_EXECUTION_POLICY;
///     for _ in 0..3 {
///         match retiring.try_retire(policy) {
///             Ok(()) => return Ok(()),
///             Err((next, WorthQueryHomeRetirementStop::CloseIncomplete)) => retiring = next,
///             Err((_, stop)) => return Err(stop),
///         }
///     }
///     Err(WorthQueryHomeRetirementStop::CloseIncomplete)
/// }
/// ```
#[must_use = "a retirement retains the Home until it closes"]
pub enum WorthQueryHomeRetirement<Schema: ApplicationSchema, Program> {
    Open(Box<WorthQueryApplicationHome<Schema, Program>>),
    Closing(Box<WorthQueryApplicationHomeCloseRefused<Schema, Program>>),
}

/// Why one retirement attempt stopped; the retirement itself is returned.
#[derive(Debug)]
pub enum WorthQueryHomeRetirementStop {
    /// The policy's execution lease was refused before closing began.
    Admission(LeaseDenial),
    /// Native has not yet finished closing; retry the returned retirement.
    CloseIncomplete,
}

impl<Schema: ApplicationSchema, Program: 'static> WorthQueryApplicationHome<Schema, Program> {
    /// Starts retiring this Home; see [`WorthQueryHomeRetirement::try_retire`].
    pub fn into_retirement(self) -> WorthQueryHomeRetirement<Schema, Program> {
        WorthQueryHomeRetirement::Open(Box::new(self))
    }
}

impl<Schema: ApplicationSchema, Program: 'static> WorthQueryHomeRetirement<Schema, Program> {
    /// Closes (or continues closing) the Home under `policy` and reclaims its
    /// retired Query resources. On refusal the retirement keeps the Home.
    pub fn try_retire(
        self,
        policy: ExecutionRequestPolicy,
    ) -> Result<(), (Self, WorthQueryHomeRetirementStop)> {
        use WorthQueryHomeClosePolicyRefused as Refused;
        let refused = match self {
            Self::Open(home) => match home.close_clean_with_policy(policy) {
                Ok(_) => return Ok(()),
                Err(Refused::Admission { owner, cause }) => (Self::Open(owner), cause),
                Err(Refused::Close(closing)) => return Err(Self::continuing(closing)),
            },
            Self::Closing(closing) => match closing.try_close_with_policy(policy) {
                Ok(closed) => {
                    closed.reclaim();
                    return Ok(());
                }
                Err(Refused::Admission { owner, cause }) => (Self::Closing(owner), cause),
                Err(Refused::Close(closing)) => return Err(Self::continuing(closing)),
            },
        };
        Err((
            refused.0,
            WorthQueryHomeRetirementStop::Admission(refused.1),
        ))
    }

    fn continuing(
        closing: WorthQueryApplicationHomeCloseRefused<Schema, Program>,
    ) -> (Self, WorthQueryHomeRetirementStop) {
        (
            Self::Closing(Box::new(closing)),
            WorthQueryHomeRetirementStop::CloseIncomplete,
        )
    }
}

/// Outcome of [`WorthQueryDirectoryOpenDenial::retire_opening`].
pub enum WorthQueryDirectoryOpenRetired<Schema: ApplicationSchemaComposition, Program> {
    /// Opening refused before any Home existed; the denial is unchanged.
    Unretained(WorthQueryDirectoryOpenDenial<Schema, Program>),
    /// The refused opening's Home closed and was reclaimed; `refusal`
    /// describes why opening failed.
    Retired { refusal: String },
    /// Closing the refused opening refused; its owner or continuation is kept.
    RetirementRefused {
        refusal: String,
        close: WorthQueryHomeClosePolicyRefused<
            WorthQueryApplicationHomeOpenRefused<Schema, Program>,
            WorthQueryApplicationHomeOpeningCloseRefused<Schema, Program>,
        >,
    },
}

impl<Schema: ApplicationSchemaComposition, Program: 'static>
    WorthQueryDirectoryOpenDenial<Schema, Program>
{
    /// Releases the Directory a refused opening still holds: closes its Home
    /// under `policy` and reclaims it, keeping why opening failed.
    pub fn retire_opening(
        self,
        policy: ExecutionRequestPolicy,
    ) -> WorthQueryDirectoryOpenRetired<Schema, Program> {
        let refused = match self.into_home_refusal() {
            Ok(refused) => refused,
            Err(unretained) => return WorthQueryDirectoryOpenRetired::Unretained(unretained),
        };
        let refusal = format!("{refused:?}");
        match refused.close_with_policy(policy) {
            Ok(closed) => {
                closed.reclaim();
                WorthQueryDirectoryOpenRetired::Retired { refusal }
            }
            Err(close) => WorthQueryDirectoryOpenRetired::RetirementRefused { refusal, close },
        }
    }
}

impl<Schema: ApplicationSchemaComposition, Program> std::fmt::Display
    for WorthQueryDirectoryOpenRetired<Schema, Program>
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unretained(denial) => write!(formatter, "the Home did not open: {denial:?}"),
            Self::Retired { refusal } => {
                write!(
                    formatter,
                    "the Home did not open and was released: {refusal}"
                )
            }
            Self::RetirementRefused { refusal, close } => write!(
                formatter,
                "the Home did not open ({refusal}) and releasing it refused: {close:?}"
            ),
        }
    }
}
