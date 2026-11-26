//! The bounded code and message grammar every domain refusal satisfies.

/// The longest admitted refusal code, in bytes.
pub const WORTH_QUERY_DOMAIN_REFUSAL_CODE_MAXIMUM_BYTES: usize = 64;
/// The longest admitted refusal message, in UTF-8 bytes.
pub const WORTH_QUERY_DOMAIN_REFUSAL_MESSAGE_MAXIMUM_BYTES: usize = 256;

/// Why a code or message is not an admissible domain refusal.
///
/// Lengths are checked before characters, so the first violation found is
/// the same for the same input on every run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDomainRefusalDenial {
    /// The code is empty.
    CodeEmpty,
    /// The code is longer than its maximum.
    CodeTooLong { maximum: usize, observed: usize },
    /// The code byte at `position` is outside its grammar (the first byte must
    /// be `a-z`; later bytes `a-z`, `0-9`, `_`, `.` or `-`).
    CodeCharacter { position: usize },
    /// The message is empty or only whitespace.
    MessageBlank,
    /// The message is longer than its maximum.
    MessageTooLong { maximum: usize, observed: usize },
    /// The message holds a control character (such as a line break) starting
    /// at byte `position`.
    MessageControlCharacter { position: usize },
}

impl WorthQueryDomainRefusalDenial {
    /// A fixed sentence naming the violated rule, with no input echoed back.
    pub const fn reason(self) -> &'static str {
        match self {
            Self::CodeEmpty => "domain refusal code is empty",
            Self::CodeTooLong { .. } => "domain refusal code exceeds 64 bytes",
            Self::CodeCharacter { .. } => {
                "domain refusal code must be a-z, then a-z, 0-9, '_', '.' or '-'"
            }
            Self::MessageBlank => "domain refusal message is blank",
            Self::MessageTooLong { .. } => "domain refusal message exceeds 256 bytes",
            Self::MessageControlCharacter { .. } => {
                "domain refusal message contains a control character"
            }
        }
    }
}

impl std::fmt::Display for WorthQueryDomainRefusalDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.reason())
    }
}

impl std::error::Error for WorthQueryDomainRefusalDenial {}

pub(super) fn admit_code(code: &str) -> Result<(), WorthQueryDomainRefusalDenial> {
    use WorthQueryDomainRefusalDenial as Denial;
    let bytes = code.as_bytes();
    let Some((first, rest)) = bytes.split_first() else {
        return Err(Denial::CodeEmpty);
    };
    if bytes.len() > WORTH_QUERY_DOMAIN_REFUSAL_CODE_MAXIMUM_BYTES {
        return Err(Denial::CodeTooLong {
            maximum: WORTH_QUERY_DOMAIN_REFUSAL_CODE_MAXIMUM_BYTES,
            observed: bytes.len(),
        });
    }
    if !first.is_ascii_lowercase() {
        return Err(Denial::CodeCharacter { position: 0 });
    }
    match rest.iter().position(|byte| !is_code_continuation(*byte)) {
        Some(offset) => Err(Denial::CodeCharacter {
            position: offset + 1,
        }),
        None => Ok(()),
    }
}

pub(super) fn admit_message(message: &str) -> Result<(), WorthQueryDomainRefusalDenial> {
    use WorthQueryDomainRefusalDenial as Denial;
    if message.len() > WORTH_QUERY_DOMAIN_REFUSAL_MESSAGE_MAXIMUM_BYTES {
        return Err(Denial::MessageTooLong {
            maximum: WORTH_QUERY_DOMAIN_REFUSAL_MESSAGE_MAXIMUM_BYTES,
            observed: message.len(),
        });
    }
    if let Some((position, _)) = message.char_indices().find(|(_, c)| c.is_control()) {
        return Err(Denial::MessageControlCharacter { position });
    }
    if message.trim().is_empty() {
        return Err(Denial::MessageBlank);
    }
    Ok(())
}

const fn is_code_continuation(byte: u8) -> bool {
    byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'.' | b'-')
}
