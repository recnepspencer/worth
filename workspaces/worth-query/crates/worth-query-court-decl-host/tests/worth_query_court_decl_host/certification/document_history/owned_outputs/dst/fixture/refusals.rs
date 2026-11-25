//! Real refusals in the model's terms. A refusal the model does not name
//! maps to `None`, and the world fails the seed with it: it is traced before
//! it is named (RULES §8), never guessed from a code read.
use super::super::reference::{Environment, Outcome};
use super::envelope::{envelope, Executed, Push};
use super::reads::{Forest, Host};
use super::*;
use worth_query_host::facade::product::WorthQueryProductBranch;

/// An envelope's refusal. A refusal the model names keeps its name; under a
/// request the call stopped, any other refusal is that stop, wherever the
/// request was when it stopped, and the oracle judges the stop by what it
/// left (as Phase 1's `output_forest/stopped_request` court does). Traced
/// (S1 stop-point sweep): a stop at each consult 1..1023 of a two-stage edit
/// refuses with the stop's own cause, from the history read's product,
/// principal, scope, admission and one-shot read, the candidate's selection
/// and creation, the recording, or the stage's execution.
pub fn classify(denial: &WorthQueryHistoryEditEnvelopeDenial, stopped: bool) -> Option<Outcome> {
    named(denial).or(stopped.then_some(Outcome::Stopped))
}

fn named(denial: &WorthQueryHistoryEditEnvelopeDenial) -> Option<Outcome> {
    use WorthQueryHistoryEditEnvelopeDenial as Denial;
    match denial {
        Denial::Empty => Some(Outcome::Empty),
        Denial::History(WorthQueryApplicationHistoryDenial::Document(denial)) | Denial::Stage { denial, .. } => {
            document(denial)
        }
        Denial::Envelope(WorthQueryEditEnvelopeOutcome::DomainDenied(_)) => Some(Outcome::StageRefused),
        // The correction's source cannot be reversed: already reversed,
        // no longer current, or diverged (the model's RefuseCorrectionSource).
        Denial::Correction(worth_query_host::facade::application_entry::WorthQueryApplicationCorrectionDenial::Derivation(_)) => {
            Some(Outcome::SourceRefused)
        }
        _ => None,
    }
}

fn document(denial: &WorthQueryDocumentHistoryDenial) -> Option<Outcome> {
    use WorthQueryDocumentHistoryDenial as Denial;
    Some(match denial {
        Denial::HeadDiverged => Outcome::HeadDiverged,
        Denial::IntentDrift => Outcome::IntentDrift,
        Denial::SourceNotOnSelectedLine { .. } => Outcome::SourceRefused,
        Denial::StepMetadataTooLarge { .. } => Outcome::Environment(Environment::MetadataTooLarge),
        Denial::ForeignModel => Outcome::Environment(Environment::ForeignModel),
        Denial::UnownedOperationScope { .. } => Outcome::Environment(Environment::UnownedOperationScope),
        Denial::OperationNotDeclared => Outcome::Environment(Environment::OperationNotDeclared),
        Denial::UnsupportedOperation => Outcome::Environment(Environment::UnsupportedCorrection),
        Denial::Stopped(_) => Outcome::Stopped,
        _ => return None,
    })
}

/// A navigation's refusal, from `selection`, `prepare` or `advance`.
pub fn classify_navigation(denial: &WorthQueryHistoryNavigationAdvanceDenial) -> Option<Outcome> {
    use WorthQueryHistoryNavigationAdvanceDenial as Denial;
    use WorthQueryHistoryNavigationDenial as Navigation;
    match denial {
        Denial::Navigation(Navigation::NoParent | Navigation::NotChild) => Some(Outcome::NavigationRefused),
        // Traced (seed 0x7b020000): with output trees bound, the move's
        // structural refusal comes from the output navigation.
        Denial::Outputs(WorthQueryHistoryOutputDenial::Navigation(Navigation::NoParent | Navigation::NotChild)) => {
            Some(Outcome::NavigationRefused)
        }
        Denial::Navigation(Navigation::Document(WorthQueryDocumentHistoryDenial::HeadDiverged)) => {
            Some(Outcome::HeadDiverged)
        }
        Denial::History(denial) => classify_read(denial),
        _ => None,
    }
}

/// A refused read or maintenance pass.
pub fn classify_read(denial: &WorthQueryApplicationHistoryDenial) -> Option<Outcome> {
    match denial {
        WorthQueryApplicationHistoryDenial::Document(WorthQueryDocumentHistoryDenial::HeadDiverged) => {
            Some(Outcome::HeadDiverged)
        }
        _ => None,
    }
}

/// An envelope built to meet `cause`: each refuses before any effect. The
/// real owned document cannot meet the others (S1 report).
pub fn provoke<'h>(
    host: &'h Host,
    document: Forest<'h>,
    branch: WorthQueryProductBranch,
    key: String,
    cause: Environment,
) -> (Forest<'h>, Outcome) {
    let maximum = ApplicationHistoryRetentionResources::DEFAULT.maximum_step_metadata_bytes;
    let (push, metadata) = match cause {
        Environment::MetadataTooLarge => (Push::OwnedModel, vec![7; maximum + 1]),
        Environment::UnownedOperationScope => (Push::UnownedModel, Vec::new()),
        Environment::OperationNotDeclared => (Push::UndeclaredModel, Vec::new()),
        other => panic!("the owned document cannot meet {other:?}"),
    };
    match envelope(host, document, branch, key, &[push], metadata, None) {
        Executed::Refused { document, denial } => {
            let outcome = classify(&denial, false)
                .unwrap_or_else(|| panic!("{cause:?}: a refusal the model does not name: {denial:?}"));
            (document, outcome)
        }
        Executed::Recorded { .. } | Executed::AlreadyRecorded { .. } | Executed::Unselectable { .. } => {
            panic!("{cause:?} must refuse")
        }
    }
}
