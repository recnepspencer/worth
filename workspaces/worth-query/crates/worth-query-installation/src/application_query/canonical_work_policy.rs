use worth_foundational::facade::CanonicalDigestWorkBudget;
use worth_query_declaration::facade::application_query::ErasedApplicationQueryDefinition;

const PARAMETER_ENCODED_BYTE_LIMIT: usize = 256 * 1024;
const ADMISSION_PLANNING_ENTRY_LIMIT: u32 = 4_096;
const ADMISSION_PLANNING_ENCODED_BYTE_LIMIT: usize = 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationQueryCanonicalWorkPolicy {
    installation: CanonicalDigestWorkBudget,
    parameters: CanonicalDigestWorkBudget,
    admission_planning: CanonicalDigestWorkBudget,
}

impl WorthQueryApplicationQueryCanonicalWorkPolicy {
    pub(super) fn for_definition(
        definition: &ErasedApplicationQueryDefinition,
        installation: CanonicalDigestWorkBudget,
    ) -> Self {
        let parameter_entries = definition
            .parameters()
            .len()
            .checked_mul(3)
            .and_then(|count| count.checked_add(1))
            .and_then(|count| u32::try_from(count).ok())
            .unwrap_or(u32::MAX);
        Self {
            installation,
            parameters: CanonicalDigestWorkBudget::new(
                parameter_entries,
                PARAMETER_ENCODED_BYTE_LIMIT,
            )
            .expect("the parameter canonical budget includes its count entry"),
            admission_planning: CanonicalDigestWorkBudget::new(
                ADMISSION_PLANNING_ENTRY_LIMIT,
                ADMISSION_PLANNING_ENCODED_BYTE_LIMIT,
            )
            .expect("the application-query admission planning budget is nonzero"),
        }
    }

    pub const fn installation(self) -> CanonicalDigestWorkBudget {
        self.installation
    }

    pub const fn parameters(self) -> CanonicalDigestWorkBudget {
        self.parameters
    }

    pub const fn admission_planning(self) -> CanonicalDigestWorkBudget {
        self.admission_planning
    }
}
