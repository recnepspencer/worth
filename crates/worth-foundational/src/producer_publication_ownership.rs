//! Installed producer lifecycle meaning; a reference never declares ownership.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum ProducerPublicationOwnership {
    /// Publication retirement remains explicitly controlled by its producer.
    #[default]
    Independent,
    /// Complete publications cascade with their exact entity-specific source.
    SourceScopeCascade,
}
