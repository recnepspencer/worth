use worth_relational::facade::branch::AdmittedRelationalForkSourceBasis;

fn assert_deserializable<'de, T: serde::Deserialize<'de>>() {}

fn main() {
    assert_deserializable::<AdmittedRelationalForkSourceBasis>();
}
