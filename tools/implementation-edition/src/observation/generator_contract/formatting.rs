use super::{
    evaluation::Evaluation,
    values::{Fault, Value},
};

impl Evaluation<'_> {
    pub(super) fn macro_value(&mut self, invocation: &syn::Macro) -> Result<Value, Fault> {
        let literal = syn::parse2::<syn::LitStr>(invocation.tokens.clone())
            .map_err(|e| e.to_string())?
            .value();
        let mut rendered = String::new();
        let mut chars = literal.chars().peekable();
        while let Some(character) = chars.next() {
            if character == '{' {
                if chars.peek() == Some(&'{') {
                    chars.next();
                    rendered.push('{');
                } else {
                    let mut name = String::new();
                    loop {
                        match chars.next() {
                            Some('}') => break,
                            Some(c) => name.push(c),
                            None => return Err("unclosed generator format capture".into()),
                        }
                    }
                    rendered.push_str(&self.lookup(&name)?.text()?);
                }
            } else if character == '}' {
                if chars.next() != Some('}') {
                    return Err("unsupported generator format brace".into());
                }
                rendered.push('}');
            } else {
                rendered.push(character);
            }
        }
        if invocation
            .path
            .get_ident()
            .is_some_and(|name| crate::rust_identifier::semantic_name(name) == "println")
        {
            self.effects.stdout.push_str(&rendered);
            self.effects.stdout.push('\n');
            Ok(Value::Unit)
        } else {
            Ok(Value::Text(rendered))
        }
    }
}
