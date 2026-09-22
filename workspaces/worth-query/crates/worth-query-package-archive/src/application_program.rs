//! Versioned, authority-free codecs for validated application-program meaning.

use worth_query_declaration::facade::application_program::{
    ApplicationProgramDefinition, ApplicationSemanticFact, ApplicationSemanticFamily,
    ValidatedApplicationProgram,
};
use worth_query_declaration::facade::application_schema::ApplicationSchema;

use crate::binary_input::BinaryInput;
use crate::binary_output::BinaryOutput;
use crate::compatibility::{
    WorthQueryPackageArchiveCompatibilityProfile, WorthQueryPackageArchiveProtocolLayer,
};
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};
use crate::limits::WorthQueryPackageArchiveLimits;

const MAGIC: &[u8; 4] = b"WQPG";
const HEADER_BYTES: u64 = 4 + 2 + 4;
const MINIMUM_SEMANTIC_FACT_BYTES: usize = 1 + 4 + 4;

#[cfg(test)]
mod tests;

/// Current deterministic application-program description protocol.
pub const WORTH_QUERY_APPLICATION_PROGRAM_ARCHIVE_PROTOCOL_VERSION: u16 = 1;

/// One decoded semantic fact. Decoding reconstructs description only; this
/// value cannot substitute for a validated program or installed support.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryUntrustedApplicationProgramSemanticFact {
    family: ApplicationSemanticFamily,
    subject: String,
    canonical_meaning: String,
}

impl WorthQueryUntrustedApplicationProgramSemanticFact {
    pub const fn family(&self) -> ApplicationSemanticFamily {
        self.family
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub fn canonical_meaning(&self) -> &str {
        &self.canonical_meaning
    }
}

/// Structurally decoded program meaning carrying no Query authority.
///
/// ```compile_fail,E0308
/// use worth_query_package_archive::facade::WorthQueryUntrustedApplicationProgramDescription;
/// use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
///
/// fn decoded_revision_is_not_validated(
///     decoded: &WorthQueryUntrustedApplicationProgramDescription,
/// ) -> &ApplicationProgramRevision {
///     decoded.revision()
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryUntrustedApplicationProgramDescription {
    identity: String,
    revision: String,
    facts: Box<[WorthQueryUntrustedApplicationProgramSemanticFact]>,
}

impl WorthQueryUntrustedApplicationProgramDescription {
    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub fn revision(&self) -> &str {
        &self.revision
    }

    pub fn facts(&self) -> &[WorthQueryUntrustedApplicationProgramSemanticFact] {
        &self.facts
    }

    pub fn compatibility_with(
        &self,
        other: &Self,
    ) -> WorthQueryApplicationProgramArchiveCompatibility {
        if self.identity != other.identity {
            WorthQueryApplicationProgramArchiveCompatibility::DifferentProgram
        } else if self.revision == other.revision && self.facts == other.facts {
            WorthQueryApplicationProgramArchiveCompatibility::ExactRevision
        } else {
            WorthQueryApplicationProgramArchiveCompatibility::ChangedRevision
        }
    }

