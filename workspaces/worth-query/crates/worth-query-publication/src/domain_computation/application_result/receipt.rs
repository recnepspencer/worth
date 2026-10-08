use worth_query_execution::facade::primary_graph::WorthQueryApplicationQueryAccessReceipt;

use super::WorthQueryApplicationQueryPublicationInspection;
use super::WorthQueryPublishedApplicationBasis;
use super::WorthQueryPublishedApplicationDisclosure;
use super::WorthQueryPublishedApplicationQueryTerminalRelease;
use super::WorthQueryPublishedApplicationReadWork;
use crate::application_aftermath::WorthQueryPublishedCanonicalWork;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPublishedApplicationQueryOmissionPosture {
    NoOmission,
    GovernedOmission,
}

/// Publication-owned evidence derived from one completed application query.
///
/// This receipt stores only a closed publication projection. It cannot be
/// constructed by consumers and is not accepted by an execution transition.
///
/// ```compile_fail
/// use worth_query_execution::facade::primary_graph::WorthQueryApplicationQueryAccessReceipt;
/// use worth_query_publication::facade::domain_computation::WorthQueryApplicationQueryPublicationReceipt;
///
/// fn counterfeit(
///     terminal: WorthQueryApplicationQueryAccessReceipt,
/// ) -> WorthQueryApplicationQueryPublicationReceipt {
///     let _ = terminal;
///     WorthQueryApplicationQueryPublicationReceipt {
///         result_count: todo!(),
///         ordinary_work_units: todo!(),
///         disclosure: todo!(),
///         publication_work: todo!(),
///         terminal_release: todo!(),
///     }
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationQueryPublicationReceipt {
    query_identity: String,
    parameter_binding_identity: String,
    basis: WorthQueryPublishedApplicationBasis,
    result_count: usize,
    ordinary_work_units: usize,
    read_work: WorthQueryPublishedApplicationReadWork,
    authorization_work_units: usize,
    disclosure: WorthQueryPublishedApplicationDisclosure,
    publication_work: WorthQueryPublishedCanonicalWork,
    terminal_release: WorthQueryPublishedApplicationQueryTerminalRelease,
}

impl WorthQueryApplicationQueryPublicationReceipt {
    pub(super) fn from_terminal(terminal: &WorthQueryApplicationQueryAccessReceipt) -> Self {
        let disclosure = WorthQueryPublishedApplicationDisclosure::capture(terminal.disclosure());
        Self {
            query_identity: terminal.query_identity().render_support_hex(),
            parameter_binding_identity: terminal.parameter_binding_identity().render_hex(),
            basis: WorthQueryPublishedApplicationBasis::capture(terminal),
            result_count: terminal.result_count(),
            ordinary_work_units: terminal.total_work_units(),
            read_work: WorthQueryPublishedApplicationReadWork::capture(terminal),
            authorization_work_units: terminal.authorization_work().observation_work_units(),
            disclosure,
            publication_work: WorthQueryPublishedCanonicalWork::from_owner(
                terminal.canonical_work().publication(),
            ),
            terminal_release: WorthQueryPublishedApplicationQueryTerminalRelease::capture(terminal),
        }
    }

    pub const fn inspect(&self) -> WorthQueryApplicationQueryPublicationInspection<'_> {
        WorthQueryApplicationQueryPublicationInspection::new(self)
    }

    pub(super) const fn result_count(&self) -> usize {
        self.result_count
    }
    pub(super) fn query_identity(&self) -> &str {
        &self.query_identity
    }
    pub(super) fn parameter_binding_identity(&self) -> &str {
        &self.parameter_binding_identity
    }
    pub(super) const fn basis(&self) -> &WorthQueryPublishedApplicationBasis {
        &self.basis
    }
    pub(super) const fn ordinary_work_units(&self) -> usize {
        self.ordinary_work_units
    }
    pub(super) const fn read_work(&self) -> WorthQueryPublishedApplicationReadWork {
        self.read_work
    }
    pub(super) const fn authorization_work_units(&self) -> usize {
        self.authorization_work_units
    }
    pub(super) const fn omission_posture(
        &self,
    ) -> WorthQueryPublishedApplicationQueryOmissionPosture {
        if self.disclosure.has_omissions() {
            WorthQueryPublishedApplicationQueryOmissionPosture::GovernedOmission
        } else {
            WorthQueryPublishedApplicationQueryOmissionPosture::NoOmission
        }
    }
    pub const fn disclosure(&self) -> &WorthQueryPublishedApplicationDisclosure {
        &self.disclosure
    }
    pub(super) const fn publication_work(&self) -> WorthQueryPublishedCanonicalWork {
        self.publication_work
    }
    pub(super) const fn terminal_release(
        &self,
    ) -> WorthQueryPublishedApplicationQueryTerminalRelease {
        self.terminal_release
    }
}
