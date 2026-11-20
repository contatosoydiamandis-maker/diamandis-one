# Arquitetura do DIAMANDIS ONE

## Princípios
1. Local-first: dados e operações permanecem na máquina do usuário.
2. Modularidade: cada domínio possui UI, serviços e comandos próprios.
3. Segurança: operações destrutivas exigem confirmação e preservação do estado anterior.
4. Conectores isolados: launchers e adaptadores de jogos não contaminam o núcleo.
5. Persistência simples até haver necessidade real de banco relacional.

## Camadas
- UI: React + TypeScript.
- Desktop bridge: Tauri IPC e plugins de capacidade mínima.
- Core: Rust.
- Preferences: arquivo JSON em app_config_dir, com escrita temporária, sincronização em disco e substituição atômica compatível com Windows.
- Theme: preferência escuro/claro/sistema; o modo sistema acompanha mudanças de aparência do Windows durante a execução.
- File Manager: leitura de diretórios e operações explícitas; exclusão via Lixeira do sistema.
- GameHub: scanners somente leitura por launcher.
- Save Center: catálogo local, cópias de segurança e restauração transacional por diretório.

## Segurança do File Manager
- Renomear valida nomes reservados e caracteres proibidos do Windows.
- Copiar cria o destino sem sobrescrever arquivos existentes.
- Mover usa rename do filesystem e não tenta fallback destrutivo entre volumes.
- Excluir envia o item à Lixeira através da crate trash.
- Links simbólicos são recusados nas operações de alteração.

## GameHub inicial
- Steam: bibliotecas e manifests appmanifest.
- Epic Games: manifests .item.
- GOG: diretórios de instalação conhecidos.
- Descoberta é local e somente leitura. Xbox, EA e Ubisoft exigem conectores específicos e testes próprios.

## Save Center
- O catálogo inicial fica em app_local_data_dir/save-center/games.json.
- Backups ficam em app_local_data_dir/save-center/backups/<game-id>/<backup-id>.
- O MVP permite cadastrar qualquer pasta existente, sem modificar seu conteúdo.
- A criação de backup copia arquivos, rejeita links simbólicos e grava hashes SHA-256 por arquivo.
- A restauração prepara uma pasta temporária, verifica contagem, tamanho e hashes, preserva o save atual em .diamandis-pre-restore-* e só então troca os diretórios.
- Backups antigos sem hashes continuam legíveis e usam a verificação estrutural anterior.
- O diálogo nativo de seleção de pasta evita exigir que a pessoa digite caminhos manualmente.
- As operações de cadastro, remoção, backup, restauração e gerenciamento de campanhas TOTK usam um lock de processo para impedir concorrência entre mutações.
- Catálogo e configurações são gravados por substituição atômica; tamanho e quantidade de arquivos são medidos novamente no payload final antes da verificação.
- A inicialização de campanha exige backup preventivo no backend antes de criar o processo do jogo.

## Adaptador Zelda: TOTK Recomp
- Configuração validada pela presença de `totk.exe`.
- Importação copia `profile` para uma campanha gerenciada e não altera a origem.
- Campanhas ficam sob `app_local_data_dir/totk-recomp/campaigns`.
- A inicialização passa `TOTK_PROFILE_DIR` e exige backup preventivo no backend.
- Backup/restauração de campanha são bloqueados quando `totk.exe` está ativo.
- A compatibilidade de `TOTK_PROFILE_DIR` ainda depende de validação com a build real do Recomp.

## Próximos marcos
- Validar o adaptador com uma instalação real do TOTK Recomp.
- Executar testes físicos de instalação, execução, atualização e remoção em Windows limpo.
- Versionar lockfiles para builds reproduzíveis.
