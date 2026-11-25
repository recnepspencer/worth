//! A refused persistent-name output keeps the naming owner's exact typed cause.
use super::{
    AttemptDenialCause, WorthQueryApplicationAttemptDenial as Denial,
    WorthQueryApplicationAttemptDenialKind as Kind,
};
use worth_query_execution_naming::facade::WorthQueryOutputNameDenial;
impl Denial {
    /// An operation's declared output name was refused before any effect.
    pub fn persistent_name_output(cause: WorthQueryOutputNameDenial) -> Self {
        Self::from_cause(
            Kind::InvalidOutputRole,
            cause.to_string(),
            AttemptDenialCause::PersistentName(Box::new(cause)),
        )
    }
    /// The typed refusal behind a declared output name, when one was refused.
    pub fn persistent_name_denial(&self) -> Option<&WorthQueryOutputNameDenial> {
        match self.cause() {
            Some(AttemptDenialCause::PersistentName(cause)) => Some(cause),
            _ => None,
        }
    }
}
