# DIAMANDIS ONE

Centro local-first para organização de arquivos, utilitários e gerenciamento gamer no Windows.

## Versão inicial: 0.1.0

A fundação do projeto foi preparada em React + TypeScript + Vite e Tauri 2 + Rust. O pacote-fonte inicial foi entregue nesta conversa como `diamandis-one-v0.1.0.zip`.

> Este repositório foi inicializado com este README. O código-fonte ainda precisa ser enviado a partir do pacote ZIP local.

## Importar o código-fonte para este repositório

1. Baixe e extraia `diamandis-one-v0.1.0.zip`.
2. Clone este repositório.
3. Copie o conteúdo da pasta extraída `diamandis-one/` para a pasta clonada, preservando os diretórios.
4. No terminal da pasta clonada, execute:

```powershell
git add .
git commit -m "chore: add Diamandis One v0.1.0 source"
git push origin main
```

## Stack
- React + TypeScript + Vite
- Tauri 2 + Rust
- SQLite planejado para a próxima etapa

## Executar no Windows
Requisitos: Node.js LTS, Rust stable/MSVC, Microsoft C++ Build Tools e WebView2.

```powershell
npm install
npm run tauri dev
```

Gerar instalador:

```powershell
npm run tauri build
```

## Escopo 0.1.0
- Dashboard e navegação modular
- Identidade visual NEXUS
- Núcleo Rust com comando de diagnóstico
- Estrutura preparada para File Manager, Toolbox, GameHub e Save Center

## Segurança
Esta versão inicial não executa exclusões nem alterações em arquivos do usuário. Operações destrutivas futuras exigirão pré-visualização, confirmação e registro.
