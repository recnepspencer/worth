use worth_signal::facade::adapters::{
    SignalDeliveryStrategyIdentity, SignalInvalidationStrategyIdentity, SignalMergeStrategyIdentity,
    SignalMergeStrategyWitness,
};

fn requires_deserialize_owned<T: serde::de::DeserializeOwned>() {}

fn main() {
    requires_deserialize_owned::<SignalMergeStrategyIdentity>();
    requires_deserialize_owned::<SignalInvalidationStrategyIdentity>();
    requires_deserialize_owned::<SignalDeliveryStrategyIdentity>();
    requires_deserialize_owned::<SignalMergeStrategyWitness>();
}
