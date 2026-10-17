use worth_execution::ChargedBytes;

#[derive(ChargedBytes)]
struct Unit;
#[derive(ChargedBytes)]
struct Tuple(String, Box<[Option<String>; 2]>);
#[derive(ChargedBytes)]
enum Resident {
    Unit,
    Tuple(String, Vec<Tuple>),
    Named {
        skipped: Option<Box<String>>,
        flag: bool,
    },
}
#[derive(ChargedBytes)]
enum Never {}

mod public_contract {
    pub use worth_execution::ChargedBytes;
}
#[derive(public_contract::ChargedBytes)]
#[charged_bytes(crate = "public_contract")]
struct OpaqueOwnerValue {
    owner: Opaque,
}
// Shared/opaque policy remains here, rather than being inferred by the derive.
struct Opaque(Vec<u8>);
impl ChargedBytes for Opaque {
    fn additional_charged_bytes(&self) -> u64 {
        self.0.capacity() as u64
    }
}
#[derive(serde::Serialize, ChargedBytes)]
struct WithSkippedStorage {
    number: u64,
    #[serde(skip)]
    resident: Vec<String>,
}
fn main() {
    assert_eq!(Unit.additional_charged_bytes(), 0);
    assert_eq!(Resident::Unit.additional_charged_bytes(), 0);
    let _ = Resident::Tuple(String::new(), vec![]).additional_charged_bytes();
    let _ = Resident::Named {
        skipped: None,
        flag: true,
    }
    .additional_charged_bytes();
    let _ = OpaqueOwnerValue {
        owner: Opaque(vec![1, 2]),
    }
    .additional_charged_bytes();
    let retained = WithSkippedStorage {
        number: 1,
        resident: vec![String::from("resident")],
    };
    assert!(retained.additional_charged_bytes() >= 8);
}
