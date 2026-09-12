use worth_foundational::facade::{
    canonicalization, prepare_canonical_basis_sequence, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisValue, CanonicalDigestAlgorithmId,
    CanonicalDigestId, CanonicalIntegerWidth, CanonicalizationRuleVersion,
};
use worth_query_decl::facade::{
    application_operation::{
        ApplicationMutationOutputContract, ApplicationMutationOutputPosture,
        ApplicationMutationOutputRoleDescriptor,
    },
    application_schema::{NoApplicationUnit, ReadOnly},
    worth_query_mutation_binding, worth_query_structured_value_binding,
};

use crate::model::{AccountId, AccountName, BankPrincipalId, InstitutionId};
use crate::proposals::{BankIdempotencyKey, BankProposalDenial};

use super::{
    CreatePersonalAccount, CreatePersonalAccountInputBinding, CreatePersonalAccountOperation,
};
use crate::schema::{
    Account, BankPrincipalBinding, BankPrincipalIdBinding, BankSchema, ExternalPrincipalMapping,
    Institution, InstitutionIdentity, InstitutionIdentityField, Principal,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CreatePersonalAccountResult {
    pub account: AccountId,
}

worth_query_structured_value_binding!(
    pub CreatePersonalAccountResultBinding for CreatePersonalAccountResult {
        identity: "bank.operation.create-personal-account.result.v1"
    }
);

worth_query_structured_value_binding!(
    pub CreatePersonalAccountDenialBinding for BankProposalDenial {
        identity: "bank.operation.create-personal-account.denial.v1"
    }
);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreatePersonalAccountDecision {
    account: AccountId,
    institution: InstitutionId,
    owner: BankPrincipalId,
    display_name: AccountName,
}

pub struct CreatePersonalAccountOutputs;

pub const CREATE_PERSONAL_ACCOUNT_OUTPUT_ACCOUNT: &str = "account";

// The decision creates ordinal zero: `operation:` + 64 hex digits + `:0`.
// Candidate keys have exact-length allocations; the ID codec retains capacity 83.
const CREATED_ACCOUNT_ID_BYTES: usize = "operation:".len() + 64 + ":0".len();
const CREATED_ACCOUNT_KEY_BYTES: usize = "bank-account:".len() + CREATED_ACCOUNT_ID_BYTES;
const INSTITUTION_LINK_KEY_BYTES: usize = "institution-account:".len() + CREATED_ACCOUNT_ID_BYTES;
const OWNER_LINK_KEY_BYTES: usize = "personal-owner:".len() + CREATED_ACCOUNT_ID_BYTES;

// Five single-field locators retain their aspect/field text and one FieldKey each.
const CREATED_ACCOUNT_FIELD_LOCATOR_BYTES: usize = 5 * std::mem::size_of::<
    worth_foundational::facade::FieldKey,
>() + "Identity".len()
    + "AccountIdentity".len()
    + "AccountProfile".len()
    + "AccountDisplayName".len()
    + "AccountState".len()
    + "AccountingRevision".len()
    + "AccountProfile".len()
    + "Kind".len()
    + "AccountState".len()
    + "Status".len();

pub const CREATE_PERSONAL_ACCOUNT_RETAINED_BYTES: usize =
    // Created effect, uniqueness key and created reference.
    3 * CREATED_ACCOUNT_KEY_BYTES + "Account".len()
    // Each relation retains its effect/uniqueness key and created endpoint.
    + 2 * INSTITUTION_LINK_KEY_BYTES + CREATED_ACCOUNT_KEY_BYTES
    + 2 * OWNER_LINK_KEY_BYTES + CREATED_ACCOUNT_KEY_BYTES
    // Encoded ID, maximum name, kind/status; UInt64 revision has no owned allocation.
    + CREATED_ACCOUNT_FIELD_LOCATOR_BYTES + 83 + AccountName::MAX_BYTES
    + "personal".len() + "open".len()
    // Expected role plus bound role/entity/created reference.
    + 2 * CREATE_PERSONAL_ACCOUNT_OUTPUT_ACCOUNT.len() + "Account".len()
    + CREATED_ACCOUNT_KEY_BYTES;

impl ApplicationMutationOutputContract<BankSchema> for CreatePersonalAccountOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[ApplicationMutationOutputRoleDescriptor::for_entity::<
            BankSchema,
            Account,
        >(
            CREATE_PERSONAL_ACCOUNT_OUTPUT_ACCOUNT,
            ApplicationMutationOutputPosture::Create,
        )];
}

impl CreatePersonalAccountDecision {
    pub fn from_admitted_input(key: &BankIdempotencyKey, input: &CreatePersonalAccount) -> Self {
        let operation_identity = derive_identity(
            CREATE_PERSONAL_ACCOUNT_IDENTITY_DOMAIN,
            CREATE_PERSONAL_ACCOUNT_IDENTITY_RULE_VERSION,
            [
                digest_entry(
                    CREATE_PERSONAL_ACCOUNT_IDENTITY_DOMAIN,
                    "client-key",
                    client_key_identity(key),
                ),
                digest_entry(
                    CREATE_PERSONAL_ACCOUNT_IDENTITY_DOMAIN,
                    "input",
                    create_personal_account_input_identity(input),
                ),
            ],
        );
        Self {
            account: AccountId::from_operation(operation_identity, 0),
            institution: input.institution,
            owner: input.owner,
            display_name: input.display_name.clone(),
        }
    }

    pub fn into_parts(self) -> (AccountId, InstitutionId, BankPrincipalId, AccountName) {
        (
            self.account,
            self.institution,
            self.owner,
            self.display_name,
        )
    }
}

