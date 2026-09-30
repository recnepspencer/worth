/// Why a family member's role name cannot enter an application candidate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationOutputRoleNameDenial {
    /// The name, or a family member's suffix, is empty.
    Empty,
    SurroundingWhitespace,
    ControlCharacter,
    RepresentationTooLarge {
        maximum_bytes: usize,
        required_bytes: usize,
    },
}

const MAXIMUM_OUTPUT_ROLE_NAME_BYTES: usize = 256;

impl WorthQueryApplicationOutputRoleNameDenial {
    /// Check one output-role name: it is non-empty, carries no surrounding
    /// whitespace or control characters, and fits the representation bound.
    pub fn validate(name: &str) -> Result<(), Self> {
        if name.is_empty() {
            return Err(Self::Empty);
        }
        if name.trim() != name {
            return Err(Self::SurroundingWhitespace);
        }
        if name.chars().any(char::is_control) {
            return Err(Self::ControlCharacter);
        }
        if name.len() > MAXIMUM_OUTPUT_ROLE_NAME_BYTES {
            return Err(Self::RepresentationTooLarge {
                maximum_bytes: MAXIMUM_OUTPUT_ROLE_NAME_BYTES,
                required_bytes: name.len(),
            });
        }
        Ok(())
    }
}

impl std::fmt::Display for WorthQueryApplicationOutputRoleNameDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => formatter.write_str(
                "an application output role name or family member suffix cannot be empty",
            ),
            Self::SurroundingWhitespace => formatter
                .write_str("an application output role name cannot contain surrounding whitespace"),
            Self::ControlCharacter => formatter
                .write_str("an application output role name cannot contain control characters"),
            Self::RepresentationTooLarge {
                maximum_bytes,
                required_bytes,
            } => write!(
                formatter,
                "application output role name requires {required_bytes} bytes but the maximum is {maximum_bytes}"
            ),
        }
    }
}

impl std::error::Error for WorthQueryApplicationOutputRoleNameDenial {}
