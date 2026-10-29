use worth_foundational::AspectValue;
use worth_store::physical_runtime::PhysicalWorkSemanticBasis;

fn accept_basis(_basis: PhysicalWorkSemanticBasis) {}

fn reject_foundational_value(value: AspectValue) {
    accept_basis(value);
}

fn reject_signal_aspect(value: worth_signal::facade::Aspect) {
    accept_basis(value);
}

fn reject_json_value(value: serde_json::Value) {
    accept_basis(value);
}

fn main() {}
