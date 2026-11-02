//! Pure canonical equality, independent of projected rows or runtime authority.
use super::*;
use crate::application_query::ApplicationQueryResultFieldPresence as Presence;

fn contract(
    names: [&str; 5],
    family: ScalarAspectType,
    presence: Presence,
) -> ApplicationQueryResultFieldSlotContract<'_> {
    ApplicationQueryResultFieldSlotContract {
        entity: names[0],
        aspect: names[1],
        field: names[2],
        output_name: names[3],
        value_type: WorthQueryPortableTypeIdentity::from_untrusted(names[4].into()),
        scalar_family: family,
        presence,
    }
}
#[test]
fn borrowed_field_contract_match_is_owned_equality() {
    let id = WorthQueryPortableTypeIdentity::declared;
    let names = ["entity", "aspect", "field", "output", "binding"];
    let base = ApplicationQueryResultSlotKey::field(
        id("query"),
        id("slot"),
        contract(names, ScalarAspectType::String, Presence::Required),
    );
    for axis in 0..10 {
        let mut changed = names;
        if axis < 5 {
            changed[axis] = "different";
        }
        let query = if axis == 5 { "different" } else { "query" };
        let slot = if axis == 6 { "different" } else { "slot" };
        let family = if axis == 7 {
            ScalarAspectType::Bool
        } else {
            ScalarAspectType::String
        };
        let presence = match axis {
            8 => Presence::Optional,
            9 => Presence::OptionalAspect,
            _ => Presence::Required,
        };
        let owned = ApplicationQueryResultSlotKey::field(
            id(query),
            id(slot),
            contract(changed, family, presence),
        );
        assert_eq!(
            base.matches_field(id(query), id(slot), contract(changed, family, presence)),
            base == owned,
            "axis {axis}"
        );
        assert_ne!(base, owned);
    }
    assert!(base.matches_field(
        id("query"),
        id("slot"),
        contract(names, ScalarAspectType::String, Presence::Required)
    ));
    let relation = ApplicationQueryResultSlotKey::orbit(
        id("query"),
        id("slot"),
        "relation",
        "from",
        "to",
        "output",
    );
    assert!(!relation.matches_field(
        id("query"),
        id("slot"),
        contract(names, ScalarAspectType::String, Presence::Required)
    ));
}
