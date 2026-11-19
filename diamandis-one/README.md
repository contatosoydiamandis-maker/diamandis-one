# DIAMANDIS ONE

Centro de organização, utilidades e gerenciamento gamer para Windows.

## Stack
- React + TypeScript + Vite
- Tauri 2
- Rust
- SQLite (planejado para a camada de persistência)

## Requisitos
- Node.js LTS
- Rust (stable) com target MSVC
- Microsoft C++ Build Tools
- WebView2 Runtime
- Windows 10/11

## Desenvolvimento

```bash
npm install
npm run tauri dev
```

## Build Windows

```bash
npm run tauri build
```

O instalador será gerado em `src-tauri/target/release/bundle/`.

## Estado da versão 0.1.0
- [x] Estrutura inicial React/TypeScript + Tauri 2
- [x] Navegação modular
- [x] Dashboard inicial
- [x] Identidade visual NEXUS
- [x] Comando Rust de diagnóstico
- [ ] Persistência SQLite
- [ ] File Manager funcional
- [ ] Toolbox funcional
- [ ] Conectores GameHub

## Segurança
A versão inicial não apaga, move ou altera arquivos do usuário. Operações destrutivas só serão implementadas com pré-visualização, confirmação explícita e logs.
