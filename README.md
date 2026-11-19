# DIAMANDIS ONE

**Um hub desktop local-first para Windows:** organização de arquivos, ferramentas de produtividade e gerenciamento da biblioteca gamer em um só lugar.

> **Status: pré-alpha 0.2.1.** A interface desktop inclui navegação real de arquivos, operações controladas, preferências persistentes, descoberta inicial de jogos Steam/Epic/GOG e um Save Center funcional em seu primeiro MVP. O Save Center permite cadastrar pastas, criar backups locais e restaurá-los preservando o estado anterior. Seleção de pastas nativa já integrada. Adaptadores automáticos, perfis de campanha e verificação criptográfica ainda estão em desenvolvimento. CI verde cobre o frontend e os testes nativos; instalador e testes físicos ainda precisam de validação manual.

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
| Dashboard e identidade visual | Interface desktop com emblema e ícone do instalador |
| File Manager | Navegação, filtro, criação, renomeação, cópia, movimentação e Lixeira |
| Configurações | Tema escuro/claro/sistema e preferências persistentes |
| Toolbox | Transformador de texto e formatador JSON offline |
| GameHub | Descoberta somente leitura de jogos Steam, Epic Games e GOG |
| Save Center | Cadastro manual de pastas, backups locais, histórico e restauração protegida |
| Seleção de pastas | Diálogo nativo do Windows via Tauri |
| Build/instalador | Pipeline preparado; ainda não validado em máquina Windows limpa |

## Instalação para usuários

A versão distribuída será instalada por um assistente gráfico do Windows. Usuários finais não precisarão instalar Node.js ou Rust, nem abrir CMD ou PowerShell. Consulte [Instalação e uso](docs/INSTALLATION.md).

O workflow de instalador pode ser executado pela aba **Actions** do GitHub ou por uma tag v*. Os pacotes gerados são artefatos de teste; nenhuma versão será considerada estável sem testes reais em Windows limpo.

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

## Próximos marcos

1. Integridade SHA-256 e testes de restauração com falhas simuladas.
2. Perfis/campanhas e adaptador dedicado ao Zelda TOTK Recomp.
3. Detecção de processos para bloquear restauração enquanto o jogo estiver aberto.
4. Paginação e operações em lote no File Manager.
5. Instalador validado em Windows limpo.

## Licença

Este projeto é distribuído sob a licença [MIT](LICENSE).
