//! Pure family and connection meaning shared by ordinary and coupled output dispatch.
use crate::application_operation::ApplicationMutationBinding;
use crate::application_query::ApplicationQueryBinding;
use crate::application_schema::{
    ApplicationEntityMarkerIdentity, ApplicationSchema, ApplicationStructuredValueBinding,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum WorthQueryProducerLifecyclePosture {
    Initial,
    Preserve,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryProducerApplicability {
    profile_kind: &'static str,
    lifecycle: WorthQueryProducerLifecyclePosture,
}

impl WorthQueryProducerApplicability {
    pub const fn new(
        profile_kind: &'static str,
        lifecycle: WorthQueryProducerLifecyclePosture,
    ) -> Self {
        Self {
            profile_kind,
            lifecycle,
        }
    }

    pub const fn profile_kind(self) -> &'static str {
        self.profile_kind
    }

    pub const fn lifecycle(self) -> WorthQueryProducerLifecyclePosture {
        self.lifecycle
    }
}

pub trait WorthQueryProducerOutputFamily<Schema>: Sized + 'static
where
    Schema: ApplicationSchema,
{
    type Source: ApplicationQueryBinding<Schema>;
    /// The entity every producer of this family outputs. A producer whose
    /// output role names another entity fails to compile.
    type Entity: ApplicationEntityMarkerIdentity<Schema> + 'static;

    const IDENTITY: &'static str;
    const SUPPORTED: &'static [WorthQueryProducerApplicability];

    fn profile_kind(
        source: &<<Self::Source as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value,
    ) -> &'static str;
}

/// Typed request for one installed output family from one authored source.
pub trait WorthQueryApplicationOutputDemand<Schema>: Sized + 'static
where
    Schema: ApplicationSchema,
{
    type OutputFamily: WorthQueryProducerOutputFamily<Schema>;

    fn source_intent(
        &self,
    ) -> <<Self::OutputFamily as WorthQueryProducerOutputFamily<Schema>>::Source as ApplicationQueryBinding<Schema>>::Input;
}

/// One installed program connection from a performed source operation to a required output.
pub trait WorthQueryApplicationRequiredOutputConnection<Schema>: Sized + 'static
where
    Schema: ApplicationSchema,
{
    type Source: ApplicationMutationBinding<Schema>;
    type Demand: WorthQueryApplicationOutputDemand<Schema>;

    /// Canonical connection identity that must be present in the installed
    /// validated program before the source operation may publish.
    const IDENTITY: &'static str;

    fn demand_from_source(
        source: &<Self::Source as ApplicationMutationBinding<Schema>>::Input,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial>;
}

/// One mutation binding admitted as a source for an installed required-output
/// connection. Multiple operations owned by one feature may implement this for
/// the same connection without duplicating the feature port.
pub trait WorthQueryApplicationRequiredOutputSource<Schema, Connection>:
    ApplicationMutationBinding<Schema>
where
    Schema: ApplicationSchema,
    Connection: WorthQueryApplicationRequiredOutputConnection<Schema>,
{
    fn demand_from_source(
        source: &Self::Input,
    ) -> Result<Connection::Demand, WorthQueryRequiredOutputConnectionDenial>;

    /// Coupled retirement and restoration may derive an owner key from retained
    /// source fields. Ordinary published-source delivery retains its input-only path.
    fn demand_from_preimage(
        source: &Self::Input,
        _preimage: &dyn crate::application_aftermath::ApplicationCorrectionPreImage<Schema>,
    ) -> Result<Connection::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Self::demand_from_source(source)
    }
}

/// Refusal, from an application's output connection, to derive a discovery
/// query or output demands from its input.
///
/// Application code returns this from its connection functions; `subject`
/// names what could not be connected.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRequiredOutputConnectionDenial {
    subject: String,
}

impl WorthQueryRequiredOutputConnectionDenial {
    pub fn new(subject: impl Into<String>) -> Self {
        Self {
            subject: subject.into(),
        }
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }
}

impl std::fmt::Display for WorthQueryRequiredOutputConnectionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "required output connection denied: {}",
            self.subject
        )
    }
}

impl std::error::Error for WorthQueryRequiredOutputConnectionDenial {}
