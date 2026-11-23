# DIAMANDIS ONE

**Um hub desktop local-first para Windows:** organização de arquivos, ferramentas de produtividade e gerenciamento da biblioteca gamer em um só lugar.

> **Status: pré-alpha 0.2.0.** A base desktop inclui navegação de arquivos, operações controladas, preferências persistentes, descoberta local de jogos Steam/Epic/GOG, XboxGames e bibliotecas Hydra/Outros configuradas pelo usuário e Save Center com backups SHA-256 e restauração protegida. O adaptador TOTK Recomp já oferece campanhas isoladas, importação por cópia e backup obrigatório antes de iniciar. A CI automatiza o build do frontend, os testes Rust e a geração do instalador NSIS para Windows. A aprovação deve ser conferida na execução associada ao commit; ainda faltam testes físicos em Windows limpo e validação com uma instalação real do TOTK Recomp.

## Visão do projeto

- **Local-first:** dados e operações permanecem no computador do usuário.
- **Privacidade:** sem anúncios, sem telemetria invasiva e sem conta ou nuvem obrigatória.
- **Modular:** Dashboard, File Manager, Toolbox, GameHub, Save Center e Configurações.
- **Seguro:** operações destrutivas exigem confirmação e usam a Lixeira; backups são cópias e restaurações preservam o estado anterior.
- **Extensível:** conectores independentes para launchers e jogos.

## Stack

- React + TypeScript + Vite
- Tauri 2 + Rust
- Persistência local em JSON para preferências e catálogo inicial do Save Center; SQLite permanece planejado quando o volume de dados justificar.

## Estado atual

| Área | Estado |
|---|---|
| Dashboard e identidade visual | Monitoramento nativo de CPU, RAM e unidades fixas/removíveis montadas no Windows; identidade visual e diagnóstico do core |
| File Manager | Navegação entre unidades, filtro, criação, renomeação, cópia de arquivos e pastas, movimentação entre volumes com proteção contra perda e Lixeira |
| Configurações | Tema escuro/claro/sistema, preferências persistentes e validador de manifesto de plugins API v1 |
| Toolbox | Texto, JSON, Base64, URL, SHA-256, UUID, conversor de cores HEX/RGB e timestamp offline |
| GameHub | Descoberta somente leitura de Steam, Epic, GOG e XboxGames acessíveis; bibliotecas Hydra/Outros configuráveis, cadastro manual, favoritos e inicialização de executáveis validados |
| Save Center | Cadastro manual de pastas, backups locais, histórico e restauração protegida |
| Seleção de pastas | Diálogo nativo do Windows via Tauri |
| Build/instalador | CI gera instalador NSIS x64; instalação física ainda não validada em máquina Windows limpa |

## Instalação para usuários

A versão distribuída será instalada por um assistente gráfico do Windows. Usuários finais não precisarão instalar Node.js ou Rust, nem abrir CMD ou PowerShell. Consulte [Instalação e uso](docs/INSTALLATION.md). O artefato de CI atual é experimental e não representa uma versão estável.

O pipeline de validação é executado automaticamente em Pull Requests direcionados à `main` e em pushes na `main`. Os pacotes gerados pelo CI são artefatos temporários de teste, não releases públicas. O workflow de distribuição é separado e só prepara um rascunho de pré-release quando uma tag `v*` é criada. Nenhuma versão será considerada estável sem testes reais em Windows limpo.

### Qualidade e reprodutibilidade

- Use Node.js 22 ou superior e npm 10 ou superior para desenvolvimento.
- `npm run check` executa a verificação TypeScript e o build de produção.
- `npm run tauri build` prepara o ícone automaticamente antes de compilar o aplicativo.
- O CI executa os testes nativos Rust em todos os targets e publica instaladores como artefatos temporários.
- Os lockfiles ainda precisam ser adicionados ao repositório; até isso acontecer, as instalações de dependências não são totalmente reproduzíveis.

## Desenvolvimento

### Requisitos

- Windows 10/11
- Node.js 22
- Rust stable com toolchain MSVC
- Microsoft C++ Build Tools
- WebView2 Runtime

### Comandos para contribuidores

~~~powershell
npm install
npm run tauri dev
~~~

Gerar o instalador:

~~~powershell
npm run tauri build
~~~

Os comandos de terminal pertencem exclusivamente ao desenvolvimento, não à instalação ou ao uso da versão distribuída.

## Como contribuir

1. Leia [CONTRIBUTING.md](CONTRIBUTING.md).
2. Consulte as [issues abertas](../../issues).
3. Para mudanças grandes, abra uma issue para discutir a proposta.
4. Envie suas alterações por Pull Request.

Use os modelos de issue para reportar bugs e sugerir funcionalidades. Leia também [Código de Conduta](CODE_OF_CONDUCT.md) e [Política de Segurança](SECURITY.md).

Consulte [Sistema de plugins](docs/PLUGINS.md) para o contrato de extensões e os limites de segurança desta pré-alpha.

O GameHub não varre a pasta protegida WindowsApps nem altera permissões. Alguns jogos Xbox empacotados exigem ativação pelo Windows e ainda precisam de um adaptador específico.

## Próximos marcos

1. Testes de restauração com falhas simuladas e validação física do Save Center.
2. Validar o adaptador TOTK Recomp com instalação real e confirmar `TOTK_PROFILE_DIR`.
3. Testar instalação, inicialização, atualização manual e remoção em Windows limpo.
4. Gerar e versionar lockfiles para builds reproduzíveis.
5. Expandir conectores do GameHub e melhorar operações em lote no File Manager.
6. Expandir monitoramento de GPU e Toolbox com ferramentas adicionais verificáveis.

## Licença

Este projeto é distribuído sob a licença [MIT](LICENSE).
