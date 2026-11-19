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
- Preferences: arquivo JSON em app_config_dir.
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
- A criação de backup copia arquivos e rejeita links simbólicos.
- A restauração prepara uma pasta temporária, verifica contagem e tamanho, preserva o save atual em .diamandis-pre-restore-* e só então troca os diretórios.
- O diálogo nativo de seleção de pasta evita exigir que a pessoa digite caminhos manualmente.
- Hashes criptográficos, detecção de jogo em execução, perfis e adaptadores por jogo ainda são etapas futuras.

## Próximos marcos
- Acrescentar hashes SHA-256 por arquivo e testes de corrupção.
- Implementar perfis/campanhas e o adaptador Zelda TOTK Recomp.
- Adicionar detecção de processos antes de restaurar saves.
- Gerar lockfiles reproduzíveis e validar o instalador em Windows limpo.
