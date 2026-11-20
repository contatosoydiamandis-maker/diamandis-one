# Política de segurança

O DIAMANDIS ONE está em desenvolvimento pré-alpha. Correções de segurança são tratadas prioritariamente na branch padrão (`main`).

## Reportar uma vulnerabilidade

**Não publique detalhes exploráveis em issues públicas, discussões ou Pull Requests.**

Quando o recurso estiver disponível, use a opção **Report a vulnerability** na aba Security do repositório. Informe:

- descrição objetiva e impacto potencial;
- passos para reproduzir ou prova de conceito não destrutiva;
- versão, commit, sistema operacional e configuração afetados;
- possíveis medidas de mitigação.

Aguarde um prazo razoável para investigação e correção antes de divulgar publicamente.

Nunca inclua senhas, tokens, chaves privadas, arquivos pessoais ou outros dados sensíveis no relato.

## Escopo especialmente sensível

Como o aplicativo gerencia arquivos e backups locais, relatos envolvendo traversal de caminhos, links simbólicos, sobrescrita acidental, restauração de backup, execução de processos, permissões Tauri e exposição de dados locais são especialmente importantes.
