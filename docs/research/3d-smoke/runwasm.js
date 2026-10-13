const fs=require("fs");
const m=new WebAssembly.Module(fs.readFileSync(process.argv[2]));
const imports={};
for (const i of WebAssembly.Module.imports(m)) {
  imports[i.module] ??= {};
  imports[i.module][i.name] = (...a)=>{ throw new Error("import called: "+i.name); };
}
const inst=new WebAssembly.Instance(m,imports);
const t=performance.now();
const r=inst.exports[process.argv[3]]();
console.log(process.argv[3], "=>", r, "in", (performance.now()-t).toFixed(1), "ms");
