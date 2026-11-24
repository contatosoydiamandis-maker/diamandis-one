# Sistema de plugins

## Estado da pré-alpha

A API de extensões está em preparação. Esta versão implementa o contrato de manifesto, validação de compatibilidade e um registro em memória para metadados. **Ela ainda não instala, carrega nem executa código externo.** Essa limitação é intencional: executar extensões arbitrárias antes de existir isolamento e consentimento explícito seria incompatível com o modelo de segurança do DIAMANDIS ONE.

## Manifesto API v1

Exemplo:

~~~json
{
  "id": "com.example.sample",
  "name": "Plugin de exemplo",
  "version": "1.0.0",
  "apiVersion": 1,
  "description": "Uma extensão local de exemplo.",
  "permissions": ["ui.panel", "tool.action"]
}
~~~

Regras:
- id: identificador minúsculo estável, com números, pontos e hífens.
- version: versão semântica MAJOR.MINOR.PATCH.
- apiVersion: precisa corresponder à API suportada pelo aplicativo.
- permissions: lista explícita, sem duplicatas, limitada às capacidades reconhecidas.
- IDs duplicados são recusados pelo registro.

## Capacidades declaradas

- ui.panel: solicitar um ponto de entrada visual.
- tool.action: registrar uma ferramenta declarativa.
- filesystem.read: solicitar leitura de arquivos.
- filesystem.write: solicitar escrita de arquivos.
- process.launch: solicitar inicialização de processos.

A declaração não concede acesso automaticamente. Cada capacidade deverá passar por consentimento, validação no backend e política de escopo. A UI atual permite validar um manifesto JSON, mas não concede permissões.

## Próximas etapas

1. Persistência local do catálogo e das aprovações por plugin.
2. Tela para instalar, atualizar, desativar e remover extensões.
3. Modelo de execução isolado, sem eval ou carregamento arbitrário no contexto principal.
4. API de mensagens com esquemas versionados e validação no backend.
5. Auditoria de permissões e testes de isolamento antes de habilitar plugins de terceiros.