const CLIENT_KEY_DOMAIN: CanonicalBasisDomain =
    CanonicalBasisDomain::Future("worth-bank.application-mutation-client-key");
const CLIENT_KEY_RULE_VERSION: &str = "worth-bank-application-mutation-client-key-v1";
const CREATE_PERSONAL_ACCOUNT_INPUT_DOMAIN: CanonicalBasisDomain =
    CanonicalBasisDomain::Future("worth-bank.create-personal-account-input");
const CREATE_PERSONAL_ACCOUNT_INPUT_RULE_VERSION: &str =
    "worth-bank-create-personal-account-input-v1";
const CREATE_PERSONAL_ACCOUNT_IDENTITY_DOMAIN: CanonicalBasisDomain =
    CanonicalBasisDomain::Future("worth-bank.create-personal-account-identity");
const CREATE_PERSONAL_ACCOUNT_IDENTITY_RULE_VERSION: &str =
    "worth-bank-create-personal-account-identity-v1";

fn institution_scope(input: &CreatePersonalAccount) -> InstitutionId {
    input.institution
}

fn client_key_identity(key: &BankIdempotencyKey) -> [u8; 32] {
    derive_identity(
        CLIENT_KEY_DOMAIN,
        CLIENT_KEY_RULE_VERSION,
        [text_entry(CLIENT_KEY_DOMAIN, "client-key", key.as_str())],
    )
}

fn create_personal_account_input_identity(input: &CreatePersonalAccount) -> [u8; 32] {
    derive_identity(
        CREATE_PERSONAL_ACCOUNT_INPUT_DOMAIN,
        CREATE_PERSONAL_ACCOUNT_INPUT_RULE_VERSION,
        [
            unsigned_entry(
                CREATE_PERSONAL_ACCOUNT_INPUT_DOMAIN,
                "institution",
                input.institution.get(),
            ),
            unsigned_entry(
                CREATE_PERSONAL_ACCOUNT_INPUT_DOMAIN,
                "owner",
                input.owner.get(),
            ),
            text_entry(
                CREATE_PERSONAL_ACCOUNT_INPUT_DOMAIN,
                "display-name",
                input.display_name.as_str(),
            ),
        ],
    )
}

worth_query_mutation_binding!(
    pub CreatePersonalAccountMutationBinding for CreatePersonalAccount, schema BankSchema,
    identity "bank.operation.create-personal-account.mutation-binding.v1",
    input CreatePersonalAccountInputBinding,
    operation CreatePersonalAccountOperation,
    result CreatePersonalAccountResultBinding,
    idempotency BankIdempotencyKey,
        identity "bank.application-mutation-client-key.v1",
        key_identity client_key_identity,
        input_identity create_personal_account_input_identity,
    decision CreatePersonalAccountDecision,
    denial CreatePersonalAccountDenialBinding,
    handler identity "bank.operation.create-personal-account.handler.v1",
    outputs CreatePersonalAccountOutputs,
    principal BankPrincipalBinding,
        mapping ExternalPrincipalMapping,
        principal_entity Principal,
        principal_identity BankPrincipalId,
        identity_binding BankPrincipalIdBinding,
    scope Institution,
        InstitutionIdentity,
        InstitutionIdentityField,
        InstitutionId,
        ReadOnly,
        NoApplicationUnit,
    field InstitutionIdentityField::reference(),
    value institution_scope,
    candidates creates 1, deletes 0, links 2, unlinks 0, writes 5, emits 0,
    resources retained_representation_bytes CREATE_PERSONAL_ACCOUNT_RETAINED_BYTES, validator_work 8
);

fn derive_identity<const N: usize>(
    domain: CanonicalBasisDomain,
    rule_version: &'static str,
    entries: [CanonicalBasisEntry; N],
) -> [u8; 32] {
    let version = CanonicalizationRuleVersion::new(rule_version)
        .expect("the fixed bank application mutation identity rule is valid");
    let basis = prepare_canonical_basis_sequence(version, domain, entries)
        .into_result()
        .expect("bank application mutation identity fields have unique typed loci");
    let ready = canonicalization()
        .digest()
        .for_sequence(basis, CanonicalDigestAlgorithmId::sha256())
        .into_result()
        .expect("SHA-256 admits the typed bank application mutation identity basis");
    *canonicalization().digest().derive(ready).value().bytes()
}

fn text_entry(
    domain: CanonicalBasisDomain,
    locus: &'static str,
    value: &str,
) -> CanonicalBasisEntry {
    identity_entry(
        domain,
        locus,
        CanonicalBasisValue::ExactText(value.to_owned().into()),
    )
}

fn unsigned_entry(
    domain: CanonicalBasisDomain,
    locus: &'static str,
    value: u64,
) -> CanonicalBasisEntry {
    identity_entry(
        domain,
        locus,
        CanonicalBasisValue::UnsignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value: value.into(),
        },
    )
}

fn digest_entry(
    domain: CanonicalBasisDomain,
    locus: &'static str,
    value: [u8; 32],
) -> CanonicalBasisEntry {
    identity_entry(
        domain,
        locus,
        CanonicalBasisValue::BytesDigest(CanonicalDigestId::new(value)),
    )
}

fn identity_entry(
    domain: CanonicalBasisDomain,
    locus: &'static str,
    value: CanonicalBasisValue,
) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        domain,
        CanonicalBasisLocus::Named(locus.into()),
        CanonicalBasisEntryKind::Identity,
        value,
    )
}
