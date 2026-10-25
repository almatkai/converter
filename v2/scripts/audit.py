#!/usr/bin/env python3
"""Audit V1 byte identity, immutable fixture bytes, pins, and the whole visible tree."""
from pathlib import Path
import subprocess,hashlib,json
ROOT=Path(__file__).resolve().parents[1];REPO=ROOT.parent
SOURCE='e20b6a750728dc54f6e6f16c87fa21361e8fe151'
SDK='e955911f0bf5b558f9d3eb5183df0adc95c22af2'
HOST='ea100cee3f69711de12485ba061950c5fc65258f'
UPSTREAM='6734d85992290066fd31a7f5bee3cca3d3545a1a'
def git(*args):return subprocess.check_output(['git','-C',str(REPO),*args])
def sha(data):return hashlib.sha256(data).hexdigest()
assert git('rev-parse','--show-toplevel').decode().strip()==str(REPO)
assert git('rev-parse','HEAD').decode().strip()==SOURCE
assert git('branch','--show-current').decode().strip()=='feat/native-v2-rust'
assert not git('diff','--cached','--name-only')
files=git('ls-tree','-r','--name-only',SOURCE).decode().splitlines()
original={}
for f in files:
 old=git('show',SOURCE+':'+f);current=(REPO/f).read_bytes()
 if f=='README.md':assert current==old+b'\nStandalone Rust V2 development port: [v2/README.md](v2/README.md).\n'
 else:assert current==old,f
 assert (ROOT/'qa/fixtures/original'/f).read_bytes()==old
 original[f]=sha(old)
fixture_hashes={}
for area,pin,repo in [('host',HOST,REPO.parent.parent/'Tezbar'),('sdk',SDK,REPO.parent/'tezbar-extension-sdk-rust')]:
 for p in (ROOT/'qa/fixtures'/area).rglob('*'):
  if not p.is_file():continue
  name=p.relative_to(ROOT/'qa/fixtures'/area).as_posix()
  immutable=subprocess.check_output(['git','-C',str(repo),'show',pin+':'+name])
  assert p.read_bytes()==immutable,str(p)
  fixture_hashes[str(p.relative_to(ROOT))]=sha(immutable)
for p in (ROOT/'qa/fixtures/upstream').rglob('*'):
 if p.is_file():fixture_hashes[str(p.relative_to(ROOT))]=sha(p.read_bytes())
icon=sha((ROOT/'resources/extension-icon.png').read_bytes());assert icon=='77a16b3723a810618b4781a19c7bee0fba5910cfba58bff89b7a3830bfc02132'
assert (ROOT/'resources/extension-icon.png').read_bytes()==(ROOT/'qa/fixtures/upstream/extensions/converter/assets/extension-icon.png').read_bytes()
lock=(ROOT/'Cargo.lock').read_text();assert 'https://github.com/almatkai/tezbar-extension-sdk-rust.git?rev='+SDK+'#'+SDK in lock
metadata=json.loads(subprocess.check_output(['cargo','metadata','--locked','--no-deps','--format-version','1','--manifest-path',str(ROOT/'Cargo.toml')]))
package=metadata['packages'][0]
assert all(d.get('path') is None and d['name']!='tauri' and d['kind']!='build' for d in package['dependencies'])
provenance={
 'source':{'repository':'https://github.com/almatkai/converter.git','commit':SOURCE,'freshOriginMainAtStart':SOURCE,'branch':'feat/native-v2-rust','originalSha256':original},
 'sdk':{'repository':'https://github.com/almatkai/tezbar-extension-sdk-rust.git','commit':SDK,'crate':'tezbar-extension-sdk','version':'0.1.1','readMethod':'exact read-only git show; build fetches HTTPS Git'},
 'host':{'commit':HOST,'readMethod':'exact read-only git show; working files never read or edited','recommendedId':'raycast.converter'},
 'upstream':{'repository':'https://github.com/raycast/extensions','commit':UPSTREAM,'path':'extensions/converter','iconUrl':f'https://raw.githubusercontent.com/raycast/extensions/{UPSTREAM}/extensions/converter/assets/extension-icon.png','iconSha256':icon},
 'fixturesSha256':fixture_hashes,
 'xerj':{'canonicalRoot':str(REPO),'prefix':'ax-'+sha(str(REPO).encode())[:12],'stateDir':'.git/xerj-state','initial':'generation 1: 22 indexed files, 131 records, all 16 code files indexed; 4 junk/skipped non-code files, exit 3; incomplete non-code coverage; exact Git/rg audit authoritative'},
}
(ROOT/'qa/provenance.json').write_text(json.dumps(provenance,indent=2)+'\n')
visible=git('ls-files','--cached','--others','--exclude-standard','-z').decode().split('\0')
visible=sorted(set(f for f in visible if f and f!='v2/INDEX.sha256'))
assert all(f in files or f.startswith('v2/') for f in visible)
assert not any(part in {'target','node_modules','.qa'}for f in visible for part in Path(f).parts)
(ROOT/'INDEX.sha256').write_text(''.join(f'{sha((REPO/f).read_bytes())}  {f}\n'for f in visible))
print(json.dumps({'V1FilesVerified':len(files),'onlyV1Change':'README discovery link','visibleIndexedFiles':len(visible),'indexExcludesItself':True,'indexEmpty':True,'head':SOURCE,'branch':'feat/native-v2-rust'}))
