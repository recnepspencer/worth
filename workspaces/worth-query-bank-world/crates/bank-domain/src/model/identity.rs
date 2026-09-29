macro_rules! domain_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize)]
        pub struct $name(u64);

        impl $name {
            pub const fn new(value: u64) -> Option<Self> {
                if value == 0 {
                    None
                } else {
                    Some(Self(value))
                }
            }

            pub const fn get(self) -> u64 {
                self.0
            }

            pub fn canonical_text(self) -> String {
                format!("fixture:{}", self.0)
            }

            pub fn parse_canonical_text(value: &str) -> Option<Self> {
                let value = value.strip_prefix("fixture:")?.parse::<u64>().ok()?;
                Self::new(value)
            }
        }
    };
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize)]
enum CreatedDomainIdentity {
    Fixture(u64),
    Operation {
        idempotency_key: [u8; 32],
        ordinal: u32,
    },
}

macro_rules! created_domain_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize)]
        pub struct $name(CreatedDomainIdentity);

        impl $name {
            pub const fn new(value: u64) -> Option<Self> {
                if value == 0 {
                    None
                } else {
                    Some(Self(CreatedDomainIdentity::Fixture(value)))
                }
            }

            pub(crate) const fn from_operation(idempotency_key: [u8; 32], ordinal: u32) -> Self {
                Self(CreatedDomainIdentity::Operation {
                    idempotency_key,
                    ordinal,
                })
            }

            pub fn canonical_text(self) -> String {
                canonical_created_identity(self.0)
            }

            pub fn parse_canonical_text(value: &str) -> Option<Self> {
                parse_created_identity(value).map(Self)
            }
        }
    };
}

created_domain_id!(AccountId);
created_domain_id!(AccountAuthorizationId);
domain_id!(BankPrincipalId);
domain_id!(BankSnapshotVersion);
domain_id!(BusinessId);
domain_id!(EmployeeAssignmentId);
domain_id!(InstitutionId);
created_domain_id!(JournalEntryId);
created_domain_id!(PaymentId);
created_domain_id!(PostingId);

fn canonical_created_identity(identity: CreatedDomainIdentity) -> String {
    match identity {
        CreatedDomainIdentity::Fixture(value) => format!("fixture:{value}"),
        CreatedDomainIdentity::Operation {
            idempotency_key,
            ordinal,
        } => {
            use std::fmt::Write;

            let mut encoded = String::with_capacity(83);
            encoded.push_str("operation:");
            for byte in idempotency_key {
                write!(&mut encoded, "{byte:02x}").expect("writing to String cannot fail");
            }
            write!(&mut encoded, ":{ordinal}").expect("writing to String cannot fail");
            encoded
        }
    }
}

fn parse_created_identity(value: &str) -> Option<CreatedDomainIdentity> {
    if let Some(value) = value.strip_prefix("fixture:") {
        let value = value.parse::<u64>().ok()?;
        return (value != 0).then_some(CreatedDomainIdentity::Fixture(value));
    }
    let value = value.strip_prefix("operation:")?;
    let (key, ordinal) = value.split_once(':')?;
    if key.len() != 64 {
        return None;
    }
    let mut idempotency_key = [0_u8; 32];
    for (index, pair) in key.as_bytes().chunks_exact(2).enumerate() {
        idempotency_key[index] = (hex_nibble(pair[0])? << 4) | hex_nibble(pair[1])?;
    }
    Some(CreatedDomainIdentity::Operation {
        idempotency_key,
        ordinal: ordinal.parse().ok()?,
    })
}

const fn hex_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_created_identity_family_preserves_key_and_ordinal_exactly() {
        let first = [1_u8; 32];
        let second = [2_u8; 32];

        assert_eq!(
            JournalEntryId::from_operation(first, 0),
            JournalEntryId::from_operation(first, 0)
        );
        assert_ne!(
            JournalEntryId::from_operation(first, 0),
            JournalEntryId::from_operation(second, 0)
        );
        assert_ne!(
            PostingId::from_operation(first, 0),
            PostingId::from_operation(first, 1)
        );
        assert_ne!(
            AccountId::from_operation(first, 0),
            AccountId::new(u64::MAX).unwrap()
        );
        assert_ne!(
            PaymentId::from_operation(first, 0),
            PaymentId::new(u64::MAX).unwrap()
        );
        assert_ne!(
            AccountAuthorizationId::from_operation(first, 0),
            AccountAuthorizationId::new(u64::MAX).unwrap()
        );
    }

    #[test]
    fn created_identity_text_round_trips_without_loss() {
        let identity = PostingId::from_operation([0xab; 32], u32::MAX);
        assert_eq!(
            PostingId::parse_canonical_text(&identity.canonical_text()),
            Some(identity)
        );
    }

    #[test]
    fn fixture_identity_text_round_trips_without_crossing_zero() {
        let identity = InstitutionId::new(7).unwrap();
        assert_eq!(
            InstitutionId::parse_canonical_text(&identity.canonical_text()),
            Some(identity)
        );
        assert_eq!(InstitutionId::parse_canonical_text("fixture:0"), None);
    }
}
