# DIAMANDIS ONE

Centro local-first para organização de arquivos, utilitários e gerenciamento gamer no Windows.

## Versão inicial: 0.1.0

O pacote-fonte completo desta versão está em [diamandis-one-v0.1.0.zip](./releases/diamandis-one-v0.1.0.zip). Baixe o arquivo e extraia seu conteúdo para obter a estrutura do projeto.

### Stack
- React + TypeScript + Vite
- Tauri 2 + Rust
- SQLite planejado para a próxima etapa

### Executar no Windows
Requisitos: Node.js LTS, Rust stable/MSVC, Microsoft C++ Build Tools e WebView2.

```powershell
npm install
npm run tauri dev
```

Gerar instalador:

```powershell
npm run tauri build
```

### Escopo 0.1.0
- Dashboard e navegação modular
- Identidade visual NEXUS
- Núcleo Rust com comando de diagnóstico
- Estrutura preparada para File Manager, Toolbox, GameHub e Save Center

### Segurança
Esta versão inicial não executa exclusões nem alterações em arquivos do usuário. Operações destrutivas futuras exigirão pré-visualização, confirmação e registro.

Consulte `docs/ARCHITECTURE.md` dentro do pacote-fonte para decisões técnicas.