    /// Compares decoded description with freshly validated typed meaning.
    /// Exact compatibility requires identity, canonical revision and every
    /// ordered semantic fact; decoded bytes alone never mint authority.
    pub fn compatibility_with_validated<Schema, Program>(
        &self,
        program: &ValidatedApplicationProgram<Schema, Program>,
    ) -> WorthQueryApplicationProgramArchiveCompatibility
    where
        Schema: ApplicationSchema,
        Program: ApplicationProgramDefinition<Schema>,
    {
        if self.identity != program.identity().as_str() {
            return WorthQueryApplicationProgramArchiveCompatibility::DifferentProgram;
        }
        let expected = program.semantic_description().facts();
        if self.revision == program.revision().to_string()
            && self.facts.len() == expected.len()
            && self
                .facts
                .iter()
                .zip(expected)
                .all(|(decoded, validated)| decoded.matches(validated))
        {
            WorthQueryApplicationProgramArchiveCompatibility::ExactRevision
        } else {
            WorthQueryApplicationProgramArchiveCompatibility::ChangedRevision
        }
    }
}

impl WorthQueryUntrustedApplicationProgramSemanticFact {
    fn matches(&self, validated: &ApplicationSemanticFact) -> bool {
        self.family == validated.family()
            && self.subject == validated.subject()
            && self.canonical_meaning == validated.canonical_meaning()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationProgramArchiveCompatibility {
    ExactRevision,
    ChangedRevision,
    DifferentProgram,
}

/// Encodes one freshly validated program into stable descriptive bytes.
pub fn encode_application_program_description<Schema, Program>(
    program: &ValidatedApplicationProgram<Schema, Program>,
    limits: WorthQueryPackageArchiveLimits,
) -> Result<Vec<u8>, Denial>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    encode_description(
        program.identity().as_str(),
        &program.revision().to_string(),
        program.semantic_description().facts(),
        limits,
    )
}

/// Decodes bounded descriptive bytes without manufacturing validated meaning.
pub fn decode_application_program_description(
    bytes: &[u8],
    limits: WorthQueryPackageArchiveLimits,
) -> Result<WorthQueryUntrustedApplicationProgramDescription, Denial> {
    let limits = limits.narrowed();
    require_byte_budget(bytes.len(), limits)?;
    let mut input = BinaryInput::new(bytes);
    if &input.array::<4>()? != MAGIC {
        return Err(Denial::new(Kind::InvalidMagic));
    }
    WorthQueryPackageArchiveCompatibilityProfile::CURRENT
        .admit(
            WorthQueryPackageArchiveProtocolLayer::ApplicationProgramDescription,
            input.u16()?,
        )
        .map_err(|compatibility| {
            Denial::incompatible(
                Kind::UnsupportedApplicationProgramDescriptionVersion,
                compatibility,
            )
        })?;
    let identity = input.text()?.to_owned();
    let revision = input.text()?.to_owned();
    if identity.is_empty() || revision.is_empty() {
        return Err(Denial::new(Kind::InvalidRecordShape));
    }
    let count = input.u32()?;
    if u64::from(count) > limits.maximum_nested_entries() {
        return Err(Denial::new(Kind::NestedEntryBudgetExceeded));
    }
    let fact_count = usize::try_from(count).map_err(|_| Denial::new(Kind::NumericWidthExceeded))?;
    let minimum_fact_bytes = fact_count
        .checked_mul(MINIMUM_SEMANTIC_FACT_BYTES)
        .ok_or_else(|| Denial::new(Kind::Truncated))?;
    if minimum_fact_bytes > input.remaining_len() {
        return Err(Denial::new(Kind::Truncated));
    }
    let mut facts = Vec::with_capacity(fact_count);
    for _ in 0..count {
        facts.push(WorthQueryUntrustedApplicationProgramSemanticFact {
            family: decode_family(input.u8()?)?,
            subject: input.text()?.to_owned(),
            canonical_meaning: input.text()?.to_owned(),
        });
    }
    if !input.is_finished() {
        return Err(Denial::new(Kind::TrailingBytes));
    }
    // Validated semantic meaning is a sorted multiset, so equal adjacent facts
    // are canonical; only a descending pair is an invalid wire ordering.
    if facts.windows(2).any(|pair| pair[0] > pair[1]) {
        return Err(Denial::new(Kind::NonCanonicalRecordSequence));
    }
    Ok(WorthQueryUntrustedApplicationProgramDescription {
        identity,
        revision,
        facts: facts.into_boxed_slice(),
    })
}

fn encode_description(
    identity: &str,
    revision: &str,
    facts: &[ApplicationSemanticFact],
    limits: WorthQueryPackageArchiveLimits,
) -> Result<Vec<u8>, Denial> {
    let limits = limits.narrowed();
    if identity.is_empty() || revision.is_empty() {
        return Err(Denial::new(Kind::InvalidRecordShape));
    }
    let fact_count =
        u32::try_from(facts.len()).map_err(|_| Denial::new(Kind::NestedEntryBudgetExceeded))?;
    if u64::from(fact_count) > limits.maximum_nested_entries() {
        return Err(Denial::new(Kind::NestedEntryBudgetExceeded));
    }
    let bytes = encoded_length(identity, revision, facts)?;
    require_byte_budget(
        usize::try_from(bytes).map_err(|_| Denial::new(Kind::LogicalByteBudgetExceeded))?,
        limits,
    )?;
    let mut output = BinaryOutput::with_capacity(
        usize::try_from(bytes).map_err(|_| Denial::new(Kind::LogicalByteBudgetExceeded))?,
    );
    output.raw_bytes(MAGIC);
    output.u16(WORTH_QUERY_APPLICATION_PROGRAM_ARCHIVE_PROTOCOL_VERSION);
    output.text(identity);
    output.text(revision);
    output.u32(fact_count);
    for fact in facts {
        output.raw_bytes(&[encode_family(fact.family())]);
        output.text(fact.subject());
        output.text(fact.canonical_meaning());
    }
    Ok(output.into_bytes())
}

fn encoded_length(
    identity: &str,
    revision: &str,
    facts: &[ApplicationSemanticFact],
) -> Result<u64, Denial> {
    let mut bytes = checked_add_length(HEADER_BYTES, text_length(identity)?)?;
    bytes = checked_add_length(bytes, text_length(revision)?)?;
    for fact in facts {
        bytes = checked_add_length(bytes, 1)?;
        bytes = checked_add_length(bytes, text_length(fact.subject())?)?;
        bytes = checked_add_length(bytes, text_length(fact.canonical_meaning())?)?;
    }
    Ok(bytes)
}

fn checked_add_length(bytes: u64, addition: u64) -> Result<u64, Denial> {
    bytes
        .checked_add(addition)
        .ok_or_else(|| Denial::new(Kind::LogicalByteBudgetExceeded))
}

fn text_length(value: &str) -> Result<u64, Denial> {
    let length = u32::try_from(value.len()).map_err(|_| Denial::new(Kind::NumericWidthExceeded))?;
    Ok(4 + u64::from(length))
}

fn require_byte_budget(bytes: usize, limits: WorthQueryPackageArchiveLimits) -> Result<(), Denial> {
    let bytes = u64::try_from(bytes).map_err(|_| Denial::new(Kind::LogicalByteBudgetExceeded))?;
    if bytes > limits.maximum_archive_bytes() || bytes > limits.maximum_logical_bytes() {
        Err(Denial::new(Kind::LogicalByteBudgetExceeded))
    } else {
        Ok(())
    }
}

const fn encode_family(family: ApplicationSemanticFamily) -> u8 {
    match family {
        ApplicationSemanticFamily::Features => 1,
        ApplicationSemanticFamily::Ports => 2,
        ApplicationSemanticFamily::Connections => 3,
        ApplicationSemanticFamily::Rules => 4,
        ApplicationSemanticFamily::Operations => 5,
        ApplicationSemanticFamily::ExternalInputs => 6,
        ApplicationSemanticFamily::Outputs => 7,
        ApplicationSemanticFamily::Resources => 8,
    }
}

const fn decode_family(value: u8) -> Result<ApplicationSemanticFamily, Denial> {
    match value {
        1 => Ok(ApplicationSemanticFamily::Features),
        2 => Ok(ApplicationSemanticFamily::Ports),
        3 => Ok(ApplicationSemanticFamily::Connections),
        4 => Ok(ApplicationSemanticFamily::Rules),
        5 => Ok(ApplicationSemanticFamily::Operations),
        6 => Ok(ApplicationSemanticFamily::ExternalInputs),
        7 => Ok(ApplicationSemanticFamily::Outputs),
        8 => Ok(ApplicationSemanticFamily::Resources),
        _ => Err(Denial::new(Kind::UnsupportedRecordVariant)),
    }
}
