const fs = require('fs');
const path = require('path');
const cp = require('child_process');
const root = path.resolve(__dirname, '..');
const descriptions = {
 'boddy-contracts':'Errores compartidos sin plataforma',
 'boddy-mission':'Razon, objetivo y aceptacion',
 'boddy-policy':'Capacidades explicitas',
 'boddy-core':'Estados, cupos y evidencia',
 'boddy-catalog':'Catalogo documental local',
 'boddy-supergrafo':'Consultas externas acotadas',
 'boddy-app':'Casos de uso de la aplicacion',
 'boddy-cli':'Entrada de terminal',
 'boddy-inference':'Chat con motor local por loopback, sin herramientas',
 'boddy-window':'Lanzador grafico Windows sin consola',
 'boddy-telemetry':'Muestra de VRAM y uso de GPU, independiente del modelo',
 'boddy-reader':'Lectura de fuentes acotada y evidencia con SHA256',
 'boddy-agent':'Investigador con busqueda de componentes y una correccion de citas verificada',
 'boddy-desktop':'Acciones manuales de escritorio registradas y auditadas',
 'boddy-neural':'MLP no_std de siguiente caracter y gradientes CPU de referencia',
 'boddy-wgpu':'Entrenamiento MLP con shaders propios WGPU y tipos GPU privados',
 'boddy-lab':'Entrenamiento reanudable, registro de modelos, control no entrenado y generacion Tiny CPU'
};
const result = cp.spawnSync('cargo', ['metadata','--offline','--no-deps','--format-version','1'], {cwd:root,encoding:'utf8'});
if(result.status !== 0) throw Error(result.stderr);
const packages = JSON.parse(result.stdout).packages;
const names = new Set(packages.map(p=>p.name));
const nodes = packages.map(p=>({id:p.name,path:path.relative(root,path.dirname(p.manifest_path)).replaceAll('\\','/'),purpose:descriptions[p.name],status:'implemented',no_std:fs.readFileSync(path.join(path.dirname(p.manifest_path),'src',['boddy-cli','boddy-window'].includes(p.name)?'main.rs':'lib.rs'),'utf8').includes('#![no_std]'),dependencies:p.dependencies.filter(d=>names.has(d.name)).map(d=>d.name)}));
nodes.find(n=>n.id==='boddy-window').dependencies.push('boddy-cli'); // Frontera de proceso, no enlace Rust.
const future = ['inference','tokenizer','ingestion','retrieval','session','executor','verification','evaluation'];
for(const id of future) {
 const folder = 'modules/'+id;
 fs.mkdirSync(path.join(root,folder),{recursive:true});
 const file = path.join(root,folder,'README.md');
 if(!fs.existsSync(file)) fs.writeFileSync(file,`# ${id} — pendiente\n\nFrontera independiente reservada. Sin implementacion conectada todavia.\nAntes de integrarla: definir entradas, salidas, permisos, presupuesto y prueba\nde aceptacion. Ver docs/MODULARIDAD.md y docs/PLAN.md.\n`);
 nodes.push({id,path:folder,purpose:'Frontera de '+id,status:'planned',dependencies:[]});
}
const byId = new Map(nodes.map(n=>[n.id,n]));
function level(id, stack=[]) {
 if(stack.includes(id)) throw Error('Ciclo: '+[...stack,id].join(' -> '));
 const n=byId.get(id); if(!n) throw Error('Nodo desconocido '+id);
 return n.dependencies.length ? 1+Math.max(...n.dependencies.map(d=>level(d,[...stack,id]))) : 0;
}
const positions={};const rows={};
for(const n of nodes) { n.layer= n.status==='planned'?6:level(n.id); const row=rows[n.layer]||0; positions[n.path]=[n.layer*340,row*145];rows[n.layer]=row+1; }
const dir=path.join(root,'super-grafo');
fs.writeFileSync(path.join(dir,'nodos.json'),JSON.stringify({schema:1,source:'Cargo metadata + fronteras planificadas explicitas',nodes},null,2)+'\n');
const orderFile=path.join(dir,'orden.json');
// Preserve layout edits after the first generation.
const order=fs.existsSync(orderFile)?JSON.parse(fs.readFileSync(orderFile,'utf8')):{posiciones:{},grafo:{},desplegadas:['crates','gpu','modules'],rejilla:false,bloqueada:false,foco:''};
order.grafo ||= {};
for(const [node,position] of Object.entries(positions)) {
 if(!Object.hasOwn(order.grafo,node)) {
  const occupied=Object.values(order.grafo).filter(p=>p[0]===position[0]).map(p=>p[1]);
  order.grafo[node]=[position[0],Math.max(position[1],occupied.length?Math.max(...occupied)+145:0)];
 }
}
fs.writeFileSync(orderFile,JSON.stringify(order,null,2)+'\n');
let notes='# Nodos de Boddy\n\n';
for(const n of nodes) notes+=`## ${n.path}\n${n.purpose}. Estado: ${n.status}. Dependencias: ${n.dependencies.join(', ')||'ninguna'}.\n\n`;
fs.writeFileSync(path.join(dir,'NOTAS.md'),notes.trimEnd()+'\n');
console.log(`${nodes.length} nodos registrados; dependencias sin ciclos.`);
