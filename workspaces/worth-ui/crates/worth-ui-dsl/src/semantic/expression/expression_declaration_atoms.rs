use crate::WorthUiArtifactInputBodyAtom;

use super::{WorthUiExpressionDeclaration, WorthUiExpressionRole};

impl WorthUiExpressionDeclaration {
    /// The body atoms a file-authored declaration of the same meaning
    /// lowers to, so both authoring paths share one sealing parser.
    pub(crate) fn body_atoms(&self) -> Vec<WorthUiArtifactInputBodyAtom> {
        let word = |text: &str| WorthUiArtifactInputBodyAtom::Identifier(text.to_owned());
        let mut atoms = Vec::new();
        for operand in self.operands() {
            atoms.extend([
                word("operand"),
                word(operand.name()),
                word(operand.source().clause_keyword()),
                word(operand.source().reference()),
                WorthUiArtifactInputBodyAtom::Semicolon,
            ]);
        }
        if let WorthUiExpressionRole::Derived(result) = self.role() {
            atoms.push(word("result"));
            atoms.push(match result.canonical_token() {
                "token" => WorthUiArtifactInputBodyAtom::KeywordToken,
                other => word(other),
            });
            atoms.push(WorthUiArtifactInputBodyAtom::Semicolon);
        }
        atoms.push(WorthUiArtifactInputBodyAtom::ExpressionBody(
            self.body().clone(),
        ));
        atoms
    }
}
