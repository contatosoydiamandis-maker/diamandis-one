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

CPU é calculada por deltas de `GetSystemTimes`; RAM usa `GlobalMemoryStatusEx`; armazenamento enumera unidades fixas e removíveis montadas no Windows, consultando espaço por volume. Leituras acontecem no backend e fora da thread de interface. A UI limita a frequência de atualização e compartilha requisições simultâneas para não duplicar consultas. GPU permanece explicitamente indisponível até existir um provedor confiável.

### File Manager

A listagem é executada fora da thread principal, com paginação, filtros e ordenação no backend. O seletor de unidades permite alternar entre volumes detectados. Operações de alteração validam nomes e caminhos, recusam links simbólicos e não sobrescrevem destinos. Movimentações no mesmo volume usam rename; entre volumes, o backend copia arquivos ou árvores de diretórios para um destino novo, sincroniza os arquivos e só então remove a origem. Se a cópia falhar, a origem permanece; se a remoção final falhar, ambas as cópias permanecem e a UI recebe um erro explícito. Links simbólicos e tipos especiais são recusados na cópia entre volumes. Exclusão usa a Lixeira do sistema.

### GameHub

Steam, Epic Games e GOG são descobertos por manifests e diretórios conhecidos. Xbox é detectado no layout acessível `<unidade>:\\XboxGames`; o scanner não percorre `WindowsApps`, não altera ACLs e não tenta assumir posse de instalações protegidas. Bibliotecas Hydra e Outros são adicionadas pelo usuário: o scanner enumera somente as pastas de primeiro nível e procura executáveis até quatro níveis, sem seguir links simbólicos. Jogos fora dos conectores podem ser cadastrados manualmente escolhendo a pasta e o `.exe`. A descoberta não inicia launchers nem altera instalações. O cache é vinculado à configuração das bibliotecas para que adicionar ou remover uma pasta invalide resultados antigos; caminhos duplicados são consolidados. Favoritos, raízes adicionais, jogos manuais e executáveis selecionados ficam no armazenamento local do WebView. Antes de iniciar, o backend canonicaliza os caminhos e exige que o `.exe` seja um arquivo dentro da pasta do jogo; o processo é iniciado diretamente, sem shell intermediário. Jogos Xbox/MSIX que dependem de ativação de pacote ainda exigem um adaptador específico.

### Save Center

O catálogo fica em `app_local_data_dir/save-center/games.json`; backups ficam em `app_local_data_dir/save-center/backups/<game-id>/<backup-id>`. A criação copia arquivos, recusa links simbólicos e gera SHA-256 por arquivo. A verificação confere estrutura, tamanho, contagem e hashes; backups legados sem manifesto recebem verificação estrutural identificada como tal. A restauração prepara e verifica um payload temporário e preserva o save atual antes da troca. O usuário pode exportar um backup verificado para uma pasta escolhida, com manifesto e payload independentes, ou excluir uma cópia específica após confirmação; nenhuma dessas ações altera a pasta original do jogo.

### Toolbox\n\nO módulo executa operações no WebView local: transformação de texto, JSON, Base64 UTF-8, codificação de URL, SHA-256, UUID v4, conversão HEX/RGB e timestamp/ISO 8601. Os campos de conversão validam entrada antes de produzir resultado. Timers de feedback da área de transferência são cancelados ao desmontar o módulo.\n\n### Plugins e extensibilidade\n\nO contrato API v1 valida manifests, versões, IDs, permissões conhecidas e duplicatas; há um registro em memória para metadados. A UI de Configurações permite validar JSON de manifesto. A pré-alpha não instala nem executa código externo: permissões declaradas não são concedidas automaticamente. Carregamento de plugins depende de persistência, consentimento, isolamento e API de mensagens versionada; ver `docs/PLUGINS.md`.\n\n### Adaptador Zelda: TOTK Recomp

A instalação é identificada por `totk.exe`. Importação copia o perfil para uma campanha gerenciada, sem modificar a origem. A inicialização exige backup preventivo e define `TOTK_PROFILE_DIR`. Compatibilidade dessa variável precisa ser confirmada com uma build real do Recomp.

## Segurança e concorrência

- Permissões Tauri são mínimas e explícitas.
- Caminhos fornecidos pela interface são validados novamente no Rust.
- Links simbólicos são recusados em operações sensíveis.
- Cópias entre volumes nunca sobrescrevem destinos e mantêm a origem até a cópia terminar.
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
