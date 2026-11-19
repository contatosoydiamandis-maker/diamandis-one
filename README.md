# DIAMANDIS ONE

**Um hub desktop local-first para Windows:** organização de arquivos, ferramentas de produtividade e gerenciamento da biblioteca gamer em um só lugar.

> **Status: início do desenvolvimento.** A versão 0.1.0 é uma fundação visual e estrutural. Os módulos File Manager, Toolbox, GameHub e Save Center ainda estão em planejamento/implementação. O projeto ainda não tem uma versão estável nem um instalador validado.

## Visão do projeto

- **Local-first:** dados e operações devem permanecer no computador do usuário.
- **Privacidade:** sem anúncios, sem telemetria invasiva e sem conta ou nuvem obrigatória.
- **Modular:** Dashboard, File Manager, Toolbox, GameHub, Save Center e Configurações.
- **Seguro:** nenhuma operação destrutiva automática. Alterações futuras em arquivos exigirão pré-visualização, confirmação e registro.
- **Extensível:** conectores independentes para launchers e serviços de jogos.

## Stack

- React + TypeScript + Vite
- Tauri 2 + Rust
- SQLite planejado para persistência local

## Estado atual

| Área | Estado |
|---|---|
| Dashboard e identidade visual | Fundação inicial |
| Navegação modular | Estrutura inicial |
| Núcleo Rust e diagnóstico | Estrutura inicial |
| File Manager | Planejado |
| Toolbox | Planejado |
| GameHub e conectores | Planejado |
| Save Center | Planejado, aproveitando o Save Manager existente |
| SQLite e migrações | Planejado |
| Testes automatizados e instalador | Ainda não validados |

## Executar no Windows

### Requisitos

- Windows 10/11
- Node.js LTS
- Rust stable com toolchain MSVC
- Microsoft C++ Build Tools
- WebView2 Runtime

### Desenvolvimento

```powershell
npm install
npm run tauri dev
```

### Build

```powershell
npm run tauri build
```

O build e o instalador ainda precisam ser executados e validados em ambiente Windows. Não considere a versão pronta para distribuição até essa validação.

## Como contribuir

Contribuições são bem-vindas! Antes de começar:

1. Leia [CONTRIBUTING.md](CONTRIBUTING.md).
2. Consulte as [issues abertas](../../issues).
3. Para mudanças grandes, abra uma issue para discutir a proposta.
4. Envie suas alterações por Pull Request.

Use os modelos de issue para reportar bugs e sugerir funcionalidades. Leia também nosso [Código de Conduta](CODE_OF_CONDUCT.md) e a [Política de Segurança](SECURITY.md).

## Roadmap inicial

1. **0.1:** fundação, navegação e identidade visual.
2. **0.2:** dashboard e configurações.
3. **0.3:** File Manager com operações seguras.
4. **0.4:** Toolbox.
5. **0.5:** GameHub e descoberta de launchers.
6. **0.6:** Save Center integrado ao Save Manager existente.
7. **0.7:** testes, otimização e instalador Windows.

O roadmap pode mudar conforme feedback e contribuições da comunidade.

## Licença

Este projeto é distribuído sob a licença [MIT](LICENSE).
