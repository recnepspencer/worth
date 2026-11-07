"""Resolve extra compiler aliases by declaration search, without measuring again."""
import collections,json,re,subprocess
from pathlib import Path
from tree_sitter import Language,Parser
import tree_sitter_rust
r=Path.cwd();e=r/'target/lane-evidence';p=Parser(Language(tree_sitter_rust.language()))
families=json.loads((e/'type-size-families.json').read_text());missing=[f for f in families if not f['source'] and f['head'].startswith(('worth_','bank'))]
names={f['head'].split('::')[-1] for f in missing};roots={};nodes={}
for name in ['root','query','store','bank-layout','consumer-layout']:
 file=e/('type-size-'+name)/('build/cargo-metadata.json' if name in ['root','query','store'] else 'cargo-metadata.json')
 meta=json.loads(file.read_text())
 for package in meta['packages']:
  path=Path(package['manifest_path']).parent
  if str(path).lower().startswith(str(r).lower()):roots[package['id']]=(package['name'].replace('-','_'),path/'src')
 for node in meta['resolve']['nodes']:nodes[node['id']]=node['dependencies']
selected={key for key,(crate,path) in roots.items() if crate in {f['head'].split('::')[0] for f in missing}}
pending=list(selected)
while pending:
 key=pending.pop()
 for dep in nodes.get(key,[]):
  if dep in roots and dep not in selected:selected.add(dep);pending.append(dep)
paths=sorted({str(path) for key,(crate,path) in roots.items() if key in selected and path.exists()})
pattern=r'\b(?:struct|enum|type)\s+(?:'+'|'.join(sorted(map(re.escape,names)))+r')\b'
command=['rg','--files-with-matches','--glob','*.rs',pattern,*paths]
result=subprocess.run(command,capture_output=True,text=True);assert result.returncode in [0,1],result.stderr
(e/'type-census-definition-search.json').write_text(json.dumps(dict(command=command,files=result.stdout.splitlines(),stderr=result.stderr),indent=2))
source=json.loads((e/'type-census-source.json').read_text());seen={(d['file'],d['name'],d['line']) for d in source['definitions']};added=[]
def walk(node):
 yield node
 for child in node.named_children:yield from walk(child)
for file in result.stdout.splitlines():
 path=Path(file);raw=Path('\\\\?\\'+str(path)).read_bytes();tree=p.parse(raw)
 crate=next(crate for crate,base in sorted(roots.values(),key=lambda row:-len(str(row[1]))) if path.is_relative_to(base))
 for node in walk(tree.root_node):
  if node.type not in ['struct_item','enum_item','type_item']:continue
  name=node.child_by_field_name('name')
  if not name or name.text.decode() not in names:continue
  relative=path.relative_to(r).as_posix();key=(relative,name.text.decode(),node.start_point[0]+1)
  if key in seen:continue
  seen.add(key);added.append(dict(crate=crate,name=key[1],file=relative,line=key[2],header=node.text.decode().split('{',1)[0],fields=[],function_ranks=[]))
source['definitions'].extend(added);(e/'type-census-source.json').write_text(json.dumps(source,indent=2));(e/'type-census-supplemental-definitions.json').write_text(json.dumps(added,indent=2))
print('Additional exact declarations',len(added),'from',len(result.stdout.splitlines()),'matching source files')
