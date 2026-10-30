use worth_signal::facade::core::ActiveResourceRevalidationProof;

fn requires_deserialize_owned<T: serde::de::DeserializeOwned>() {}

fn main() {
    requires_deserialize_owned::<ActiveResourceRevalidationProof>();
}
