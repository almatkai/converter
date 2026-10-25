// QA only: execute exact pinned TS/hooks in a deny-by-default VM. TypeScript
// and Node Buffer are the actual root-lock toolchain; never shipped in product.
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const assert = require('node:assert/strict');
const root = path.resolve(__dirname, '..');
const ts = require(path.join(root,'.qa/v1/node_modules/typescript'));
const sourceRoot = path.join(root,'qa/fixtures/original');
const units = {bits:0,Bytes:3,KB:13,MB:23,GB:33,TB:43,PB:53,EB:63};
const codes = ['native','unicode','base64','utf8','ascii','hex','decimal','url','entity'];
function sandbox(prefs, clipboard) {
  let slots=[], cursor=0, effects=[], calledEffects=new Set(), focus='';
  const react={
    useState(initial) { const i=cursor++; if(!(i in slots)) slots[i]=initial;return [slots[i],v=>{slots[i]=typeof v==='function'?v(slots[i]):v;}]; },
    useReducer(reducer, initial, init) { const [v,set]=react.useState(init?init(initial):initial);return [v,a=>set(old=>reducer(old,a))]; },
    useRef(initial) { const [ref]=react.useState({current:initial});return ref; },
    useEffect(fn) { const i=cursor++;if(!calledEffects.has(i)){calledEffects.add(i);effects.push(fn);} },
    useMemo(fn) { const [value]=react.useState(fn());return value; },
    createElement(type, props, ...children) { return {type,props:{...props,children}}; },
    Fragment:'Fragment',
  };
  const deny=()=>{throw new Error('QA producer denied');};
  const component = name => new Proxy(Object.assign(function(){},{_qaType:name}),{get:(t,k)=>k in t?t[k]:component(name+'.'+String(k))});
  const api={getPreferenceValues:()=>prefs,Clipboard:{readText:()=>Promise.resolve(clipboard),copy:deny,paste:deny},Form:component('Form'),Action:component('Action'),ActionPanel:component('ActionPanel'),Icon:component('Icon'),openCommandPreferences:deny,openExtensionPreferences:deny};
  const cache=new Map();
  function load(file) {
    if(cache.has(file))return cache.get(file).exports;
    assert(file.startsWith(sourceRoot+path.sep));
    const text=fs.readFileSync(file,'utf8');
    const compiled=ts.transpileModule(text,{compilerOptions:{module:ts.ModuleKind.CommonJS,target:ts.ScriptTarget.ES2022,jsx:ts.JsxEmit.ReactJSX,esModuleInterop:true}}).outputText;
    const module={exports:{}};cache.set(file,module);
    const req=(name)=>{
      if(name==='react')return react;
      if(name==='react/jsx-runtime')return {jsx:(type,props)=>({type,props}),jsxs:(type,props)=>({type,props}),Fragment:'Fragment'};
      if(name==='@raycast/api')return api;
      if(name.startsWith('.')) {
        const base=path.resolve(path.dirname(file),name);const match=['.ts','.tsx'].map(e=>base+e).find(f=>fs.existsSync(f));
        if(match)return load(match);
      }
      throw new Error(`QA unknown import denied: ${name}`);
    };
    const context=vm.createContext({module,exports:module.exports,require:req,Buffer,console:Object.freeze({log:deny,error:deny}),setTimeout:deny,setInterval:deny,fetch:deny,process:undefined},{codeGeneration:{strings:false,wasm:false}});
    new vm.Script(compiled,{filename:file}).runInContext(context,{timeout:1000});return module.exports;
  }
  return {load,render(fn){cursor=0;return fn();},async effects(){const copy=effects;effects=[];for(const fn of copy)fn();await Promise.resolve();await Promise.resolve();},setFocus(s){focus=s;},get focus(){return focus;}};
}
async function original(c) {
  const env=sandbox({autoPaste:c.autopaste!==undefined,asciiCommaSeparated:c.comma??true,advanceView:c.advance??true,advanceViewLocation:'Bottom'},c.autopaste||'');
  const hook=env.load(path.join(sourceRoot,'src/hooks/use-'+c.command+'.ts')).default;
  const render=()=>env.render(()=>c.command==='byte-converter'?hook(units):hook());
  let converter=render();let from=10,to=2;
  if(converter.ref) {
    if(c.command==='base-converter') {for(const b of [2,8,10,16,32])converter.ref.current[b]={focus:()=>env.setFocus('base'+b)};if(c.advance!==false)converter.ref.current[36]={focus:()=>{from=36;env.setFocus('input');}};}
    else for(const u of Object.keys(units))converter.ref.current[u]={focus:()=>env.setFocus(u)};
  }
  await env.effects();converter=render();
  try {
    for(const step of c.steps||[]) {
      const {key,value}=step;
      if(c.command==='base-converter') {
        if(key==='fromBase')from=Number(value);
        else if(key==='toBase')to=Number(value);
        else if(key==='input')converter.set(from,value,undefined,1);
        else {const b=Number(key.slice(4));converter.set(b,value,({2:'0b',8:'0o',16:'0x'})[b]);}
      } else if(c.command==='byte-converter')converter.set(units[key],value);
      else converter.set(key,value);
      converter=render();
    }
    let expected={};
    if(c.command==='base-converter'){for(const b of [10,2,4,8,16,32])expected['base'+b]=converter.get(b);expected.input=converter.get(from,1);expected.output=converter.get(to,1);}
    else if(c.command==='byte-converter')for(const [key,e] of Object.entries(units))expected[key]=converter.get(e);
    else for(const key of codes)expected[key]=converter.get(key);
    return {expected:JSON.parse(JSON.stringify(expected)),best:c.command==='byte-converter'?converter.getBestUnitExpression():null,focus:env.focus};
  }catch(error){return {throws:error.name+': '+error.message};}
}
async function main() {
 const cases=[];
 const add=(command,key,value,extra={})=>cases.push({command,steps:[{key,value}],...extra});
 const numbers=['','0','00','0012',' 42 ','-1','+1','1.5','NaN','Infinity','1e3','0xFF','0Xff','0b101','0o77','12junk','１２','\u0085'+'12','\ufeff12','9007199254740993','999999999999999999999999999999999999999999999'];
 for(let b=2;b<=36;b++)for(const value of [...numbers,'abcdefghijklmnopqrstuvwxyz'.slice(0,b-10),b.toString(36)])add('base-converter','input',value,{steps:[{key:'fromBase',value:String(b)},{key:'toBase',value:String(38-b)},{key:'input',value}]});
 for(const key of ['base10','base2','base4','base8','base16','base32'])for(const value of numbers)add('base-converter',key,value);
 for(const text of ['0xFF','0Xff','0b101','0o77','hello','xyz','1e3','  42 ','-10','Infinity'])for(const advance of [true,false])cases.push({command:'base-converter',autopaste:text,advance});
 for(const key of Object.keys(units))for(const value of [...numbers,'0.1','0.999','1.234567','1e-3','1E3','1e+3','1e0x2','1e2junk','1e2e8','1e','1.2.3','.5','1.','0.00001','1e100','1e-100'])add('byte-converter',key,value);
 for(const text of ['8 bits','1 Bytes','1 KB','1 MB','1 GB','1 TB','1 PB','1 EB','1 b','1 B','1 kb','1 KiB','0.5 KB','12'])cases.push({command:'byte-converter',autopaste:text});
 const text=['','hello','a b+?!*~\'()','你好','Қазақша','😀','A\0B','\r\n\t','é','e\u0301','\u0080','\ufffd','\ud7ff','\ue000'];
 const encoded={unicode:['\\u0041','41junk','\\u0000','\\uD83D\\uDE00','\\ud800','\\udc00','\\uZZ','\\u-1','\\u10041'],base64:['MQ==','MQ','M',' M Q = = ','SGVsbG8_','/w==','4oI=','SGV$sbG8=ignored','====','8J-YgA','SGVsbG8===','🐱QQ'],utf8:['a\\x20b','\\xF0\\x9F\\x98\\x80','%E0%A4%A','\\xED\\xA0\\x80','%','a+b'],ascii:['65,66','65junk,0x42,,','55357,56832','55357','-1','65536','9999999999999999999999999999','131465'],hex:['0x0041','0x00','0x0','[4f60][597d]','[d83d][de00]','[d800]','0X41','zz41!','[41','[4]1','[]','123','[-1]','0'.repeat(64)+'41'],decimal:['65','0','-1','+65','0x4142','0b1000001','0o101','1.5','NaN','Infinity','12junk','9999999999999999999999999999'],url:['%F0%9F%98%80','%ED%A0%80','%00','%GG','%E0%A4%A','a+b','%FF'],entity:['&#x0041;','&#65;','&amp;','&#xD83D;&#xDE00;','&#xd800;','41;garbage','&#x1F600;',';&#x0041;;','&#xZZ;']};
 for(const comma of [true,false]) {
   for(const value of text)add('code-converter','native',value,{comma});
   for(const [key,values] of Object.entries(encoded))for(const value of values)add('code-converter',key,value,{comma});
 }
 // deterministic finite pseudo-random corpus; no source randomness exists.
 let seed=0x12345678;function next(){seed=(Math.imul(seed,1664525)+1013904223)>>>0;return seed;}
 for(let i=0;i<200;i++){let value='';for(let j=0;j<1+next()%30;j++)value+=String.fromCharCode(next()%65536);if(!/[\ud800-\udfff]/u.test(value))add('code-converter','native',value,{comma:!!(i%2)});}
 for(const key of ['unicode','ascii','hex','base64','entity'])for(let i=0;i<50;i++){let value='';for(let j=0;j<next()%40;j++)value+='0123456789abcdefXYZ,;[]\\u&#x% +-/='.charAt(next()%33);add('code-converter',key,value,{comma:!!(i%2)});}
 cases.push({command:'base-converter',steps:[{key:'input',value:'junk'},{key:'fromBase',value:'16'},{key:'fromBase',value:'10'}]});
 cases.push({command:'base-converter',steps:[{key:'base10',value:'00042'},{key:'toBase',value:'10'}]});
 const out=[];for(const c of cases)out.push({...c,...await original(c)});
 fs.writeFileSync(path.join(root,'qa/oracle.json'),JSON.stringify(out,null,2)+'\n');
 console.log(JSON.stringify({cases:out.length,throws:out.filter(c=>c.throws).length,typescript:ts.version,node:process.version}));
}
if(require.main===module)main().catch(e=>{console.error(e);process.exitCode=1;});
module.exports={sandbox,original};
