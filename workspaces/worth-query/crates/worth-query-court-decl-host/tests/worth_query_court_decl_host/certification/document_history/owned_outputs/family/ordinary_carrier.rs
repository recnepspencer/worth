//! Existing authored-carrier roles make the independent support producer ordinary.
macro_rules! ordinary_carrier_contract {
    (OrdinaryInputInitial) => { ordinary_carrier_contract!(@declare); };
    (OrdinaryInputPreserve) => { ordinary_carrier_contract!(@declare); };
    (@declare) => {
        pub struct Carrier;
        impl WorthQueryApplicationOutputRole for Carrier {
            type Schema = Schema; type Contract = Outputs; type Entity = SourceEntity;
            type Action = WorthQueryPreserveOutput; type Cardinality = WorthQueryExactlyOneOutput;
            const NAME: &'static str = "authored-carrier";
        }
    };
    ($other:ident) => {};
}
macro_rules! producer_role_descriptors {
    (OrdinaryInputInitial) => {
        &[Output::DESCRIPTOR, Carrier::DESCRIPTOR]
    };
    (OrdinaryInputPreserve) => {
        &[Output::DESCRIPTOR, Carrier::DESCRIPTOR]
    };
    ($other:ident) => {
        &[<Output as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR]
    };
}
macro_rules! ordinary_carrier_output {
    (OrdinaryInputInitial,$writer:ident,$source:ident) => {
        $writer.preserve_output::<Carrier>(&$source)?;
    };
    (OrdinaryInputPreserve,$writer:ident,$source:ident) => {
        $writer.preserve_output::<Carrier>(&$source)?;
    };
    ($other:ident,$writer:ident,$source:ident) => {};
}
