const fs=require('node:fs'),path=require('node:path');const {sandbox}=require('./oracle.cjs');
const root=path.resolve(__dirname,'..');
function expand(n){if(n==null||typeof n!=='object')return n;if(Array.isArray(n))return n.flatMap(x=>{const v=expand(x);return Array.isArray(v)?v:[v];});if(typeof n.type==='function'&&!n.type._qaType)return expand(n.type(n.props));return {type:n.type?._qaType||n.type,props:n.props,children:expand(n.props?.children||[])};}
(async()=>{const result=[];
for(const command of ['base-converter','code-converter','byte-converter'])for(const prefs of [{advanceView:true,advanceViewLocation:'Bottom'},{advanceView:true,advanceViewLocation:'Top'},{advanceView:false,advanceViewLocation:'Bottom'}]){
const env=sandbox({autoPaste:false,asciiCommaSeparated:true,...prefs},'');const component=env.load(path.join(root,'qa/fixtures/original/src',command+'.tsx')).default;
const tree=env.render(()=>expand(component()));const fields=[],actions=[];
function walk(n){if(!n||typeof n!=='object')return;if(Array.isArray(n)){n.forEach(walk);return;}if(n.type?.startsWith('Form.'))fields.push({type:n.type,title:n.props.title??null,value:n.props.value??null});if(n.type?.startsWith('Action.')||n.type==='Action')actions.push({title:n.props.title,content:n.props.content??null});walk(n.children);if(n.props?.actions)walk(expand(n.props.actions));}
walk(tree);result.push({command,prefs,fields,actions});}
fs.writeFileSync(path.join(root,'qa/ui-oracle.json'),JSON.stringify(result,null,2)+'\n');console.log('Source TSX UI snapshots:',result.length);
})().catch(e=>{console.error(e);process.exitCode=1;});
