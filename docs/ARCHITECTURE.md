# Arquitetura inicial

## Princípios
1. Local-first: dados e operações permanecem na máquina do usuário.
2. Modularidade: cada domínio possui UI, serviços e comandos próprios.
3. Segurança: operações destrutivas exigem prévia, confirmação e registro.
4. Conectores isolados: launchers não contaminam o núcleo.
5. Persistência versionada: migrações explícitas e backups antes de alterações.

## Camadas
- UI: React + TypeScript.
- Desktop bridge: Tauri IPC.
- Core: Rust.
- Persistence: SQLite (próxima etapa).
- Integrations: adaptadores independentes para launchers.

## Próximos marcos
- Criar camada SQLite e migrations.
- Definir contratos TypeScript/Rust para File Manager.
- Implementar scanner read-only.
- Implementar Toolbox sem permissões de filesystem desnecessárias.
- Implementar descoberta de jogos e launchers.