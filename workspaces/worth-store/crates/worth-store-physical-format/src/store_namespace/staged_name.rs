use core::fmt;

const STAGED_IDENTITY_PREFIX: &str = "identity-";
const STAGED_IDENTITY_SUFFIX: &str = ".staged";
const ATTEMPT_BYTES: usize = 16;

/// Non-authoritative correlation identity for one initialization attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NamespaceInitializationAttempt([u8; ATTEMPT_BYTES]);

impl NamespaceInitializationAttempt {
    /// Validates representation only; entropy provenance belongs to the owner.
    pub fn from_nonzero_bytes(bytes: [u8; ATTEMPT_BYTES]) -> Option<Self> {
        (bytes != [0; ATTEMPT_BYTES]).then_some(Self(bytes))
    }

    pub const fn bytes(self) -> [u8; ATTEMPT_BYTES] {
        self.0
    }
}

/// Canonical name for unpublished initialization residue.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StagedNamespaceName(String);

impl StagedNamespaceName {
    pub fn for_identity(attempt: NamespaceInitializationAttempt) -> Self {
        let mut name = String::with_capacity(
            STAGED_IDENTITY_PREFIX.len() + ATTEMPT_BYTES * 2 + STAGED_IDENTITY_SUFFIX.len(),
        );
        name.push_str(STAGED_IDENTITY_PREFIX);
        for byte in attempt.bytes() {
            use core::fmt::Write;
            write!(&mut name, "{byte:02x}").expect("writing into String cannot fail");
        }
        name.push_str(STAGED_IDENTITY_SUFFIX);
        Self(name)
    }

    pub fn parse(name: &str) -> Option<Self> {
        let encoded = name
            .strip_prefix(STAGED_IDENTITY_PREFIX)?
            .strip_suffix(STAGED_IDENTITY_SUFFIX)?;
        if encoded.len() != ATTEMPT_BYTES * 2
            || !encoded.bytes().all(|byte| byte.is_ascii_hexdigit())
            || encoded.bytes().any(|byte| byte.is_ascii_uppercase())
        {
            return None;
        }
        let mut bytes = [0; ATTEMPT_BYTES];
        for (index, pair) in encoded.as_bytes().as_chunks::<2>().0.iter().enumerate() {
            bytes[index] = decode_hex(pair[0])? << 4 | decode_hex(pair[1])?;
        }
        NamespaceInitializationAttempt::from_nonzero_bytes(bytes)?;
        Some(Self(name.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for StagedNamespaceName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

fn decode_hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}
