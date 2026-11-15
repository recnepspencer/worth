//! Closed incoming strategies retain the same authored connection and ports.
use super::*;

mod source_seal {
    pub trait Sealed {}
}

/// The platform's finite prerequisite strategies, never an application executor.
pub trait ApplicationPrerequisiteSource: source_seal::Sealed + 'static {}

/// Incoming rows come from the connection's complete declared Query selection.
pub struct ApplicationStoredPrerequisites;
impl source_seal::Sealed for ApplicationStoredPrerequisites {}
impl ApplicationPrerequisiteSource for ApplicationStoredPrerequisites {}

/// Incoming rows come from the selected producer's readonly input preparation.
pub struct ApplicationCapturedPrerequisites<Capture>(PhantomData<fn() -> Capture>);
impl<Capture> source_seal::Sealed for ApplicationCapturedPrerequisites<Capture> {}
impl<Capture: 'static> ApplicationPrerequisiteSource for ApplicationCapturedPrerequisites<Capture> {}

/// One producer family's desired-source meaning and its complete typed ports.
///
/// `Parents` names the base connections; the program places their
/// [`ApplicationCapturedInputConnection`] forms in its complete component.
/// Initial and Preserve retain separate operation admissions for this meaning.
pub trait WorthQueryProducerInputCaptureDefinition<S: ApplicationSchema>: 'static {
    type OutputFamily: super::super::WorthQueryProducerOutputFamily<S>;
    type Parents: super::ApplicationOccurrenceConnections<S>;
    const IDENTITY: &'static str;
}

/// Selects captured incoming data without changing an edge's outgoing meaning.
pub struct ApplicationCapturedInputConnection<Connection, Capture>(
    PhantomData<fn() -> (Connection, Capture)>,
);
impl<Connection, Capture> sealed::ConnectionShape
    for ApplicationCapturedInputConnection<Connection, Capture>
{
}
impl<S, Connection, Capture> ApplicationConnectionShape<S>
    for ApplicationCapturedInputConnection<Connection, Capture>
where
    S: ApplicationSchema + 'static,
    Connection: ApplicationConnectionShape<S, Prerequisites = ApplicationStoredPrerequisites>,
    Capture: WorthQueryProducerInputCaptureDefinition<S>,
{
    type Binding = Connection::Binding;
    type Prerequisites = ApplicationCapturedPrerequisites<Capture>;
    type SourceFeature = Connection::SourceFeature;
    type SourcePort = Connection::SourcePort;
    type TargetFeature = Connection::TargetFeature;
    type TargetPort = Connection::TargetPort;

    fn declaration() -> ApplicationConnectionDeclaration {
        let mut connection = Connection::declaration();
        connection.set_input_capture::<Capture>(Capture::IDENTITY);
        connection
    }
}
