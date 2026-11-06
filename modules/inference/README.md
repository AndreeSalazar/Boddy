# Inferencia: adaptador local disponible, motor propio pendiente

crates/boddy-inference conecta el chat con llama.cpp en loopback. Los pesos son
Qwen2.5-Coder 7B Q4_K_M; el cálculo usa Vulkan en un proceso externo. Este nodo
mantiene pendiente el motor neuronal propio y su integración WGPU.
