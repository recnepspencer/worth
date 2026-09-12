const MAX_ACCOUNT_NAME_BYTES: usize = 120;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountName(String);

impl AccountName {
    pub const MAX_BYTES: usize = MAX_ACCOUNT_NAME_BYTES;

    pub fn new(value: impl Into<String>) -> Result<Self, AccountNameDenial> {
        let value = value.into();
        let invalid = value.is_empty()
            || value.trim() != value
            || value.len() > MAX_ACCOUNT_NAME_BYTES
            || value.chars().any(char::is_control);
        if invalid {
            return Err(AccountNameDenial);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn into_string(self) -> String {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccountNameDenial;

impl std::fmt::Display for AccountNameDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("account name must be trimmed, nonempty, bounded, and printable")
    }
}

impl std::error::Error for AccountNameDenial {}
