#!/usr/bin/env python3
"""Create an ignored local development ZIP, without install/publish/catalog edits."""
import hashlib,json,pathlib,shutil,stat,subprocess,sys,zipfile
ROOT=pathlib.Path(__file__).resolve().parents[1]
binary=pathlib.Path(sys.argv[1]).resolve() if len(sys.argv)>1 else ROOT/'.qa/fresh-target/release/converter'
assert binary.is_file()
manifest=json.loads((ROOT/'extension.json').read_text());assert manifest['artifacts']=={}
base=ROOT/'.qa/package';base.mkdir(parents=True,exist_ok=True)
package=base/'root';package.mkdir(exist_ok=True)
for directory in ['bin','resources','notices']:(package/directory).mkdir(exist_ok=True)
shutil.copy2(binary,package/'bin/converter');(package/'bin/converter').chmod(0o755)
for name in ['LICENSE','NOTICE']:shutil.copyfile(ROOT/name,package/name)
shutil.copytree(ROOT/'resources',package/'resources',dirs_exist_ok=True)
shutil.copytree(ROOT/'notices',package/'notices',dirs_exist_ok=True)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
manifest['files']={p.relative_to(package).as_posix():sha(p)for p in sorted(package.rglob('*')) if p.is_file() and p.name!='extension.json'}
(package/'extension.json').write_text(json.dumps(manifest,indent=2)+'\n')
archive=base/'converter-aarch64-apple-darwin-dev.zip'
with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED)as z:
 for p in sorted(package.rglob('*')):
  if p.is_file():
   info=zipfile.ZipInfo(p.relative_to(package).as_posix(),(2026,10,26,0,0,0));info.external_attr=(stat.S_IFREG|(0o755 if p.name=='converter' else 0o644))<<16;info.compress_type=zipfile.ZIP_DEFLATED;z.writestr(info,p.read_bytes())
# Extract the actual ZIP and verify its complete closed inventory before smoke.
extracted=base/'extracted';extracted.mkdir(exist_ok=True)
with zipfile.ZipFile(archive)as z:z.extractall(extracted)
(extracted/'bin/converter').chmod(0o755)
m=json.loads((extracted/'extension.json').read_text());actual={p.relative_to(extracted).as_posix():sha(p)for p in extracted.rglob('*') if p.is_file() and p.name!='extension.json'};assert actual==m['files']
result={'kind':'local-development-only','target':'aarch64-apple-darwin','archive':str(archive.relative_to(ROOT)),'archiveSha256':sha(archive),'executableSha256':sha(extracted/'bin/converter'),'files':actual,'artifacts':{},'installed':False,'published':False}
(ROOT/'qa/dev-package.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({k:v for k,v in result.items()if k!='files'}))
subprocess.run([sys.executable,str(ROOT/'scripts/process-qa.py'),str(extracted/'bin/converter')],check=True)
