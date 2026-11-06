// Convierte una descarga oficial ya realizada. No abre red ni crea miles de carpetas.
const fs=require('fs'),path=require('path'),crypto=require('crypto');
const root=path.resolve(__dirname,'..');
const input=fs.readFileSync(path.join(root,'data/library/cache/iana-registry.txt'));
const text=input.toString('utf8').replaceAll('\r\n','\n');
const date=/^File-Date: (\d{4}-\d{2}-\d{2})/m.exec(text)?.[1];
if(!date)throw Error('Registro IANA sin fecha');
const rows=[];
for(const record of text.split('\n%%\n')){
 const fields={};let key;
 for(const line of record.split('\n')){const m=/^([A-Za-z-]+): (.*)$/.exec(line);if(m){key=m[1];(fields[key]??=[]).push(m[2]);}else if(key&&line.startsWith(' ')){fields[key][fields[key].length-1]+=' '+line.trim();}}
 if(fields.Type?.[0]!=='language')continue;
 const code=fields.Subtag?.[0];if(!code)throw Error('Idioma sin subtag');
 const safe=v=>(v||'').replace(/[\t\r\n]/g,' ');
 rows.push([code,fields.Description.join(' / '),fields.Deprecated?.[0]||'',fields['Preferred-Value']?.[0]||'',fields.Scope?.[0]||''].map(safe));
}
rows.sort((a,b)=>a[0].localeCompare(b[0],'en'));
if(rows.length<7000||new Set(rows.map(r=>r[0])).size!==rows.length)throw Error('Registro incompleto o duplicado');
const output='code\tname\tdeprecated\tpreferred\tscope\n'+rows.map(r=>r.join('\t')).join('\n')+'\n';
const folder=path.join(root,'biblioteca/02-babel');fs.mkdirSync(folder,{recursive:true});
fs.writeFileSync(path.join(folder,'idiomas.tsv'),output);
fs.writeFileSync(path.join(folder,'procedencia.json'),JSON.stringify({schema:1,source:'https://www.iana.org/assignments/language-subtag-registry/language-subtag-registry',file_date:date,source_sha256:crypto.createHash('sha256').update(input).digest('hex'),catalog_sha256:crypto.createHash('sha256').update(output).digest('hex'),records:rows.length,bytes:Buffer.byteLength(output),scope:'Todos los registros Type: language de esta instantanea IANA; incluye alias obsoletos, macrolenguajes y rangos. No contiene cursos ni prueba conocimiento de idiomas.'})+'\n');
console.log(JSON.stringify({date,records:rows.length,bytes:Buffer.byteLength(output)}));
