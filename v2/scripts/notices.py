"""Copy authentic license/notice files from the actual locked packages, never infer grants."""
import pathlib,json,subprocess,hashlib,shutil
ROOT=pathlib.Path(__file__).resolve().parents[1]
out=ROOT/'notices';out.mkdir(exist_ok=True)
records=[]
def copy(src,dest,info):
 dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(src,dest)
 records.append({**info,'path':str(dest.relative_to(ROOT)),'sha256':hashlib.sha256(dest.read_bytes()).hexdigest()})
for source,name in [('qa/fixtures/original/LICENSE','converter-original-LICENSE'),('qa/fixtures/upstream/LICENSE','raycast-LICENSE'),('qa/fixtures/sdk/LICENSE','sdk-LICENSE'),('qa/fixtures/sdk/NOTICE','sdk-NOTICE'),('qa/fixtures/host/LICENSE','host-fixtures-LICENSE')]:
 copy(ROOT/source,out/name,{'origin':source})
# Own license has literal escaped line breaks. The readable package license changes
# layout only; raw evidence is retained exactly above.
(ROOT/'LICENSE').write_text((ROOT/'qa/fixtures/original/LICENSE').read_text().replace('\\n','\n'))
meta=json.loads(subprocess.check_output(['cargo','metadata','--locked','--format-version','1','--manifest-path',str(ROOT/'Cargo.toml')]))
for p in meta['packages']:
 if p['name']=='tezbar-converter':continue
 directory=pathlib.Path(p['manifest_path']).parent
 names=[]
 for f in directory.iterdir():
  if f.name.lower().startswith(('license','copying','notice','unlicense')):
   names.extend([f] if f.is_file() else [p for p in f.rglob('*') if p.is_file()])
 info={'ecosystem':'cargo','package':p['name'],'version':p['version'],'source':p['source'],'declaredLicense':p['license']}
 if not names:records.append({**info,'missingLicenseFile':True})
 for f in names:copy(f,out/'cargo'/(p['name']+'-'+p['version'])/f.relative_to(directory),info)
for area in ['v1','schema-tools']:
 base=ROOT/'.qa'/area/'node_modules'
 lock=json.loads((ROOT/'.qa'/area/'package-lock.json').read_text())
 for key in sorted(lock['packages']):
  if not key.startswith('node_modules/'):continue
  pkgfile=ROOT/'.qa'/area/key/'package.json'
  if not pkgfile.exists():continue
  pkg=json.loads(pkgfile.read_text());directory=pkgfile.parent
  if not pkg.get('name'):continue
  info={'ecosystem':'npm','toolchain':area,'package':pkg['name'],'version':pkg.get('version'),'declaredLicense':pkg.get('license')}
  names=[f for f in directory.iterdir() if f.is_file() and f.name.lower().startswith(('license','copying','notice','unlicense'))]
  if not names:records.append({**info,'missingLicenseFile':True})
  for f in names:copy(f,out/'npm'/(pkg['name'].replace('/','__')+'-'+pkg.get('version','unknown'))/f.name,info)
(ROOT/'qa/license-evidence.json').write_text(json.dumps(records,indent=2)+'\n')
print(json.dumps({'evidenceEntries':len(records),'missingFiles':sum(bool(r.get('missingLicenseFile'))for r in records)}))
