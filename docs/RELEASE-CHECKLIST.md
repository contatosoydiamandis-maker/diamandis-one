# Checklist de qualidade para releases

Esta lista é obrigatória antes de promover qualquer build para uma versão estável.

## 1. Integração contínua

- [ ] `package-lock.json` e `Cargo.lock` versionados para instalações reproduzíveis.
- [ ] CI utiliza `npm ci` e comandos Cargo com `--locked` assim que os lockfiles forem adicionados.
- [ ] Build de produção do frontend concluído sem erros.
- [ ] Testes Rust concluídos sem falhas.
- [ ] Instalador NSIS gerado no Windows x64.
- [ ] Artefato conferido: nome, versão, arquitetura e tamanho.
- [ ] Nenhuma chave privada, token ou segredo incluído no repositório ou no pacote.

## 2. Instalação limpa

Executar em uma máquina virtual ou computador de teste sem ferramentas de desenvolvimento instaladas.

- [ ] Instalar sem Node.js, npm, Rust, Cargo ou CMD.
- [ ] Instalação por usuário funciona sem exigir administrador.
- [ ] Se WebView2 estiver ausente, o instalador orienta a instalação e apresenta falhas com clareza.
- [ ] Atalho do Menu Iniciar abre o aplicativo.
- [ ] O aplicativo inicia sem janela de terminal.
- [ ] Nome, ícone, versão e editor exibidos pelo Windows estão corretos.

## 3. Uso e estabilidade

- [ ] Dashboard abre sem erros.
- [ ] Navegação entre módulos funciona.
- [ ] File Manager abre a pasta inicial e navega entre diretórios.
- [ ] Pastas protegidas ou inexistentes geram mensagens compreensíveis.
- [ ] Criação de pastas rejeita nomes inválidos e não sobrescreve itens existentes.
- [ ] Diretórios grandes não deixam a interface permanentemente bloqueada.
- [ ] Encerrar e abrir novamente não corrompe preferências ou dados.

## 4. Atualização e remoção

- [ ] Atualizar a partir da versão anterior preserva dados do usuário.
- [ ] Falha durante atualização não deixa o aplicativo inutilizável.
- [ ] Desinstalar pelo Windows remove binários e atalhos.
- [ ] Configurações, backups e arquivos pessoais seguem uma política explícita de preservação.
- [ ] Atualizações automáticas permanecem desativadas até assinatura e verificação criptográfica estarem configuradas.

## 5. Desempenho e privacidade

- [ ] Uso de memória em repouso medido e registrado.
- [ ] Tempo de inicialização medido em máquina de referência.
- [ ] Nenhuma telemetria ou conexão externa não documentada.
- [ ] Nenhum processo auxiliar permanece aberto após fechar o aplicativo, salvo se explicitamente configurado.
- [ ] Operações destrutivas futuras exigem prévia, confirmação e registro.

## Critério de publicação

Uma Release só pode sair de rascunho quando todos os itens aplicáveis forem verificados. Builds de CI e pré-releases são artefatos de teste, não garantias de estabilidade.
