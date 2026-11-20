# Instalação e uso do DIAMANDIS ONE

## Estado da distribuição

O DIAMANDIS ONE está em pré-alpha. O instalador NSIS x64 produzido pelo CI é um artefato de teste e não deve ser tratado como versão estável. A aprovação do build automatizado não substitui testes de instalação e execução em um Windows limpo.

Enquanto não houver uma Release publicada, baixe somente os artefatos vinculados à execução de CI indicada pelo mantenedor. Artefatos de Actions podem expirar.

## Instalação para usuários

A experiência de distribuição pretendida é totalmente gráfica:

1. Baixar o instalador `.exe` publicado na página Releases.
2. Abrir o arquivo com dois cliques.
3. Seguir o assistente de instalação.
4. Iniciar o DIAMANDIS ONE pelo Menu Iniciar ou atalho criado.
5. Usar o aplicativo sem abrir terminal ou instalar ferramentas de desenvolvimento.

Não será necessário instalar Node.js, Rust, npm, CMD ou PowerShell para usar a versão distribuída.

## Requisitos

- Windows 10 ou Windows 11 de 64 bits, conforme a matriz validada para a versão.
- WebView2 Runtime. O instalador está configurado para usar o bootstrapper do WebView2; o download exige conexão com a internet quando o runtime não estiver presente.
- Permissão de instalação adequada ao modo por usuário.

## Atualização e remoção

A atualização automática dentro do aplicativo permanece desativada até existir assinatura criptográfica e verificação segura dos pacotes. Use somente instaladores publicados pelo repositório oficial.

A remoção deve ser feita por **Configurações do Windows → Aplicativos → Aplicativos instalados**. Configurações, backups e arquivos pessoais devem seguir política de preservação explícita; nunca devem ser apagados silenciosamente.

## Desenvolvimento

Os comandos de terminal pertencem exclusivamente ao fluxo de desenvolvimento:

```powershell
npm install
npm run dev
npm run check
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
npm run tauri build
```

O comando de build prepara automaticamente os assets do Tauri. Consulte [Como contribuir](../CONTRIBUTING.md) para preparar o ambiente.
