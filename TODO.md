# TODO: HW Rendering — Reimplementação

## Problema

O PPSSPP crasha em `Draw::CreateShader` porque o contexto GL não existe
quando o core chama `get_proc_address` durante `SET_HW_RENDER`.
A causa raiz é a ordem errada de inicialização.

## Ordem correta (baseada no Ludo)

```
1. Criar janela (sem GL)
2. Criar contexto GL + FBO          ← antes de load_game
3. retro_load_game()
     └── core chama SET_HW_RENDER
           └── get_proc_address já funciona, GL existe
4. Chamar context_reset do CORE     ← frontend avisa o core que GL está pronto
5. Loop de frames:
     ├── bind FBO
     ├── retro_run()
     └── blit FBO → tela → swap_buffers
```

---

## Etapas

### 1. Janela

- [ ] Criar a janela winit/glutin **sem** inicializar o contexto GL
- [ ] O contexto GL deve ser criado em um passo separado, depois que a janela estiver visível

---

### 2. Contexto GL

- [ ] Criar contexto OpenGL Core Profile (versão mínima 3.1)
- [ ] Fazer `make_current` antes de qualquer operação GL
- [ ] Expor `get_proc_address` para resolver símbolos GL — usado pelo core via `hw_cb.get_proc_address`

---

### 3. FBO (Framebuffer Object)

- [ ] Criar o FBO com dimensões `max_width` × `max_height` vindas do `av_info`
- [ ] Anexar textura RGBA8 ao FBO
- [ ] Anexar RBO de depth se `graphic_api.depth == true`
- [ ] Anexar RBO de depth+stencil se `graphic_api.depth && graphic_api.stencil`
- [ ] Verificar `GL_FRAMEBUFFER_COMPLETE` antes de continuar
- [ ] Salvar o ID do FBO em `graphic_api.fbo` para o callback `get_current_framebuffer`

---

### 4. Callbacks registrados no `hw_cb` (SET_HW_RENDER)

O frontend preenche apenas dois campos — o resto pertence ao core:

- [ ] `hw_cb.get_current_framebuffer` → retorna o ID do FBO
- [ ] `hw_cb.get_proc_address` → resolve símbolos GL via `gl_context.display()`
- [ ] **Não tocar** em `hw_cb.context_reset` nem `hw_cb.context_destroy` — são do core
- [ ] Salvar `hw_cb.context_reset` e `hw_cb.context_destroy` para chamar depois
- [ ] Salvar `hw_cb.context_type`, `depth`, `stencil`, `version_major/minor` no `GraphicApi`

---

### 5. Sequência de inicialização do jogo

```
criar_janela()
criar_contexto_gl()         ← GL current antes de load_game
criar_fbo()                 ← FBO pronto antes de load_game
retro_load_game()           ← core chama SET_HW_RENDER aqui
chamar_context_reset_core() ← frontend chama o fn pointer do core
init_audio()
```

- [ ] `GET_PREFERRED_HW_RENDER` deve retornar `true`
- [ ] Não chamar `context_reset` do core antes de `load_game` retornar

---

### 6. Loop de frames

- [ ] Antes de `retro_run`: bind do FBO (`glBindFramebuffer(GL_FRAMEBUFFER, fbo_id)`)
- [ ] Após `retro_run`: verificar se o frame é HW (`data == RETRO_HW_FRAME_BUFFER_VALID`)
- [ ] Frame HW: blit do FBO para o backbuffer (`glBlitFramebuffer`)
- [ ] Frame SW: upload de pixels via `glTexImage2D` e draw normal
- [ ] `swap_buffers` ao final de cada frame

---

### 7. Sincronização de frames

- [ ] Ignorar frames enquanto `fps == 0` (av_info ainda não populado)
- [ ] Não chamar `Duration::from_secs_f64` com valor zero, NaN ou infinito

---

## Referência

```go
// Ludo — core/core.go
ok := state.Core.LoadGame(*gi)
if state.Core.HWRenderCallback != nil {
    vid.InitFramebuffer()                      // cria FBO
    state.Core.HWRenderCallback.ContextReset() // chama context_reset do CORE
}
```
