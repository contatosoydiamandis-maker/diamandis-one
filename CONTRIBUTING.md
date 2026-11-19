# Contributing to DIAMANDIS ONE

Obrigado pelo interesse em contribuir! Toda ajuda é bem-vinda, desde correções pequenas até novos módulos.

## Antes de começar

1. Leia o [README](README.md) e a documentação em `docs/`.
2. Confira as [issues abertas](../../issues) para evitar trabalho duplicado.
3. Para mudanças grandes, abra uma issue antes de implementar para alinharmos a proposta.
4. Respeite o escopo atual: aplicativo desktop Windows, local-first, sem anúncios e sem telemetria invasiva.

## Como contribuir

1. Faça um fork do repositório.
2. Crie uma branch a partir de `main`, por exemplo:
   - `feat/nome-da-funcionalidade`
   - `fix/descricao-do-problema`
   - `docs/descricao`
3. Faça alterações pequenas e focadas.
4. Execute os testes e verificações disponíveis.
5. Abra um Pull Request descrevendo o problema, a solução, como testar e possíveis impactos.

## Padrão de commits

Prefira mensagens objetivas no estilo Conventional Commits:

- `feat:` nova funcionalidade
- `fix:` correção
- `docs:` documentação
- `refactor:` refatoração sem mudança funcional
- `test:` testes
- `chore:` manutenção

## Diretrizes técnicas

- Mantenha a separação entre interface React/TypeScript e lógica nativa Rust/Tauri.
- Prefira soluções locais e offline sempre que possível.
- Não introduza telemetria, publicidade ou serviços obrigatórios em nuvem.
- Não apague, mova ou altere arquivos do usuário sem consentimento explícito.
- Operações potencialmente destrutivas devem ter prévia, confirmação e registro.
- Não armazene tokens, senhas, chaves ou dados pessoais nos commits.
- Documente permissões Tauri e mantenha-as no mínimo necessário.
- Adicione ou atualize testes quando alterar comportamento.

## Pull Requests

Um PR deve:

- Ter escopo claro e limitado.
- Explicar o motivo da mudança.
- Informar os passos usados para validar.
- Incluir capturas de tela quando alterar a interface.
- Informar limitações conhecidas e riscos de compatibilidade.

Ainda estamos na fase inicial. Nem todos os módulos, testes ou pipelines estão prontos. Não marque uma validação como aprovada se ela não foi executada.

## Código de conduta

Ao participar, mantenha comunicação respeitosa, construtiva e acolhedora. Assédio, discriminação, ataques pessoais e publicação de dados privados não serão tolerados.
