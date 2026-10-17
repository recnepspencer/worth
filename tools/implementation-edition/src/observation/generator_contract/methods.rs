use super::{
    evaluation::Evaluation,
    numeric::{self, Integer},
    values::{Fault, Value},
};
use crate::rust_identifier::semantic_name;
use serde_json::json;
use std::{
    fs,
    path::{Component, Path},
    process::Command,
};

impl Evaluation<'_> {
    pub(super) fn method(&mut self, call: &syn::ExprMethodCall) -> Result<Value, Fault> {
        let method = semantic_name(&call.method);
        let receiver = self.expr(&call.receiver)?;
        let mut args = Vec::new();
        for argument in &call.args {
            args.push(self.expr(argument)?);
        }
        match (method.as_str(), receiver, args.as_slice()) {
            ("unwrap", Value::Optional(Some(value)) | Value::Result(Ok(value)), []) => Ok(*value),
            ("ok", Value::Result(value), []) => Ok(Value::Optional(value.ok())),
            ("trim", Value::Text(text), []) => Ok(Value::Text(text.trim().to_owned())),
            ("parse", Value::Text(text), []) => {
                let parsed = match numeric::parse_type(call)? {
                    Integer::U8 => text.parse::<u8>().map(i128::from),
                    Integer::U32 => text.parse::<u32>().map(i128::from),
                };
                Ok(Value::Result(
                    parsed
                        .map(|i| Box::new(Value::Int(i)))
                        .map_err(|e| e.to_string()),
                ))
            }
            ("replace", Value::Text(text), [from, to]) => {
                Ok(Value::Text(text.replace(&from.text()?, &to.text()?)))
            }
            ("starts_with", Value::Text(text), [prefix]) => {
                Ok(Value::Bool(text.starts_with(&prefix.text()?)))
            }
            ("split", Value::Text(text), [separator]) => Ok(Value::Pieces(
                text.split(&separator.text()?).map(str::to_owned).collect(),
                0,
            )),
            ("next", Value::Pieces(parts, index), []) => {
                let syn::Expr::Path(path) = call.receiver.as_ref() else {
                    return Err("generator iterator requires a local receiver".into());
                };
                let name = path
                    .path
                    .get_ident()
                    .ok_or("generator iterator receiver is not local")?;
                let name = semantic_name(name);
                let variables = self
                    .frames
                    .last_mut()
                    .unwrap()
                    .iter_mut()
                    .rev()
                    .find(|scope| scope.contains_key(&name))
                    .ok_or("generator iterator local missing")?;
                variables.insert(name, Value::Pieces(parts.clone(), index + 1));
                Ok(Value::Optional(
                    parts.get(index).map(|s| Box::new(Value::Text(s.clone()))),
                ))
            }
            ("join", Value::OutputPath(root), [Value::Text(child)]) => {
                let child = Path::new(child);
                if child.as_os_str().is_empty()
                    || !child
                        .components()
                        .all(|c| matches!(c, Component::Normal(_)))
                {
                    return Err("generator output join escapes its owner".into());
                }
                let path = root.join(child);
                if path.exists()
                    && !fs::canonicalize(&path)
                        .map_err(|e| e.to_string())?
                        .starts_with(self.output)
                {
                    return Err("generator output path escapes its owner".into());
                }
                Ok(Value::OutputPath(path))
            }
            ("arg", Value::Command(path, mut arguments), [Value::Text(argument)]) => {
                arguments.push(argument.clone());
                Ok(Value::Command(path, arguments))
            }
            ("output", Value::Command(path, arguments), []) => {
                if path != self.compiler || arguments != ["--version"] {
                    return Err("generator compiler query requires selected rustc --version".into());
                }
                let output = Command::new(&path)
                    .args(&arguments)
                    .output()
                    .map_err(|e| e.to_string())?;
                if !output.status.success() {
                    return Err("generator selected compiler query failed".into());
                }
                self.effects
                    .queries
                    .push(json!({"operation":"selected-rustc-version", "stdout":output.stdout}));
                Ok(Value::Result(Ok(Box::new(Value::Output(output.stdout)))))
            }
            _ => Err(format!("generator method effect requires qualification: {method}").into()),
        }
    }
}
