# Nodos de Boddy

## crates/boddy-agent
Investigador de lectura y controlador Rust de ranking, propuesta y verificacion acotada. Estado: implemented. Dependencias: boddy-core, boddy-inference, boddy-reader, boddy-rust-tasks, boddy-supergrafo.

## crates/boddy-core
Estados, cupos y evidencia. Estado: implemented. Dependencias: boddy-contracts, boddy-mission, boddy-policy.

## crates/boddy-contracts
Errores compartidos sin plataforma. Estado: implemented. Dependencias: ninguna.

## crates/boddy-mission
Razon, objetivo y aceptacion. Estado: implemented. Dependencias: boddy-contracts.

## crates/boddy-policy
Capacidades explicitas. Estado: implemented. Dependencias: ninguna.

## crates/boddy-inference
Chat con motor local por loopback, sin herramientas. Estado: implemented. Dependencias: ninguna.

## crates/boddy-reader
Lectura de fuentes acotada y evidencia con SHA256. Estado: implemented. Dependencias: ninguna.

## crates/boddy-rust-tasks
Curriculo Rust y ejercicios tipados no_std con verificador finito de expresiones cerradas. Estado: implemented. Dependencias: ninguna.

## crates/boddy-supergrafo
Consultas externas acotadas. Estado: implemented. Dependencias: ninguna.

## crates/boddy-app
Casos de uso de la aplicacion. Estado: implemented. Dependencias: boddy-agent, boddy-catalog, boddy-core, boddy-desktop, boddy-inference, boddy-lab, boddy-library, boddy-reader, boddy-supergrafo, boddy-telemetry.

## crates/boddy-catalog
Catalogo documental local. Estado: implemented. Dependencias: ninguna.

## crates/boddy-desktop
Acciones manuales de escritorio registradas y auditadas. Estado: implemented. Dependencias: boddy-reader.

## crates/boddy-lab
Experimentos Tiny y Attention: corpus, entrenamiento, evaluacion, registro y checkpoints propios. Estado: implemented. Dependencias: boddy-neural, boddy-reader, boddy-rust-tasks, boddy-transformer, boddy-wgpu.

## crates/boddy-neural
MLP no_std de siguiente caracter y gradientes CPU de referencia. Estado: implemented. Dependencias: ninguna.

## crates/boddy-transformer
Decoder no_std con embeddings, atencion causal, RMSNorm y autodiferenciacion de referencia. Estado: implemented. Dependencias: ninguna.

## gpu/wgpu
Entrenamiento MLP y cinta Transformer con shaders propios y tipos WGPU privados. Estado: implemented. Dependencias: boddy-neural, boddy-transformer.

## crates/boddy-library
Biblioteca local: registro, recuperacion lexica por presupuesto, recuerdos y exportacion con procedencia. Estado: implemented. Dependencias: boddy-reader, library-data.

## crates/boddy-telemetry
Muestra de VRAM y uso de GPU, independiente del modelo. Estado: implemented. Dependencias: ninguna.

## crates/boddy-cli
Entrada de terminal. Estado: implemented. Dependencias: boddy-app.

## window
Lanzador grafico Windows sin consola. Estado: implemented. Dependencias: boddy-cli.

## biblioteca/01-forja
La Forja: Programacion: escuelas, fundamentos, motivos y comprobaciones. Estado: partial. Dependencias: ninguna.

## biblioteca/02-babel
Babel: Idiomas naturales: catalogo IANA y escuelas documentales ampliables. Estado: partial. Dependencias: ninguna.

## biblioteca/03-agora
Agora: Razonamiento: hipotesis, evidencia y contradicciones. Estado: partial. Dependencias: ninguna.

## biblioteca/04-atelier
Atelier: Creacion: problemas, alternativas y criterios de decision. Estado: partial. Dependencias: ninguna.

## biblioteca/05-bitacora
Bitacora: Memoria documental: referencias, contexto y relectura. Estado: partial. Dependencias: ninguna.

## biblioteca
Registro de fichas, relaciones documentales y catalogo IANA; contenido parcial, no pesos entrenados. Estado: partial. Dependencias: biblioteca-forja, biblioteca-babel, biblioteca-agora, biblioteca-atelier, biblioteca-bitacora.

## assets/identity
Arte conceptual de documentacion, prompts y procedencia; no capacidad de los modelos. Estado: documentation. Dependencias: ninguna.

## modules/inference
Frontera de inference. Estado: planned. Dependencias: ninguna.

## modules/tokenizer
Frontera de tokenizer. Estado: planned. Dependencias: ninguna.

## modules/ingestion
Frontera de ingestion. Estado: planned. Dependencias: ninguna.

## modules/retrieval
Frontera de retrieval. Estado: planned. Dependencias: ninguna.

## modules/session
Frontera de session. Estado: planned. Dependencias: ninguna.

## modules/executor
Frontera de executor. Estado: planned. Dependencias: ninguna.

## modules/verification
Frontera de verification. Estado: planned. Dependencias: ninguna.

## modules/evaluation
Frontera de evaluation. Estado: planned. Dependencias: ninguna.
