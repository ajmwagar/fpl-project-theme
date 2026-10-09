// Run with node --test tests/docs-copy.cjs; no DOM package needed.
const test = require('node:test');
const assert = require('node:assert/strict');
const vm = require('node:vm');
const fs = require('node:fs');
const code = fs.readFileSync(new URL('../assets/docs.js', `file://${__filename}`), 'utf8');
function harness({response, clipboard}) {
  let click;
  const status = {textContent:'', setAttribute(name,value){this[name]=value;}};
  const button = {dataset:{copyMarkdown:'model.md'}, disabled:false, parentElement:{querySelector(){return status;}},addEventListener(event,fn){click=fn;}};
  vm.runInNewContext(code, {document:{querySelectorAll(){return [button];}},navigator:{clipboard}, fetch:async path => {assert.equal(path,'model.md');return response;}});
  return {button,status,click};
}
test('copies exact fetched canonical Markdown, including frontmatter and tables', async () => {
  const markdown='---\ntitle: Models\n---\n# Models\n\n| A | B |\n|---|---|\n| x | y |\n';
  let copied;
  const h=harness({response:{ok:true,text:async()=>markdown},clipboard:{writeText:async value=>{copied=value;}}});
  await h.click(); assert.equal(copied,markdown); assert.equal(h.status.textContent,'Markdown copied.'); assert.equal(h.button.disabled,false);
});
test('request errors never copy an error page and expose download fallback', async () => {
  let copied=false;
  const h=harness({response:{ok:false,status:404},clipboard:{writeText:async()=>{copied=true;}}});
  await h.click(); assert.equal(copied,false); assert.match(h.status.textContent,/404.*Download Markdown/);assert.equal(h.status.role,'alert');assert.equal(h.button.disabled,false);
});
test('denied clipboard exposes visible fallback and restores the control', async () => {
  const h=harness({response:{ok:true,text:async()=>'# Page'},clipboard:{writeText:async()=>{throw new Error('Permission denied');}}});
  await h.click();assert.match(h.status.textContent,/Permission denied.*Download Markdown/);assert.equal(h.button.disabled,false);
});
test('insecure context exposes visible fallback', async () => {
  const h=harness({response:{ok:true,text:async()=>'# Page'}});await h.click();assert.match(h.status.textContent,/Clipboard unavailable.*Download Markdown/);
});
