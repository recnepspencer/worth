#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SettingValueSchema {
    Boolean,
    Integer,
    Decimal,
    Text,
    Enumeration(Vec<String>),
}

impl SettingValueSchema {
    pub fn boolean() -> Self {
        Self::Boolean
    }

    pub fn integer() -> Self {
        Self::Integer
    }

    pub fn decimal() -> Self {
        Self::Decimal
    }

    pub fn text() -> Self {
        Self::Text
    }

    pub fn enumeration(options: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self::Enumeration(options.into_iter().map(Into::into).collect())
    }

    pub(crate) fn admits_default_value(&self, value: &super::SettingDefaultValue) -> bool {
        match (self, value) {
            (Self::Boolean, super::SettingDefaultValue::Boolean(_))
            | (Self::Integer, super::SettingDefaultValue::Integer(_))
            | (Self::Text, super::SettingDefaultValue::Text(_)) => true,
            (Self::Decimal, super::SettingDefaultValue::Decimal(value)) => {
                is_decimal_literal(value)
            }
            (Self::Enumeration(options), super::SettingDefaultValue::Enumeration(value)) => {
                options.iter().any(|option| option == value)
            }
            _ => false,
        }
    }

    pub(crate) fn is_valid_schema(&self) -> bool {
        match self {
            Self::Enumeration(options) => {
                !options.is_empty() && options.iter().all(|option| !option.trim().is_empty()) && {
                    let mut sorted = options.clone();
                    sorted.sort();
                    sorted.dedup();
                    sorted.len() == options.len()
                }
            }
            Self::Boolean | Self::Integer | Self::Decimal | Self::Text => true,
        }
    }

    pub(crate) fn digest_basis(&self) -> String {
        match self {
            Self::Boolean => "boolean".to_string(),
            Self::Integer => "integer".to_string(),
            Self::Decimal => "decimal".to_string(),
            Self::Text => "text".to_string(),
            Self::Enumeration(options) => {
                let options = options
                    .iter()
                    .map(|option| length_prefixed(option))
                    .collect::<Vec<_>>()
                    .join("");
                format!("enum[{options}]")
            }
        }
    }
}

fn length_prefixed(value: &str) -> String {
    format!("{}:{value}", value.len())
}

fn is_decimal_literal(value: &str) -> bool {
    let unsigned = value.strip_prefix('-').unwrap_or(value);
    if unsigned.is_empty() {
        return false;
    }

    let mut parts = unsigned.split('.');
    let whole = parts.next().unwrap_or_default();
    let fractional = parts.next();
    if parts.next().is_some() || whole.is_empty() {
        return false;
    }

    whole.chars().all(|character| character.is_ascii_digit())
        && fractional.is_none_or(|part| {
            !part.is_empty() && part.chars().all(|character| character.is_ascii_digit())
        })
}
