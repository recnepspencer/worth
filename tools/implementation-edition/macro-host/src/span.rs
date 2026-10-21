use super::objects::{Object, Objects};
use proc_macro::Span;
impl Objects {
    pub(super) fn span_operation(
        &mut self,
        op: u32,
        a: u32,
        b: u32,
    ) -> Result<u32, super::Failure> {
        let object = match op {
            10 => Object::Span(Span::call_site()),
            11 => Object::Span(Span::mixed_site()),
            12 => Object::Span(self.span(a)?.resolved_at(self.span(b)?)),
            13 => Object::Span(self.span(a)?.located_at(self.span(b)?)),
            14 => {
                let Some(span) = self.span(a)?.join(self.span(b)?) else {
                    return Ok(0);
                };
                Object::Span(span)
            }
            15 => {
                let text = self.span(a)?.source_text();
                self.observe(super::SpanRead::SourceText(text.clone()))?;
                let Some(text) = text else { return Ok(0) };
                Object::Text(text)
            }
            16 => Object::Span(self.span(a)?.start()),
            17 => Object::Span(self.span(a)?.end()),
            18 => {
                let line = self.span(a)?.line();
                self.observe(super::SpanRead::Line(line))?;
                return line
                    .try_into()
                    .map_err(|_| "span line exceeds protocol".into());
            }
            19 => {
                let column = self.span(a)?.column();
                self.observe(super::SpanRead::Column(column))?;
                return column
                    .try_into()
                    .map_err(|_| "span column exceeds protocol".into());
            }
            20 => {
                let file = self.span(a)?.file();
                self.observe(super::SpanRead::File(file.clone()))?;
                Object::Text(file)
            }
            21 => {
                let path = self.span(a)?.local_file();
                self.observe(super::SpanRead::LocalFile(path.clone()))?;
                let Some(path) = path else { return Ok(0) };
                Object::Text(path.to_str().ok_or("non-UTF8 span path")?.to_owned())
            }
            22 => {
                let debug = format!("{:?}", self.span(a)?);
                self.observe(super::SpanRead::Debug(debug.clone()))?;
                Object::Text(debug)
            }
            _ => return Err("unknown span operation".into()),
        };
        self.push(object)
    }
}
