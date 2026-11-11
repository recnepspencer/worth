from pathlib import Path
import json,re,sys
sys.path.insert(0,'scripts/quality')
from rust_function_scrutiny import sanitize_rust,signature_terminator,matching_delimiter
E=Path('target/lane-evidence');R=Path('crates/worth-relational/src')
assert 'APPROVE' in (E/'review-root-source-verdict.md').read_text()
D=Path('crates/worth-court-relational-certification/tests/worth_court_relational_certification/relational/durable_publication')
cases=[]
for p in sorted((E/'pa-root-public-sources').glob('*.rs')):
 assert not (D/p.name).exists(),p.name
 s=p.read_text();(D/p.name).write_text(s)
 cases+=re.findall(r'#\[test\]\s*fn (\w+)',s)
 pmod=D/'mod.rs';pmod.write_text(pmod.read_text()+f'\nmod {p.stem};\n')
assert len(cases)==5,cases
(E/'pa-root-public-cases.json').write_text(json.dumps(cases,indent=2)+'\n')
p=R/'durability/log/native_checkpoint_codec/tests.rs';p.write_text((E/'pa-root-codec-candidate.rs').read_text())
p=R/'durability/authority/runtime_rebuild/checkpoint_restore_tests.rs';s=p.read_text();c=sanitize_rust(s)
n='recovered_main_and_sibling_catalogs_reuse_exact_canonical_payloads';m=re.search(r'\bfn\s+'+n+r'\s*\(',c)
a=s.rfind('#[test]',0,m.start());b=matching_delimiter(c,signature_terminator(c,m.end()-1),'{','}')+1
candidate=(E/'pa-root-catalog-candidate.rs').read_text().split('#[test]',1)[1]
p.write_text(s[:a]+'#[test]'+candidate.rstrip()+s[b:])
(E/'pa-root-native-cases.json').write_text(json.dumps([n,'member_directory_preserves_divergent_sibling_roots_and_pinned_reader'],indent=2)+'\n')
print('Mounted5public and2Memory owners; other original roots remain until qualification')
