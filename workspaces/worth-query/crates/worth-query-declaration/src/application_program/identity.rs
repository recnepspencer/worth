use std::borrow::Cow;

/// Stable authored identity of one immutable application program.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationProgramIdentity(Cow<'static, str>);

impl ApplicationProgramIdentity {
    pub const fn new(identity: &'static str) -> Self {
        Self(Cow::Borrowed(identity))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}
