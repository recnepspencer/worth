use worth_signal::facade::core::TerminalStateResourceRevalidationProof;

fn requires_deserialize_owned<T: serde::de::DeserializeOwned>() {}

fn main() {
    requires_deserialize_owned::<TerminalStateResourceRevalidationProof>();
}
