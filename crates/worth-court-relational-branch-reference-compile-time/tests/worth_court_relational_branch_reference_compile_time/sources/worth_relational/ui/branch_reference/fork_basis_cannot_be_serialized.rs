use worth_relational::facade::branch::AdmittedRelationalForkSourceBasis;

fn assert_serializable<T: serde::Serialize>() {}

fn main() {
    assert_serializable::<AdmittedRelationalForkSourceBasis>();
}
