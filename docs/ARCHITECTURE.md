# Arquitetura do DIAMANDIS ONE

## Princípios
1. Local-first: dados e operações permanecem na máquina do usuário.
2. Modularidade: cada domínio possui UI, serviços e comandos próprios.
3. Segurança: operações destrutivas exigem confirmação, prévia quando aplicável e registro.
4. Conectores isolados: launchers e o Save Manager existente não contaminam o núcleo.
5. Persistência versionada: configurações simples em JSON; SQLite para dados relacionais quando o GameHub/Save Center exigirem.

## Camadas
- UI: React + TypeScript.
- Desktop bridge: Tauri IPC.
- Core: Rust.
- Preferences: arquivo JSON em app_config_dir, com defaults e validação.
- File Manager: leitura de diretórios e operações explícitas; exclusão via Lixeira do sistema.
- GameHub: scanners somente leitura por launcher; sem iniciar processos automaticamente.
- Save Center: adaptador para o Save Manager existente. Não manter uma segunda implementação de backup.

## Segurança do File Manager
- Renomear valida nomes reservados e caracteres proibidos do Windows.
- Copiar cria o destino com create_new, impedindo sobrescrita.
- Mover usa rename do filesystem e não tenta fallback destrutivo entre volumes.
- Excluir envia o item à Lixeira através da crate trash; não usa remoção permanente.
- Links simbólicos são recusados nas operações de alteração.

## GameHub inicial
- Steam: bibliotecas e manifests appmanifest.
- Epic Games: manifests .item.
- GOG: diretórios de instalação conhecidos.
- Descoberta é local e somente leitura. Xbox, EA e Ubisoft exigem conectores específicos e testes próprios.

## Save Center
src/modules/save-center/adapter.ts define a fronteira de integração. O adaptador real deve ser implementado somente após recuperar a fonte original do Save Manager, preservando seu formato de dados e rotinas de backup/restauração.

## Próximos marcos
- Gerar e versionar package-lock.json e Cargo.lock para builds reproduzíveis.
- Adicionar testes de integração do filesystem em Windows.
- Paginar diretórios grandes e evitar bloquear a UI.
- Integrar o repositório original do Save Manager.
- Expandir scanners GameHub com fixtures e conectores testados.


## Save Center

O Save Center mantém um catálogo local de jogos e caminhos de salvamento em `app_config_dir/save-center/games.json`. Os backups são cópias independentes em `app_local_data_dir/save-center/backups`, organizadas por jogo e identificador. Cada backup registra data, origem, quantidade de arquivos e bytes copiados. Links simbólicos são rejeitados durante a cópia. A restauração prepara uma cópia temporária, confere contagem e tamanho, move o estado atual para uma pasta `.diamandis-pre-restore-*` ao lado do save e só então coloca a versão restaurada no caminho original. Se a troca falhar, o sistema tenta devolver o estado anterior. O primeiro MVP usa caminhos cadastrados manualmente; detecção automática e adaptadores específicos por jogo serão adicionados sem alterar esse formato-base.
