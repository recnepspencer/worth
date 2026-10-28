from pathlib import Path
import re, json, sys, subprocess, tarfile, io
from tree_sitter import Language, Parser
import tree_sitter_rust
p=Parser(Language(tree_sitter_rust.language()))
revision=sys.argv[2] if len(sys.argv)>2 else None
sources=None
if revision:
 archive=subprocess.check_output(['git','archive','--format=tar',revision,'crates/worth-relational/src'])
 with tarfile.open(fileobj=io.BytesIO(archive)) as entries:
  sources={entry.name:entries.extractfile(entry).read() for entry in entries if entry.isfile()}
def source_key(f):return f.resolve().relative_to(Path.cwd()).as_posix()
def exists(f):return source_key(f) in sources if sources is not None else f.exists()
def read_source(f):return sources[source_key(f)] if sources is not None else f.read_bytes()
seen=set(); sites=[]
def visit_file(f):
 f=f.resolve()
 if f in seen:return
 seen.add(f)
 data=read_source(f); root=p.parse(data).root_node
 base=f.parent if f.name in ('mod.rs','lib.rs') else f.parent/f.stem
 visit(root,f,base,data)
def visit(node,f,base,data):
 attrs=[]
 for child in node.named_children:
  if child.type=='attribute_item': attrs.append(child.text.decode());continue
  if child.type in ('line_comment','block_comment'):continue
  skip=any(re.search(r'cfg\s*\(\s*test\s*\)|#\[\s*(?:test|bench)\s*\]',a) for a in attrs)
  path=next((re.search(r'path\s*=\s*"([^"]+)"',a).group(1) for a in attrs if re.search(r'path\s*=\s*"([^"]+)"',a)),None)
  attrs=[]
  if skip:continue
  if child.type=='mod_item':
   name=child.child_by_field_name('name').text.decode();body=child.child_by_field_name('body')
   if body:visit(body,f,base/name,data)
   else:
    candidates=[f.parent/path] if path else [base/(name+'.rs'),base/name/'mod.rs']
    for c in candidates:
     if exists(c):visit_file(c);break
    else:print('MISSING',f,name,candidates,file=sys.stderr)
   continue
  if child.type=='call_expression':
   fun=child.child_by_field_name('function')
   if fun and fun.type=='field_expression' and fun.child_by_field_name('field').text in (b'expect',b'unwrap'):
    sites.append({'path':f.relative_to(Path.cwd()).as_posix(),'line':data[:child.start_byte].count(b'\n')+1,'kind':fun.child_by_field_name('field').text.decode(),'text':child.text.decode(),'start':child.start_byte,'end':child.end_byte})
  if child.type=='macro_invocation' and child.child_by_field_name('macro').text in (b'panic',b'unreachable',b'assert',b'assert_eq',b'assert_ne',b'debug_assert',b'debug_assert_eq',b'debug_assert_ne'):
   sites.append({'path':f.relative_to(Path.cwd()).as_posix(),'line':data[:child.start_byte].count(b'\n')+1,'kind':child.child_by_field_name('macro').text.decode(),'text':child.text.decode(),'start':child.start_byte,'end':child.end_byte})
  if child.type=='macro_invocation':
   # Rust expressions inside macro arguments remain token trees in this parser.
   # Count executable spellings there without matching strings or comments.
   def token_sites(tree):
    children=[n for n in tree.children if n.type not in ('line_comment','block_comment')]
    for i,n in enumerate(children):
     kind=None
     if n.type=='identifier' and n.text in (b'expect',b'unwrap') and i and children[i-1].text==b'.' and i+1<len(children) and children[i+1].type=='token_tree':kind=n.text.decode()
     if n.type=='identifier' and n.text in (b'panic',b'unreachable',b'assert',b'assert_eq',b'assert_ne',b'debug_assert',b'debug_assert_eq',b'debug_assert_ne') and i+2<len(children) and children[i+1].text==b'!' and children[i+2].type=='token_tree':kind=n.text.decode()
     if kind:
      end=children[i+1 if kind in ('expect','unwrap') else i+2].end_byte
      sites.append({'path':f.relative_to(Path.cwd()).as_posix(),'line':data[:n.start_byte].count(b'\n')+1,'kind':kind,'text':data[n.start_byte:end].decode(),'start':n.start_byte,'end':end,'macro_argument':True})
     if n.type=='token_tree':token_sites(n)
   for token in child.named_children:
    if token.type=='token_tree':token_sites(token)
  visit(child,f,base,data)
visit_file(Path('crates/worth-relational/src/lib.rs'))
Path(sys.argv[1]).write_text(json.dumps(sites,indent=2))
print(len(sites),'sites in',len(seen),'production modules')
for s in sites: print(f"{s['path'].removeprefix('crates/worth-relational/src/')}:{s['line']}: {s['kind']} {s['text'][-160:].replace(chr(10),' ')}")
