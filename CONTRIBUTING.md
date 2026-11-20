# Como contribuir com o DIAMANDIS ONE

Obrigado por considerar contribuir! Correções, documentação, testes e novos módulos são bem-vindos.

## Antes de começar

1. Leia o [README](README.md), a [arquitetura](docs/ARCHITECTURE.md) e a documentação em `docs/`.
2. Consulte as [issues abertas](../../issues) para evitar trabalho duplicado.
3. Para mudanças grandes, abra uma issue antes de implementar.
4. Preserve o escopo do produto: desktop, local-first, sem anúncios e sem telemetria invasiva.

## Ambiente de desenvolvimento

- Windows 10/11 de 64 bits.
- Node.js 22 ou superior e npm 10 ou superior.
- Rust stable com toolchain MSVC.
- Microsoft C++ Build Tools e WebView2 Runtime.

Comandos principais:

```powershell
npm install
npm run dev
```

Para verificar TypeScript e gerar o build de produção:

```powershell
npm run check
```

Para executar testes nativos e compilar o aplicativo:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
npm run tauri build
```

O comando de build prepara automaticamente os assets do Tauri. Não é necessário executar o script de preparação manualmente antes dele.

## Fluxo de contribuição

1. Faça um fork do repositório.
2. Crie uma branch a partir de `main`:
   - `feat/nome-da-funcionalidade`
   - `fix/descricao-do-problema`
   - `docs/descricao`
   - `chore/descricao`
3. Faça alterações pequenas e focadas.
4. Atualize testes e documentação quando o comportamento mudar.
5. Execute as verificações aplicáveis.
6. Abra um Pull Request com problema, solução, validação e limitações conhecidas.

## Padrão de commits

Use mensagens objetivas no estilo Conventional Commits:

- `feat:` nova funcionalidade
- `fix:` correção
- `docs:` documentação
- `refactor:` refatoração sem mudança funcional
- `test:` testes
- `chore:` manutenção

## Diretrizes técnicas e de segurança

- Mantenha a separação entre interface React/TypeScript e lógica nativa Rust/Tauri.
- Prefira operações locais e offline.
- Não introduza telemetria, publicidade ou serviços obrigatórios em nuvem.
- Nunca altere ou exclua arquivos do usuário sem consentimento explícito.
- Operações destrutivas devem ter validação, prévia, confirmação e preservação recuperável sempre que possível.
- Não armazene tokens, senhas, chaves ou dados pessoais em commits, logs ou screenshots.
- Mantenha as permissões Tauri no mínimo necessário.
- Trate caminhos recebidos da interface como entrada não confiável.
- Adicione testes para regras de negócio, validações e operações de arquivos.
- Não declare uma validação como aprovada se ela não foi executada.

## Pull Requests

Todo PR deve ter escopo claro, explicar o motivo da mudança, listar como foi testado e apontar riscos ou limitações. Mudanças de interface devem incluir capturas de tela quando possível.

Leia também o [Código de Conduta](CODE_OF_CONDUCT.md) e a [Política de Segurança](SECURITY.md).
