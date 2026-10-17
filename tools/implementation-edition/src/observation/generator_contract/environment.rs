use super::{
    evaluation::Evaluation,
    values::{Fault, Value},
};
use serde_json::json;
use std::fs;

impl Evaluation<'_> {
    pub(super) fn environment_value(&mut self, name: &str) -> Result<Value, Fault> {
        let actual = self
            .environment
            .get(name)
            .cloned()
            .ok_or_else(|| format!("generator Cargo environment missing: {name}"))?;
        match name {
            "OUT_DIR" => {
                let path = fs::canonicalize(actual).map_err(|e| e.to_string())?;
                if path != self.output {
                    return Err("generator output context differs".into());
                }
                self.effects
                    .environment
                    .insert(name.into(), json!({"role":"output-path"}));
                Ok(Value::OutputPath(path))
            }
            "RUSTC" => {
                let path = fs::canonicalize(actual).map_err(|e| e.to_string())?;
                if path != self.compiler {
                    return Err("generator compiler query differs from selected compiler".into());
                }
                self.effects
                    .environment
                    .insert(name.into(), json!({"role":"selected-compiler"}));
                Ok(Value::Toolchain(path))
            }
            "TARGET" | "HOST" | "CARGO_PKG_VERSION_PATCH" => {
                let text = actual
                    .into_string()
                    .map_err(|_| "non-Unicode generator environment")?;
                self.effects
                    .environment
                    .insert(name.into(), json!({"role":"semantic","value":text}));
                Ok(Value::Text(text))
            }
            _ => Err("generator environment effect requires qualification".into()),
        }
    }
}
