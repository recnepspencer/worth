//! One live Server assembly's address domain; neither Query identity nor execution authority.
use std::{fmt::Write, sync::Arc};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerInstanceIdentity(Arc<str>);

impl WorthServerInstanceIdentity {
    pub(crate) fn fresh() -> Result<Self, getrandom::Error> {
        let mut nonce = [0; 32];
        getrandom::fill(&mut nonce)?;
        let mut value = String::with_capacity(80);
        value.push_str("server-instance:");
        for byte in nonce {
            write!(value, "{byte:02x}").expect("writing to a String is infallible");
        }
        Ok(Self(Arc::from(value)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
