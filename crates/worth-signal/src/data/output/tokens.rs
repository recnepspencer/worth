use std::hash::{Hash, Hasher};

mod retained_charge;

use serde::{Deserialize, Serialize};

use crate::data::core_profile::StableHashValue;

macro_rules! define_string_token {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(from = "String", into = "String")]
        pub struct $name {
            value: String,
            stable_hash: StableHashValue,
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new(String::new())
            }
        }

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                let value = value.into();
                Self {
                    stable_hash: stable_string_hash(&value),
                    value,
                }
            }

            pub fn as_str(&self) -> &str {
                &self.value
            }

            pub fn stable_hash(&self) -> StableHashValue {
                self.stable_hash
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self::new(value)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self::new(value)
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.value
            }
        }

        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                self.value == other.value
            }
        }

        impl Eq for $name {}

        impl PartialOrd for $name {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for $name {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.value.cmp(&other.value)
            }
        }

        impl Hash for $name {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.value.hash(state);
            }
        }
    };
}

define_string_token!(
    /// Host-supplied stable identity token for one evaluated output artifact.
    OutputIdentity
);

define_string_token!(
    /// Host-supplied continuity token for lineage preservation when output
    /// identity is too coarse or intentionally absent.
    ///
    /// This is domain-agnostic. Host code can use it to express â€œthis result
    /// should continue the same artifact lineageâ€ without forcing that meaning
    /// onto `OutputIdentity`.
    ArtifactContinuityToken
);

define_string_token!(
    /// Family namespace for keyed computations.
    ComputationFamily
);

define_string_token!(
    /// Stable key for one keyed computation inside a family.
    ComputationKey
);

define_string_token!(
    /// Stable host-provided structural memoization key.
    StructuralMemoKey
);

fn stable_string_hash(value: &str) -> StableHashValue {
    #[cfg(feature = "profile-compact")]
    let mut hash: StableHashValue = 0xcbf29ce484222325_u64;
    #[cfg(any(feature = "profile-standard", feature = "profile-extended"))]
    let mut hash: StableHashValue = 0x6c62272e07bb014262b821756295c58d_u128;
    for byte in value.as_bytes() {
        hash ^= *byte as StableHashValue;
        #[cfg(feature = "profile-compact")]
        {
            hash = hash.wrapping_mul(0x100000001b3_u64);
        }
        #[cfg(any(feature = "profile-standard", feature = "profile-extended"))]
        {
            hash = hash.wrapping_mul(0x0000000001000000000000000000013B_u128);
        }
    }
    hash
}
