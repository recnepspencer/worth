from pathlib import Path
import hashlib,json,subprocess
root=Path('target/lane-evidence')
head=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()
snapshot=json.loads((root/'funding-landing-source-state.json').read_text())
paths=list(snapshot['files'])
proc=subprocess.run(['git','cat-file','--batch'],input=('\n'.join(head+':'+p for p in paths)+'\n').encode(),stdout=subprocess.PIPE,check=True)
raw=proc.stdout
position=0
blobs={}
for path in paths:
    end=raw.index(b'\n',position)
    sha,kind,length=raw[position:end].split()
    assert kind==b'blob',(path,kind)
    size=int(length)
    data=raw[end+1:end+1+size]
    position=end+2+size
    actual=Path(path).read_bytes()
    assert hashlib.sha256(actual).hexdigest()==snapshot['files'][path],path
    assert data.replace(b'\r\n',b'\n')==actual.replace(b'\r\n',b'\n'),path
    blobs[path]=sha.decode()
assert position==len(raw)
message=subprocess.check_output(['git','log','-1','--format=%B'],text=True)
assert message.rstrip().endswith('Co-Authored-By: GPT-6.1 Sol <noreply@openai.com>')
(root/'funding-landing-commit-identity.json').write_text(json.dumps({'funding_commit':head,'review_parent':snapshot['review_base_head'],'review_manifest_sha256':hashlib.sha256((root/'funding-landing-source-state.json').read_bytes()).hexdigest(),'verified_source_config_files':len(paths),'git_tree_blobs':blobs,'normalization':'Only Git CRLF to LF normalization; raw working bytes still equal every frozen61 hash'},indent=2)+'\n',encoding='utf-8')
print('All61 frozen source/config hashes match committed tree at',head)
print('Review manifest SHA256:',hashlib.sha256((root/'funding-landing-source-state.json').read_bytes()).hexdigest())
print('Commit trailer verified.')
