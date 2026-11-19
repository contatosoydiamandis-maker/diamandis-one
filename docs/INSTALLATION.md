# Instalação e uso do DIAMANDIS ONE

## Para quem vai usar o aplicativo

O pacote só será promovido a estável depois de passar pelo [checklist de qualidade para releases](RELEASE-CHECKLIST.md).

A distribuição oficial do DIAMANDIS ONE será feita por um instalador gráfico para Windows (`.exe`).

A experiência pretendida é:

1. Baixar o instalador na página **Releases** do GitHub.
2. Abrir o arquivo baixado com dois cliques.
3. Seguir as etapas do assistente de instalação.
4. Abrir o DIAMANDIS ONE pelo atalho criado no Menu Iniciar (e na área de trabalho, quando selecionado).
5. Usar o aplicativo normalmente, sem abrir terminal ou instalar ferramentas de desenvolvimento.

**Não será necessário instalar Node.js, Rust, npm ou usar CMD/PowerShell para instalar ou executar a versão distribuída.**

> Enquanto o projeto estiver em pré-alpha, os instaladores de teste não devem ser considerados versões estáveis. A disponibilidade de um instalador em uma Release não significa, por si só, que ele passou por todos os testes de compatibilidade.

## Requisitos da versão distribuída

- A instalação padrão será **somente para o usuário atual**, dentro da pasta de perfil do Windows, evitando solicitar privilégios de administrador sem necessidade.
- O instalador usará o bootstrapper do WebView2. Se o runtime estiver ausente, será necessário acesso à internet para obtê-lo; o instalador deverá informar claramente qualquer falha nessa etapa.

- Windows 10 ou Windows 11 de 64 bits, conforme a matriz de compatibilidade validada para cada versão.
- WebView2 Runtime. Em sistemas Windows compatíveis ele normalmente já está presente; o instalador e a documentação deverão orientar o usuário caso seja necessário instalá-lo.
- Permissão de instalação adequada ao modo escolhido pelo usuário.

O objetivo é evitar dependências manuais e orientar claramente quando algum componente do Windows estiver ausente.

## Atualizações e remoção

A distribuição deve oferecer um caminho simples para obter novas versões. A atualização automática dentro do aplicativo só será ativada depois de configurar assinatura criptográfica, publicação e verificação segura dos pacotes. O instalador de teste será disponibilizado como pré-lançamento e não será promovido a estável sem testes de instalação, atualização, execução e remoção em Windows limpo.

O aplicativo deverá poder ser removido pelo caminho **Configurações do Windows → Aplicativos → Aplicativos instalados**, sem exigir comandos. Dados pessoais, configurações e backups deverão ter política de preservação explícita, sem apagar arquivos do usuário silenciosamente.

## Para desenvolvedores e contribuidores

Os comandos de terminal pertencem exclusivamente ao fluxo de desenvolvimento. Eles não fazem parte da instalação nem do uso cotidiano por usuários finais.

Consulte [CONTRIBUTING.md](../CONTRIBUTING.md) para preparar o ambiente de desenvolvimento e compilar o projeto.
