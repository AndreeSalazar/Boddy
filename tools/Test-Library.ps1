param([string]$EvidencePath)
$ErrorActionPreference='Stop'
$OutputEncoding=[Text.UTF8Encoding]::new($false);[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false)
$taskRoot=(Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$taskBinary=Join-Path $taskRoot 'target\debug\boddy-cli.exe'
function Invoke-Library([hashtable]$Request){$raw=($Request | ConvertTo-Json -Compress) | & $taskBinary biblioteca;if($LASTEXITCODE -ne 0){throw 'Fallo Biblioteca.'};return ($raw -join "`n" | ConvertFrom-Json)}
$catalog=Invoke-Library @{action='categories'}
$found=Invoke-Library @{action='search';query='rust prestamos'}
if($found.documents_read -ne 0 -or $found.cards[0].id -ne 'rust-prestamos'){throw 'No se busco solo en registro o no se recupero la ficha esperada.'}
$packet=Invoke-Library @{action='context';query='rust prestamos';budget_bytes=512}
if($packet.context_bytes -gt 512 -or $packet.references.Count -lt 1){throw 'No se respeto el presupuesto.'}
$memory=Invoke-Library @{action='remember';query='rust prestamos';budget_bytes=512}
$recall=Invoke-Library @{action='recall';memory_id=$memory.memory_id;query='referencias mutables';budget_bytes=512}
if(-not $recall.reread -or ($recall.changes | Where-Object {$_.content_changed}).Count){throw 'Relectura incoherente.'}
$export=Invoke-Library @{action='export';memory_id=$memory.memory_id}
if(-not (Test-Path -LiteralPath (Join-Path $taskRoot $export.markdown_path))){throw 'Falta exportacion Markdown.'}
$language=Invoke-Library @{action='languages';query='fr'}
if($language.results[0].code -ne 'fr' -or $language.results[0].school_status -ne 'catalog_only'){throw 'Se presento un idioma catalogado como escuela completa.'}
$evidence=[ordered]@{schema=1;test='library-v1';categories=$catalog.categories.Count;cards=$catalog.cards;first_match=$found.cards[0].id;search_document_reads=$found.documents_read;budget_bytes=$packet.budget_bytes;context_bytes=$packet.context_bytes;source_bytes_read=$packet.source_bytes_read;references=$packet.references.Count;memory_bytes=$memory.bytes;memory_id=$memory.memory_id;reread=$recall.reread;new_query=$recall.query;export=$export;language_records=$language.records;iana_file_date=$language.provenance.file_date;iana_catalog_bytes=$language.provenance.bytes;tokens_measured=$false;training_effect=$packet.training_effect}
if($EvidencePath){[IO.File]::WriteAllText($ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($EvidencePath),($evidence | ConvertTo-Json -Depth 12),[Text.UTF8Encoding]::new($false))}
$evidence | ConvertTo-Json -Depth 12
