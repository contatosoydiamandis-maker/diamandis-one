# Manutenção do repositório

Este documento define uma rotina curta para manter o repositório previsível e fácil de contribuir.

## Antes de alterar

1. Atualize sua branch a partir de `main`.
2. Identifique os módulos afetados e seus limites de responsabilidade.
3. Evite refatorações amplas misturadas com mudanças funcionais.
4. Não adicione dependências para resolver tarefas triviais sem justificar o custo.

## Durante a implementação

- Prefira funções pequenas com entradas e saídas explícitas.
- Mantenha a lógica de filesystem e sistema no Rust; a interface deve orquestrar e apresentar estados.
- Centralize validações de segurança no backend.
- Trate estados de carregamento, erro, vazio e sucesso.
- Evite timers, listeners e tarefas assíncronas sem limpeza no unmount.
- Compartilhe requisições idênticas quando várias partes da UI puderem dispará-las ao mesmo tempo.
- Use nomes e mensagens consistentes em português na interface.
- Preserve acessibilidade: rótulos para botões de ícone, foco visível, estados anunciados e controles desabilitados durante operações.

## Verificações locais

```powershell
npm run check
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
```

Para gerar o instalador Windows:

```powershell
npm run tauri build
```

## Revisão antes do Pull Request

- [ ] Mudança limitada ao escopo descrito.
- [ ] Sem arquivos gerados, binários, logs ou segredos adicionados por acidente.
- [ ] Testes e verificações executados.
- [ ] Documentação atualizada.
- [ ] Operações de arquivos e backups revisadas quanto a sobrescrita e perda de dados.
- [ ] Cópias entre volumes testadas com arquivos, pastas aninhadas, destino existente e links simbólicos.
- [ ] Limitações conhecidas registradas.
- [ ] Nenhuma afirmação de compatibilidade sem teste real.

## Arquivos gerados

O ícone `src-tauri/icons/icon.ico` é produzido a partir de `icon.ico.b64` pelo script `prepare:assets` durante o build. Diretórios `node_modules`, `dist`, `src-tauri/target` e `src-tauri/gen` não devem ser versionados.

Lockfiles devem ser versionados assim que forem recuperados e revisados; então os workflows devem trocar `npm install` por `npm ci` e usar `--locked` no Cargo.
