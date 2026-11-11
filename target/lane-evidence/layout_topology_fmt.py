import subprocess,sys
from pathlib import Path
r=Path.cwd();e=r/'target/lane-evidence';groups=[('Cargo.toml',['worth-relational','worth-runtime-bridge','worth-signal','worth-court-runtime-bridge']),('workspaces/worth-query/Cargo.toml',['worth-query-execution','worth-query-publication','worth-query']),('workspaces/worth-store/Cargo.toml',['worth-store'])]
for manifest,packages in groups:
 cmd=['cargo','fmt','--manifest-path',manifest,*[x for p in packages for x in ['-p',p]]]
 if subprocess.call(cmd):sys.exit(1)
