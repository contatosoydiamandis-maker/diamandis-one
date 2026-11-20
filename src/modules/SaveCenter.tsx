import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Archive, CheckCircle2, Clock3, FolderOpen, HardDrive, LoaderCircle, Plus, RefreshCw, RotateCcw, ShieldCheck } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

type SaveGame = { id: string; name: string; savePath: string; addedAt: string };
type TotkCampaign = { id: string; name: string; path: string; createdAt: string };
type SaveBackup = { id: string; gameId: string; gameName: string; createdAt: string; sourcePath: string; sizeBytes: number; fileCount: number; fileHashes: Record<string, string>; backupPath: string };

const formatSize = (bytes: number) => {
  if (bytes < 1024) return `${bytes} B`;
  const units = ["KB", "MB", "GB", "TB"];
  let value = bytes / 1024;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) { value /= 1024; unit++; }
  return `${value.toFixed(value >= 100 ? 0 : 1)} ${units[unit]}`;
};
const formatDate = (value: string) => {
  const numeric = Number(value);
  const date = Number.isFinite(numeric) ? new Date(numeric) : new Date(value);
  return Number.isNaN(date.getTime()) ? "Data indisponível" : date.toLocaleString("pt-BR");
};

export default function SaveCenter() {
  const [games, setGames] = useState<SaveGame[]>([]);
  const [backups, setBackups] = useState<SaveBackup[]>([]);
  const [selectedGame, setSelectedGame] = useState("");
  const selectedGameRef = useRef("");
  const [name, setName] = useState("");
  const [savePath, setSavePath] = useState("");
  const [adding, setAdding] = useState(false);
  const [busy, setBusy] = useState(false);
  const [workingId, setWorkingId] = useState("");
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [totkInstall, setTotkInstall] = useState("");
  const [totkName, setTotkName] = useState("");
  const [totkCampaigns, setTotkCampaigns] = useState<TotkCampaign[]>([]);
  const [totkRunning, setTotkRunning] = useState(false);
  const [totkBusy, setTotkBusy] = useState(false);

  const refresh = useCallback(async (gameId?: string) => {
    setBusy(true); setError("");
    try {
      const nextGames = await invoke<SaveGame[]>("list_save_games");
      const activeGameId = gameId || (
        nextGames.some(game => game.id === selectedGameRef.current)
          ? selectedGameRef.current
          : nextGames[0]?.id || ""
      );
      const nextBackups = activeGameId
        ? await invoke<SaveBackup[]>("list_save_backups", { gameId: activeGameId })
        : [];
      setGames(nextGames);
      setBackups(nextBackups);
      selectedGameRef.current = activeGameId;
      setSelectedGame(activeGameId);
    } catch (e) { setError(String(e)); }
    finally { setBusy(false); }
  }, []);

  const refreshTotk = useCallback(async () => {
    try { const [campaigns, running, install] = await Promise.all([invoke<TotkCampaign[]>("list_totk_campaigns"), invoke<boolean>("is_totk_running"), invoke<string>("get_totk_install")]); setTotkCampaigns(campaigns); setTotkRunning(running); setTotkInstall(install); }
    catch (e) { setError(String(e)); }
  }, []);

  useEffect(() => { void refresh(); void refreshTotk(); }, [refresh, refreshTotk]);

  const selected = useMemo(() => games.find(game => game.id === selectedGame), [games, selectedGame]);

  const browseTotkInstall = async () => {
    try { const selectedPath = await open({ directory: true, multiple: false, title: "Selecione a pasta que contém totk.exe" });
      if (typeof selectedPath === "string") { setTotkBusy(true); const saved = await invoke<string>("configure_totk_install", { installPath: selectedPath }); setTotkInstall(saved); setNotice("Instalação do TOTK Recomp validada."); await refreshTotk(); }
    } catch (e) { setError(String(e)); } finally { setTotkBusy(false); }
  };
  const createTotkCampaign = async (mode: "import" | "empty") => {
    if (!totkName.trim()) return; setTotkBusy(true); setError(""); setNotice("");
    try { const campaign = await invoke<TotkCampaign>(mode === "import" ? "import_totk_profile" : "create_empty_totk_campaign", { name: totkName });
      try { await invoke<SaveGame>("add_save_game", { name: "TOTK Recomp • " + campaign.name, savePath: campaign.path }); } catch (e) { if (!String(e).includes("já cadastrada")) throw e; }
      setTotkName(""); setNotice(mode === "import" ? "Campanha importada por cópia. O profile original permaneceu intacto." : "Campanha vazia criada. Esta build pode exigir um profile válido para inicializar."); await refreshTotk(); await refresh();
    } catch (e) { setError(String(e)); } finally { setTotkBusy(false); }
  };
  const launchTotkCampaign = async (campaign: TotkCampaign) => {
    setTotkBusy(true); setError(""); setNotice("");
    try { let game = games.find(item => item.savePath === campaign.path);
      if (!game) game = await invoke<SaveGame>("add_save_game", { name: "TOTK Recomp • " + campaign.name, savePath: campaign.path });
      const backup = await invoke<SaveBackup>("launch_totk_campaign", { campaignId: campaign.id });
      setNotice("Backup preventivo verificado (" + backup.fileCount + " arquivo(s)). Campanha iniciada."); await refreshTotk(); await refresh(game.id);
    } catch (e) { setError(String(e)); } finally { setTotkBusy(false); }
  };

  const browseSaveFolder = async () => {
    setError("");
    try {
      const selectedPath = await open({ directory: true, multiple: false, title: "Selecione a pasta de salvamento" });
      if (typeof selectedPath === "string") setSavePath(selectedPath);
    } catch (e) { setError(`Não foi possível abrir o seletor de pastas: ${String(e)}`); }
  };

  const addGame = async () => {
    setAdding(true); setError(""); setNotice("");
    try {
      const game = await invoke<SaveGame>("add_save_game", { name, savePath });
      setName(""); setSavePath(""); selectedGameRef.current = game.id;
      setSelectedGame(game.id);
      setNotice("Jogo cadastrado. A pasta original não foi modificada.");
      await refresh(game.id);
    } catch (e) { setError(String(e)); }
    finally { setAdding(false); }
  };

  const createBackup = async () => {
    if (!selected) return;
    setWorkingId("create"); setError(""); setNotice("");
    try {
      const backup = await invoke<SaveBackup>("create_save_backup", { gameId: selected.id });
      setNotice(`Backup concluído: ${backup.fileCount} arquivo(s), ${formatSize(backup.sizeBytes)}.`);
      await refresh(selected.id);
    } catch (e) { setError(String(e)); }
    finally { setWorkingId(""); }
  };

  const restore = async (backup: SaveBackup) => {
    if (!selected) return;
    const accepted = window.confirm(`Restaurar o backup de ${formatDate(backup.createdAt)}? O save atual será preservado em uma pasta de segurança antes da troca.`);
    if (!accepted) return;
    setWorkingId(backup.id); setError(""); setNotice("");
    try {
      const message = await invoke<string>("restore_save_backup", { backupId: backup.id, gameId: selected.id });
      setNotice(message);
    } catch (e) { setError(String(e)); }
    finally { setWorkingId(""); }
  };

  return <section className="save-center-page">
    <div className="save-center-hero">
      <div className="save-center-icon"><HardDrive size={26}/></div>
      <div><span className="tool-kicker">PROTEÇÃO DE PROGRESSO</span><h3>Seus saves, sob seu controle.</h3><p>Cadastre qualquer pasta de salvamento, crie cópias locais e restaure versões anteriores com uma camada extra de segurança.</p></div>
      <div className="save-center-total"><strong>{games.length.toString().padStart(2, "0")}</strong><span>JOGOS</span></div>
    </div>

    <div className="save-center-toolbar">
      <div><span className="tool-kicker">ADAPTADOR ESPECIALIZADO</span><h3>Zelda: TOTK Recomp</h3><p>Campanhas isoladas com TOTK_PROFILE_DIR. O perfil original é somente origem de importação.</p></div>
      <button className="secondary-button" onClick={() => void browseTotkInstall()} disabled={totkBusy}><FolderOpen size={14}/> {totkInstall ? "Alterar instalação" : "Localizar totk.exe"}</button>
    </div>
    <div className="save-safety-strip"><ShieldCheck size={15}/><span>{totkInstall ? "Instalação validada: " + totkInstall : "Selecione a pasta que contém totk.exe para ativar o adaptador."} {totkRunning ? "O jogo está em execução. Alterações de campanha estão bloqueadas." : "Ao iniciar, o DIAMANDIS ONE cria um backup preventivo."}</span></div>
    <div className="save-add-form">
      <div className="save-panel-heading"><strong>Nova campanha</strong><span>Importar ou começar do zero</span></div>
      <label>Nome da campanha<input value={totkName} onChange={e => setTotkName(e.target.value)} placeholder="Ex.: Principal, 100%, Sem glitches"/></label>
      <div className="save-form-actions"><button className="secondary-button" onClick={() => void createTotkCampaign("import")} disabled={!totkInstall || !totkName.trim() || totkBusy || totkRunning}><Archive size={14}/> Importar profile atual</button><button className="primary-button compact" onClick={() => void createTotkCampaign("empty")} disabled={!totkInstall || !totkName.trim() || totkBusy || totkRunning}><Plus size={13}/> Criar vazia</button></div>
      <small>Importar copia a pasta profile da instalação. Criar vazia não garante que todas as builds inicializem o perfil automaticamente.</small>
    </div>
    <div className="save-history-heading"><div><h4>Campanhas TOTK Recomp</h4><p>{totkCampaigns.length} campanha(s) • cada uma usa uma pasta independente</p></div><button className="secondary-button" onClick={() => void refreshTotk()} disabled={totkBusy}><RefreshCw size={14}/></button></div>
    {totkCampaigns.length ? <div className="save-backup-list">{totkCampaigns.map(campaign => <article className="save-backup-row" key={campaign.id}><div className="save-backup-icon"><Archive size={16}/></div><div className="save-backup-info"><strong>{campaign.name}</strong><span>{formatDate(campaign.createdAt)} • Perfil isolado</span><small>{campaign.path}</small></div><button className="primary-button compact" onClick={() => void launchTotkCampaign(campaign)} disabled={totkBusy || totkRunning}>Jogar</button></article>)}</div> : <div className="save-empty compact-empty"><Archive size={22}/><strong>Nenhuma campanha TOTK</strong><span>Valide a instalação e importe seu profile ou crie uma campanha vazia.</span></div>}
    <div className="save-center-toolbar">
      <div><span className="tool-kicker">BIBLIOTECA DE SAVES</span><h3>Jogos cadastrados</h3><p>Compatibilidade universal por pasta. Integrações automáticas serão adicionadas por adaptadores.</p></div>
      <button className="secondary-button" onClick={() => void refresh(selectedGame || undefined)} disabled={busy}><RefreshCw size={14} className={busy ? "spin" : ""}/> Atualizar</button>
    </div>

    {error && <div className="file-message error-message">{error}</div>}
    {notice && <div className="save-feedback"><CheckCircle2 size={15}/><span>{notice}</span></div>}

    <div className="save-center-layout">
      <div className="save-games-panel">
        {games.length ? games.map(game => <button key={game.id} className={selectedGame === game.id ? "save-game-row active" : "save-game-row"} onClick={() => { selectedGameRef.current = game.id; setSelectedGame(game.id); void refresh(game.id); }}>
          <span className="save-game-icon"><Archive size={17}/></span><span className="save-game-copy"><strong>{game.name}</strong><small title={game.savePath}>{game.savePath}</small></span><span className="save-game-arrow">›</span>
        </button>) : <div className="save-empty"><FolderOpen size={24}/><strong>Nenhum jogo cadastrado</strong><span>Adicione a pasta onde o jogo guarda seus arquivos de progresso.</span></div>}
        <button className="save-add-toggle" onClick={() => setAdding(value => !value)}><Plus size={15}/> Adicionar jogo manualmente</button>
      </div>

      <div className="save-detail-panel">
        {adding && <div className="save-add-form">
          <div className="save-panel-heading"><strong>Novo jogo</strong><span>Cadastro local</span></div>
          <label>Nome do jogo<input value={name} onChange={e => setName(e.target.value)} placeholder="Ex.: Zelda TOTK Recomp"/></label>
          <label>Pasta do save
            <div className="save-path-picker"><input value={savePath} onChange={e => setSavePath(e.target.value)} placeholder="Selecione a pasta de salvamento"/><button type="button" className="secondary-button" onClick={() => void browseSaveFolder()}><FolderOpen size={14}/> Procurar</button></div>
          </label>
          <small>Escolha a pasta que contém os arquivos de progresso. O DIAMANDIS ONE não altera o conteúdo ao cadastrar.</small>
          <div className="save-form-actions"><button className="secondary-button" onClick={() => setAdding(false)}>Cancelar</button><button className="primary-button compact" onClick={() => void addGame()} disabled={!name.trim() || !savePath.trim() || adding}><Plus size={13}/> Cadastrar</button></div>
        </div>}

        {selected ? <>
          <div className="save-selected-heading"><div><span className="tool-kicker">JOGO SELECIONADO</span><h3>{selected.name}</h3><code title={selected.savePath}>{selected.savePath}</code></div><button className="primary-button compact" onClick={() => void createBackup()} disabled={!!workingId}><Archive size={14}/>{workingId === "create" ? "Criando..." : "Criar backup"}</button></div>
          <div className="save-safety-strip"><ShieldCheck size={15}/><span>Backup somente por cópia. Restaurações preservam o estado atual em uma pasta de segurança.</span></div>
          <div className="save-history-heading"><div><h4>Histórico de backups</h4><p>{backups.length} versão(ões) armazenada(s) localmente</p></div><Clock3 size={17}/></div>
          {busy ? <div className="save-empty"><LoaderCircle className="spin" size={22}/><span>Carregando histórico...</span></div> : backups.length ? <div className="save-backup-list">{backups.map(backup => <article className="save-backup-row" key={backup.id}>
            <div className="save-backup-icon"><Archive size={16}/></div><div className="save-backup-info"><strong>{formatDate(backup.createdAt)}</strong><span>{backup.fileCount} arquivo(s) • {formatSize(backup.sizeBytes)} • {Object.keys(backup.fileHashes || {}).length ? "SHA-256 verificado" : "Verificação estrutural"}</span><small>{backup.backupPath}</small></div>
            <button className="game-icon-button" title="Restaurar este backup" onClick={() => void restore(backup)} disabled={!!workingId}><RotateCcw size={15}/></button>
          </article>)}</div> : <div className="save-empty compact-empty"><Archive size={22}/><strong>Seu primeiro backup começa aqui</strong><span>Crie uma cópia antes de trocar ou atualizar seus saves.</span></div>}
        </> : !adding && <div className="save-empty save-select-empty"><HardDrive size={28}/><strong>Selecione um jogo</strong><span>Ou cadastre uma pasta de salvamento para começar.</span></div>}
      </div>
    </div>
    <div className="gamehub-foot"><ShieldCheck size={14}/> Dados e backups permanecem no computador • Links simbólicos são bloqueados • Nenhum arquivo original é apagado pelo backup.</div>
  </section>;
}
