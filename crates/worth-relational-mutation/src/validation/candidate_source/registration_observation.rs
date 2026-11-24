//! Read authority for the companion registration used by a candidate Source.
use worth_relational_contracts::internal_port::mvcc::publication::companion::registration_stop::PublicationCompanionRegistrationStop;

/// Each call observes the issuing owner's live registration with the same
/// refusal semantics as publication. The observer conveys no write authority.
pub trait ManagedSourceRegistration: std::fmt::Debug + Send + Sync {
    fn active_generation(&self) -> Result<Option<u64>, PublicationCompanionRegistrationStop>;
}
