# Nodos de Boddy

## crates/boddy-agent
Investigador con busqueda de componentes y una correccion de citas verificada. Estado: implemented. Dependencias: boddy-core, boddy-inference, boddy-reader, boddy-supergrafo.

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

## crates/boddy-supergrafo
Consultas externas acotadas. Estado: implemented. Dependencias: ninguna.

## crates/boddy-app
Casos de uso de la aplicacion. Estado: implemented. Dependencias: boddy-agent, boddy-catalog, boddy-core, boddy-desktop, boddy-inference, boddy-lab, boddy-reader, boddy-supergrafo, boddy-telemetry.

## crates/boddy-catalog
Catalogo documental local. Estado: implemented. Dependencias: ninguna.

## crates/boddy-desktop
Acciones manuales de escritorio registradas y auditadas. Estado: implemented. Dependencias: boddy-reader.

## crates/boddy-lab
Experimentos Tiny y Attention: corpus, entrenamiento, evaluacion, registro y checkpoints propios. Estado: implemented. Dependencias: boddy-neural, boddy-reader, boddy-transformer, boddy-wgpu.

## crates/boddy-neural
MLP no_std de siguiente caracter y gradientes CPU de referencia. Estado: implemented. Dependencias: ninguna.

## crates/boddy-transformer
Decoder no_std con embeddings, atencion causal, RMSNorm y autodiferenciacion de referencia. Estado: implemented. Dependencias: ninguna.

## gpu/wgpu
Entrenamiento MLP y cinta Transformer con shaders propios y tipos WGPU privados. Estado: implemented. Dependencias: boddy-neural, boddy-transformer.

## crates/boddy-telemetry
Muestra de VRAM y uso de GPU, independiente del modelo. Estado: implemented. Dependencias: ninguna.

## crates/boddy-cli
Entrada de terminal. Estado: implemented. Dependencias: boddy-app.

## window
Lanzador grafico Windows sin consola. Estado: implemented. Dependencias: boddy-cli.

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
