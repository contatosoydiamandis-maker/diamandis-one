# Checklist de qualidade para releases

Esta lista é obrigatória antes de promover qualquer build para uma versão estável. Registre links de CI, hashes e evidências de teste no PR ou na Release.

## 1. Código e dependências

- [ ] `package-lock.json` e `src-tauri/Cargo.lock` versionados.
- [ ] CI usa `npm ci` e comandos Cargo com `--locked`.
- [ ] Dependências diretas e transitivas revisadas; nenhuma atualização automática sem validação.
- [ ] Nenhuma chave privada, token, segredo ou dado pessoal no repositório ou pacote.
- [ ] Permissões Tauri revisadas e mínimas.

## 2. Integração contínua

- [ ] TypeScript e build de produção aprovados.
- [ ] Testes Rust concluídos sem falhas em todos os targets aplicáveis.
- [ ] Build Tauri e instalador NSIS x64 concluídos.
- [ ] Artefato conferido: nome, versão, arquitetura, tamanho e SHA-256.
- [ ] CI executada sobre o commit exato que será publicado.

## 3. Instalação limpa

Executar em VM ou computador de teste sem ferramentas de desenvolvimento:

- [ ] Instalar sem Node.js, npm, Rust, Cargo, CMD ou PowerShell.
- [ ] Instalação por usuário funciona sem exigir administrador.
- [ ] Bootstrapper WebView2 funciona e apresenta erros compreensíveis sem internet.
- [ ] Atalho do Menu Iniciar abre o aplicativo.
- [ ] Aplicativo inicia sem janela de terminal.
- [ ] Nome, ícone, versão e editor exibidos pelo Windows estão corretos.

## 4. Uso e estabilidade

- [ ] Dashboard e navegação entre módulos funcionam.
- [ ] File Manager navega, filtra, pagina e trata diretórios grandes.
- [ ] Pastas protegidas ou inexistentes geram mensagens compreensíveis.
- [ ] Operações de arquivos não sobrescrevem destinos nem seguem links simbólicos.
- [ ] GameHub detecta corretamente as bibliotecas suportadas sem alterar instalações.
- [ ] Save Center cria, verifica e restaura backups; falhas simuladas não destroem o save original.
- [ ] Preferências e dados persistem após reiniciar o aplicativo.
- [ ] TOTK Recomp é validado com uma instalação real antes de anunciar compatibilidade.

## 5. Atualização e remoção

- [ ] Atualizar a partir da versão anterior preserva dados do usuário.
- [ ] Falha durante atualização não deixa o aplicativo inutilizável.
- [ ] Desinstalar pelo Windows remove binários e atalhos.
- [ ] Configurações, backups e arquivos pessoais seguem política explícita de preservação.
- [ ] Atualização automática permanece desativada até assinatura e verificação criptográfica estarem configuradas.

## 6. Desempenho e privacidade

- [ ] Uso de memória em repouso medido e registrado.
- [ ] Tempo de inicialização medido em máquina de referência.
- [ ] Nenhuma telemetria ou conexão externa não documentada.
- [ ] Nenhum processo auxiliar permanece após fechar o aplicativo, salvo comportamento documentado.
- [ ] Operações destrutivas exigem prévia, confirmação e recuperação apropriada.

## Critério de publicação

Uma Release estável só pode sair quando todos os itens aplicáveis forem verificados. Builds de CI e pré-releases são artefatos de teste, não garantias de estabilidade.
