use worth_signal::facade::core::DependencyChangeResourceRevalidationProof;

fn requires_deserialize_owned<T: serde::de::DeserializeOwned>() {}

fn main() {
    requires_deserialize_owned::<DependencyChangeResourceRevalidationProof>();
}
