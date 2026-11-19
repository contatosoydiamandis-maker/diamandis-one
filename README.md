# DIAMANDIS ONE

**Um hub desktop local-first para Windows:** organização de arquivos, ferramentas de produtividade e gerenciamento da biblioteca gamer em um só lugar.

> **Status: pré-alpha 0.2.0.** A interface desktop inclui navegação real de arquivos, operações controladas, preferências persistentes e detecção inicial de bibliotecas Steam, Epic Games e GOG. O Save Center aguarda a fonte do Save Manager original para integração sem duplicação. O frontend e os testes nativos passaram em CI antes da implementação atual do Save Center; a nova implementação está em validação. Instalador e testes físicos ainda precisam de validação manual.

## Visão do projeto

- **Local-first:** dados e operações permanecem no computador do usuário.
- **Privacidade:** sem anúncios, sem telemetria invasiva e sem conta ou nuvem obrigatória.
- **Modular:** Dashboard, File Manager, Toolbox, GameHub, Save Center e Configurações.
- **Seguro:** operações destrutivas exigem confirmação e usam a Lixeira; cópias não sobrescrevem arquivos.
- **Extensível:** conectores independentes para launchers e serviços de jogos.

## Stack

- React + TypeScript + Vite
- Tauri 2 + Rust
- Persistência local em JSON para preferências; SQLite permanece planejado para dados relacionais do GameHub/Save Center.

## Estado atual

| Área | Estado |
|---|---|
| Dashboard e identidade visual | Interface desktop com emblema e ícone do instalador |
| File Manager | Navegação, filtro, criação, renomeação, cópia de arquivos, movimentação e envio à Lixeira |
| Configurações | Tema escuro/claro/sistema, pasta inicial, confirmação de exclusão e última pasta |
| Toolbox | Transformador de texto e formatador JSON offline |
| GameHub | Descoberta somente leitura de jogos Steam, Epic Games e GOG |
| Save Center | Cadastro local de jogos, backup por cópia, histórico e restauração protegida; detecção automática por adaptadores ainda em evolução |
| Dependências | Versões diretas do frontend fixadas; lockfiles reproduzíveis ainda pendentes |
| Build/instalador | Pipeline preparado; não validado em máquina Windows |

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

O comando de build prepara automaticamente o ícone do aplicativo e executa a compilação de produção. Os comandos de terminal pertencem exclusivamente ao desenvolvimento, não à instalação ou ao uso da versão distribuída.

## Como contribuir

1. Leia [CONTRIBUTING.md](CONTRIBUTING.md).
2. Consulte as [issues abertas](../../issues).
3. Para mudanças grandes, abra uma issue para discutir a proposta.
4. Envie suas alterações por Pull Request.

Use os modelos de issue para reportar bugs e sugerir funcionalidades. Leia também [Código de Conduta](CODE_OF_CONDUCT.md) e [Política de Segurança](SECURITY.md).

## Roadmap

1. **0.2:** base desktop, preferências, File Manager seguro e GameHub inicial.
2. **0.3:** paginação, operações em lote com prévia e testes de filesystem.
3. **0.4:** Toolbox ampliada.
4. **0.5:** conectores adicionais de launchers e inicialização de jogos.
5. **0.6:** integrar o Save Manager existente ao Save Center.
6. **0.7:** testes completos, otimização, acessibilidade e instalador Windows validado.

## Licença

Este projeto é distribuído sob a licença [MIT](LICENSE).
