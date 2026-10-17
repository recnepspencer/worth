use super::{
    structure::{Structure, Work},
    Failure,
};
use proc_macro::{Span, TokenStream, TokenTree};
pub(super) enum Object {
    Text(String),
    Stream(TokenStream),
    Token(TokenTree),
    Span(Span),
    Iterator(proc_macro::token_stream::IntoIter),
    LexError(proc_macro::LexError),
}
#[derive(Default)]
pub(super) struct Objects {
    values: Vec<(Object, Structure)>,
    payload_bytes: usize,
    work: Work,
    pub(super) observations: Vec<super::SpanRead>,
}
impl Objects {
    pub(super) fn observe(&mut self, read: super::SpanRead) -> Result<(), Failure> {
        if self.observations.len() >= 100_000 {
            return Err(Failure::WorkLimit);
        }
        let bytes = match &read {
            super::SpanRead::SourceText(v) => v.as_ref().map_or(0, String::len),
            super::SpanRead::File(v) | super::SpanRead::Debug(v) => v.len(),
            super::SpanRead::LocalFile(v) => v.as_ref().map_or(0, |p| p.as_os_str().len()),
            super::SpanRead::Line(_) | super::SpanRead::Column(_) => 16,
        };
        self.reserve_payload(bytes)?;
        self.observations.push(read);
        Ok(())
    }
    fn reserve_payload(&mut self, bytes: usize) -> Result<(), Failure> {
        self.payload_bytes = self
            .payload_bytes
            .checked_add(bytes)
            .ok_or(Failure::WorkLimit)?;
        if self.payload_bytes > 32 * 1024 * 1024 {
            return Err(Failure::WorkLimit);
        }
        Ok(())
    }
    pub(super) fn structure(&self, handle: u32) -> Result<Structure, Failure> {
        self.get(handle)?;
        Ok(self.values[(handle - 1) as usize].1)
    }
    pub(super) fn charge(&mut self, handle: u32) -> Result<(), Failure> {
        self.work.charge(self.structure(handle)?.nodes)
    }
    pub(super) fn parsed_text(&self, handle: u32) -> Result<String, Failure> {
        let text = self.text(handle)?;
        // A native parse is host work outside Wasmi fuel. Limit before calling
        // it: 256 UTF-8 bytes can contain at most 128 nested delimiter pairs.
        if text.len() > 256 {
            return Err(Failure::WorkLimit);
        }
        Ok(text)
    }
    pub(super) fn push(&mut self, value: Object) -> Result<u32, Failure> {
        let structure = match &value {
            Object::Stream(v) => self.work.measure(v.clone())?,
            Object::Token(v) => self.work.measure(TokenStream::from(v.clone()))?,
            Object::Iterator(_) => return Err("iterator requires carried structure".into()),
            Object::Text(v) => Structure {
                bytes: v.len(),
                ..Structure::default()
            },
            Object::Span(_) | Object::LexError(_) => Structure {
                bytes: 128,
                ..Structure::default()
            },
        };
        self.push_carried(value, structure)
    }
    pub(super) fn push_carried(
        &mut self,
        value: Object,
        structure: Structure,
    ) -> Result<u32, Failure> {
        structure.checked()?;
        self.work.charge(1)?;
        if self.values.len() >= 200_000 {
            return Err(Failure::WorkLimit);
        }
        self.reserve_payload(structure.bytes)?;
        self.values.push((value, structure));
        Ok(self.values.len() as u32)
    }
    pub(super) fn get(&self, handle: u32) -> Result<&Object, Failure> {
        handle
            .checked_sub(1)
            .and_then(|i| self.values.get(i as usize))
            .map(|(value, _)| value)
            .ok_or_else(|| "invalid invocation-scoped macro handle".into())
    }
    pub(super) fn get_mut(&mut self, handle: u32) -> Result<&mut Object, Failure> {
        handle
            .checked_sub(1)
            .and_then(|i| self.values.get_mut(i as usize))
            .map(|(value, _)| value)
            .ok_or_else(|| "invalid invocation-scoped macro handle".into())
    }
    pub(super) fn text(&self, h: u32) -> Result<String, Failure> {
        match self.get(h)? {
            Object::Text(s) => Ok(s.clone()),
            _ => Err("macro handle is not text".into()),
        }
    }
    pub(super) fn stream(&self, h: u32) -> Result<TokenStream, Failure> {
        match self.get(h)? {
            Object::Stream(s) => Ok(s.clone()),
            _ => Err("macro handle is not a token stream".into()),
        }
    }
    pub(super) fn span(&self, h: u32) -> Result<Span, Failure> {
        match self.get(h)? {
            Object::Span(s) => Ok(*s),
            _ => Err("macro handle is not a span".into()),
        }
    }
    pub(super) fn token(&self, h: u32) -> Result<TokenTree, Failure> {
        match self.get(h)? {
            Object::Token(s) => Ok(s.clone()),
            _ => Err("macro handle is not a token".into()),
        }
    }
}
