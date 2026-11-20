# Arquitetura do DIAMANDIS ONE

## Princípios

1. **Local-first:** dados e operações permanecem na máquina do usuário.
2. **Modularidade:** cada domínio tem responsabilidades e interfaces explícitas.
3. **Segurança por padrão:** entradas da interface são não confiáveis; alterações destrutivas exigem validação e confirmação.
4. **Conectores isolados:** launchers e adaptadores de jogos não devem contaminar o núcleo.
5. **Falha explícita:** sensores indisponíveis e operações incompletas nunca devem ser representados como sucesso.
6. **Distribuição simples:** usuários finais instalam e executam o produto por interface gráfica, sem ferramentas de desenvolvimento.

## Visão em camadas

- **Interface:** React + TypeScript + Vite.
- **Desktop bridge:** Tauri 2 IPC e permissões declaradas em `src-tauri/capabilities`.
- **Núcleo:** Rust, responsável por filesystem, métricas nativas, persistência e integração com processos.
- **Preferências:** JSON em `app_config_dir`, gravado por arquivo temporário e substituição atômica.
- **Dados de usuário:** catálogo e backups do Save Center em `app_local_data_dir`.

## Organização do código

### Frontend

- `src/App.tsx`: navegação principal e File Manager.
- `src/modules/`: módulos de domínio independentes.
- `src/theme.ts`: aplicação e acompanhamento do tema.
- `src/types.ts`: tipos compartilhados da navegação.
- `src/styles.css`: tokens visuais, layout e estilos dos módulos.

### Backend Rust

- `lib.rs`: registro de comandos Tauri e listagem de diretórios.
- `atomic_file.rs`: substituição de arquivos com proteção contra gravação parcial.
- `file_ops.rs`: operações explícitas de arquivos e validação de nomes.
- `gamehub.rs`: descoberta somente leitura e abertura de pastas de jogos.
- `save_center.rs`: catálogo, hashes, backups, verificação e restauração.
- `settings.rs`: validação e persistência de preferências.
- `system_metrics.rs`: métricas nativas do sistema.
- `totk_recomp.rs`: campanhas e integração isolada do TOTK Recomp.

## Responsabilidades dos módulos

### Dashboard

CPU é calculada por deltas de `GetSystemTimes`; RAM usa `GlobalMemoryStatusEx`; armazenamento usa `GetDiskFreeSpaceExW`. Leituras acontecem no backend e fora da thread de interface. A UI limita a frequência de atualização e compartilha requisições simultâneas para não duplicar consultas. GPU permanece explicitamente indisponível até existir um provedor confiável.

### File Manager

A listagem é executada fora da thread principal, com paginação, filtros e ordenação no backend. Operações de alteração validam nomes e caminhos, recusam links simbólicos e não sobrescrevem destinos. Exclusão usa a Lixeira do sistema.

### GameHub

Steam, Epic Games e GOG são descobertos por leitura de manifests e diretórios conhecidos. A varredura não inicia launchers nem altera instalações. Requisições simultâneas de descoberta são compartilhadas no frontend e o resultado recente é mantido em cache curto. Favoritos são locais ao perfil do navegador/WebView.

### Save Center

O catálogo fica em `app_local_data_dir/save-center/games.json`; backups ficam em `app_local_data_dir/save-center/backups/<game-id>/<backup-id>`. A criação copia arquivos, recusa links simbólicos e gera SHA-256 por arquivo. A verificação confere estrutura, tamanho, contagem e hashes; backups legados sem manifesto recebem verificação estrutural identificada como tal. A restauração prepara e verifica um payload temporário e preserva o save atual antes da troca.

### Adaptador Zelda: TOTK Recomp

A instalação é identificada por `totk.exe`. Importação copia o perfil para uma campanha gerenciada, sem modificar a origem. A inicialização exige backup preventivo e define `TOTK_PROFILE_DIR`. Compatibilidade dessa variável precisa ser confirmada com uma build real do Recomp.

## Segurança e concorrência

- Permissões Tauri são mínimas e explícitas.
- Caminhos fornecidos pela interface são validados novamente no Rust.
- Links simbólicos são recusados em operações sensíveis.
- Catálogos e preferências usam gravação atômica.
- Operações mutáveis do Save Center e do adaptador TOTK compartilham lock de processo.
- A UI nunca deve substituir validações do backend por validações apenas visuais.
- Segredos, tokens e dados privados não pertencem ao repositório.

## Build e qualidade

- Node.js 22+ e npm 10+.
- `npm run check`: TypeScript e build de produção.
- `cargo test --manifest-path src-tauri/Cargo.toml --all-targets`: testes nativos.
- `npm run tauri build`: prepara assets e gera o pacote desktop.
- CI valida frontend, testes Rust e instalador NSIS x64.

Lockfiles ainda precisam ser versionados. Até lá, os builds de CI não são totalmente reproduzíveis byte a byte.

## Próximos marcos técnicos

- Versionar `package-lock.json` e `src-tauri/Cargo.lock`, então migrar para `npm ci` e `cargo --locked`.
- Validar instalação, atualização e remoção em Windows limpo.
- Validar TOTK Recomp com instalação real.
- Medir inicialização, memória em repouso e operações em diretórios grandes.
- Expandir conectores do GameHub com testes específicos por launcher.
