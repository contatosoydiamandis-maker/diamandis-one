import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Copy, FolderOpen, Gamepad2, Heart, LoaderCircle, Play, Plus, RefreshCw, Search, Settings2, ShieldCheck, Sparkles, Trash2 } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

type Game = { name: string; source: string; path: string; executable?: string };
type LibraryRoot = { path: string; source: "Hydra" | "Outros" };
type SourceFilter = "Todos" | "Xbox" | "Hydra" | "Steam" | "Epic Games" | "GOG" | "Outros" | "Manual";
let cachedGames: Game[] | null = null;
let cachedAt = 0;
let scanPromise: Promise<Game[]> | null = null;
let scanPromiseKey = "";

const readFavorites = (): string[] => {
  try {
    const stored: unknown = JSON.parse(localStorage.getItem("diamandis.gamehub.favorites") || "[]");
    return Array.isArray(stored)
      ? stored.filter((item): item is string => typeof item === "string")
      : [];
  } catch {
    return [];
  }
};

export default function GameHub() {
  const [games, setGames] = useState<Game[]>(cachedGames ?? []);
  const [manualGames, setManualGames] = useState<Game[]>(() => {
    try { const value: unknown = JSON.parse(localStorage.getItem("diamandis.gamehub.manualGames") || "[]"); return Array.isArray(value) ? value.filter((item): item is Game => !!item && typeof item.name === "string" && typeof item.path === "string" && typeof item.executable === "string") : []; }
    catch { return []; }
  });
  const [libraryRoots, setLibraryRoots] = useState<LibraryRoot[]>(() => {
    try { const value: unknown = JSON.parse(localStorage.getItem("diamandis.gamehub.libraryRoots") || "[]"); return Array.isArray(value) ? value.filter((item): item is LibraryRoot => !!item && typeof item.path === "string" && (item.source === "Hydra" || item.source === "Outros")) : []; }
    catch { return []; }
  });

  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [query, setQuery] = useState("");
  const [source, setSource] = useState<SourceFilter>("Todos");
  const [favorites, setFavorites] = useState<string[]>(readFavorites);
  const [showFavorites, setShowFavorites] = useState(false);
  const [copied, setCopied] = useState("");
  const [openingPath, setOpeningPath] = useState("");
  const [launchingPath, setLaunchingPath] = useState("");
  const [launchTargets, setLaunchTargets] = useState<Record<string, string>>(() => {
    try { const value: unknown = JSON.parse(localStorage.getItem("diamandis.gamehub.launchTargets") || "{}"); return value && typeof value === "object" && !Array.isArray(value) ? value as Record<string, string> : {}; }
    catch { return {}; }
  });
  const copyTimer = useRef<number | null>(null);
  const scanVersion = useRef(0);

  useEffect(() => () => {
    if (copyTimer.current !== null) window.clearTimeout(copyTimer.current);
  }, []);

  const scan = useCallback(async (force = false) => {
    if (!force && cachedGames && Date.now() - cachedAt < 45_000) {
      setGames([...cachedGames, ...manualGames]);
      return;
    }
    const requestVersion = ++scanVersion.current;
    setLoading(true);
    setError("");
    const requestKey = JSON.stringify(libraryRoots.map(root => ({ path: root.path.toLowerCase(), source: root.source })));
    const request = scanPromise && scanPromiseKey === requestKey
      ? scanPromise
      : invoke<Game[]>("scan_games", { customRoots: libraryRoots });
    scanPromise = request;
    scanPromiseKey = requestKey;
    try {
      const result = await request;
      if (requestVersion !== scanVersion.current) return;
      cachedGames = result;
      cachedAt = Date.now();
      setGames([...result, ...manualGames]);
    } catch (e) {
      if (requestVersion === scanVersion.current) setError(String(e));
    } finally {
      if (scanPromise === request) {
        scanPromise = null;
        scanPromiseKey = "";
      }
      if (requestVersion === scanVersion.current) setLoading(false);
    }
  }, [libraryRoots, manualGames]);

  useEffect(() => { void scan(); }, [scan]);

  const filtered = useMemo(() => games.filter(game => {
    const matchesQuery = game.name.toLocaleLowerCase().includes(query.toLocaleLowerCase());
    const matchesSource = source === "Todos" || game.source === source || (source === "Outros" && game.source === "Manual");
    const matchesFavorite = !showFavorites || favorites.includes(game.path);
    return matchesQuery && matchesSource && matchesFavorite;
  }).sort((a, b) => a.name.localeCompare(b.name, "pt-BR")), [games, query, source, showFavorites, favorites]);

  const toggleFavorite = (path: string) => {
    setFavorites(current => {
      const next = current.includes(path) ? current.filter(item => item !== path) : [...current, path];
      try { localStorage.setItem("diamandis.gamehub.favorites", JSON.stringify(next)); } catch { /* armazenamento local opcional */ }
      return next;
    });
  };

  const copyPath = async (path: string) => {
    try {
      await navigator.clipboard.writeText(path);
      setCopied(path);
      if (copyTimer.current !== null) window.clearTimeout(copyTimer.current);
      copyTimer.current = window.setTimeout(() => {
        setCopied("");
        copyTimer.current = null;
      }, 1400);
    } catch {
      setError("Não foi possível copiar o caminho. Selecione o texto do caminho no cartão.");
    }
  };

  const openGameFolder = async (path: string) => {
    setOpeningPath(path);
    setError("");
    try {
      await invoke("open_game_folder", { path });
    } catch (e) {
      setError(String(e));
    } finally {
      setOpeningPath("");
    }
  };


  const configureExecutable = async (game: Game) => {
    setError("");
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        defaultPath: game.path,
        title: `Selecione o executável de ${game.name}`,
        filters: [{ name: "Executáveis Windows", extensions: ["exe"] }]
      });
      if (typeof selected !== "string") return;
      const next = { ...launchTargets, [game.path]: selected };
      setLaunchTargets(next);
      try { localStorage.setItem("diamandis.gamehub.launchTargets", JSON.stringify(next)); }
      catch { setError("Executável configurado nesta sessão, mas não foi possível persistir a preferência local."); }
    } catch (e) { setError(String(e)); }
  };

  const launchGame = async (game: Game) => {
    const executable = game.executable || launchTargets[game.path];
    if (!executable) { await configureExecutable(game); return; }
    setLaunchingPath(game.path);
    setError("");
    try {
      await invoke("launch_game", { gameFolderPath: game.path, executablePath: executable });
    } catch (e) {
      setError(String(e));
    } finally {
      setLaunchingPath("");
    }
  };

  const counts = useMemo(() => ({
    xbox: games.filter(game => game.source === "Xbox").length,
    hydra: games.filter(game => game.source === "Hydra").length,
    steam: games.filter(game => game.source === "Steam").length,
    epic: games.filter(game => game.source === "Epic Games").length,
    gog: games.filter(game => game.source === "GOG").length,
    others: games.filter(game => game.source === "Outros" || game.source === "Manual").length
  }), [games]);

  const addLibraryRoot = async (rootSource: "Hydra" | "Outros") => {
    setError("");
    try {
      const selected = await open({ directory: true, multiple: false, title: rootSource === "Hydra" ? "Selecione a pasta onde o Hydra guarda os jogos" : "Selecione uma pasta de biblioteca de jogos" });
      if (typeof selected !== "string") return;
      const next = [...libraryRoots.filter(root => root.path.toLowerCase() !== selected.toLowerCase()), { path: selected, source: rootSource }];
      setLibraryRoots(next);
      try { localStorage.setItem("diamandis.gamehub.libraryRoots", JSON.stringify(next)); }
      catch { setError("Pasta adicionada nesta sessão, mas não foi possível salvar a preferência."); }
    } catch (e) { setError(String(e)); }
  };

  const removeManualGame = (path: string) => {
    const next = manualGames.filter(game => game.path !== path);
    setManualGames(next);
    setGames(current => current.filter(game => game.path !== path));
    try { localStorage.setItem("diamandis.gamehub.manualGames", JSON.stringify(next)); }
    catch { setError("Não foi possível atualizar a biblioteca manual local."); }
  };

  const addManualGame = async () => {
    setError("");
    try {
      const folder = await open({ directory: true, multiple: false, title: "Selecione a pasta do jogo" });
      if (typeof folder !== "string") return;
      const executable = await open({ directory: false, multiple: false, defaultPath: folder, title: "Selecione o executável principal do jogo", filters: [{ name: "Executáveis Windows", extensions: ["exe"] }] });
      if (typeof executable !== "string") return;
      const normalizedFolder = folder.replace(/[\\\\/]+$/, "").toLowerCase();
      const normalizedExecutable = executable.toLowerCase();
      if (!normalizedExecutable.startsWith(normalizedFolder + "\\\\")) {
        setError("O executável precisa estar dentro da pasta selecionada para esse jogo.");
        return;
      }
      const name = folder.split(/[\\\\/]/).filter(Boolean).pop() || "Jogo manual";
      const next = [...manualGames.filter(game => game.path.toLowerCase() !== folder.toLowerCase()), { name, source: "Manual", path: folder, executable }];
      setManualGames(next);
      setGames(current => [...current.filter(game => game.path.toLowerCase() !== folder.toLowerCase()), next[next.length - 1]]);
      try { localStorage.setItem("diamandis.gamehub.manualGames", JSON.stringify(next)); }
      catch { setError("Jogo adicionado nesta sessão, mas não foi possível salvar a biblioteca manual."); }
    } catch (e) { setError(String(e)); }
  };


  return <section className="gamehub-page">
    <div className="gamehub-banner">
      <div className="gamehub-banner-art"><div className="gamehub-banner-ring"/><Gamepad2 size={48}/><Sparkles size={17}/></div>
      <div className="gamehub-banner-copy"><span className="tool-kicker">DIAMANDIS GAMING NETWORK</span><h2>Sua biblioteca. <em>Suas regras.</em></h2><p>Xbox, Hydra, Steam, Epic, GOG e jogos independentes em uma biblioteca unificada.</p></div>
      <div className="gamehub-total"><strong>{games.length.toString().padStart(2, "0")}</strong><span>JOGOS LOCALIZADOS</span></div>
    </div>

    <div className="gamehub-platforms">
      <button className={source === "Todos" ? "platform-stat active" : "platform-stat"} onClick={() => setSource("Todos")}><span className="platform-mark all"><Gamepad2 size={16}/></span><span><strong>{games.length}</strong><small>Todos os jogos</small></span></button>
      <button className={source === "Xbox" ? "platform-stat active" : "platform-stat"} onClick={() => setSource("Xbox")}><span className="platform-mark xbox">X</span><span><strong>{counts.xbox}</strong><small>Xbox</small></span></button>
      <button className={source === "Hydra" ? "platform-stat active" : "platform-stat"} onClick={() => setSource("Hydra")}><span className="platform-mark hydra">H</span><span><strong>{counts.hydra}</strong><small>Hydra</small></span></button>
      <button className={source === "Steam" ? "platform-stat active" : "platform-stat"} onClick={() => setSource("Steam")}><span className="platform-mark steam">S</span><span><strong>{counts.steam}</strong><small>Steam</small></span></button>
      <button className={source === "Epic Games" ? "platform-stat active" : "platform-stat"} onClick={() => setSource("Epic Games")}><span className="platform-mark epic">E</span><span><strong>{counts.epic}</strong><small>Epic Games</small></span></button>
      <button className={source === "GOG" ? "platform-stat active" : "platform-stat"} onClick={() => setSource("GOG")}><span className="platform-mark gog">G</span><span><strong>{counts.gog}</strong><small>GOG</small></span></button>
      <button className={source === "Outros" || source === "Manual" ? "platform-stat active" : "platform-stat"} onClick={() => setSource("Outros")}><span className="platform-mark other">+</span><span><strong>{counts.others}</strong><small>Outros / Manual</small></span></button>
    </div>

    <div className="gamehub-library-heading"><div><span className="tool-kicker">COLEÇÃO LOCAL</span><h3>Biblioteca de jogos</h3><p>Os jogos são detectados no computador. O GameHub não altera instalações.</p></div>
      <button className="primary-button compact" onClick={() => void scan(true)} disabled={loading}><RefreshCw size={14} className={loading ? "spin" : ""}/> Atualizar biblioteca</button>
    </div>

    <div className="gamehub-library-actions">
      <button className="secondary-button" onClick={() => void addLibraryRoot("Hydra")}><FolderOpen size={14}/> Adicionar biblioteca Hydra</button>
      <button className="secondary-button" onClick={() => void addLibraryRoot("Outros")}><FolderOpen size={14}/> Adicionar outra biblioteca</button>
      <button className="primary-button compact" onClick={() => void addManualGame()}><Plus size={14}/> Adicionar jogo manual</button>
    </div>
    {libraryRoots.length > 0 && <div className="gamehub-roots" aria-label="Bibliotecas adicionais">{libraryRoots.map(root => <span key={root.path}>{root.source}: {root.path}<button onClick={() => { const next = libraryRoots.filter(item => item.path !== root.path); setLibraryRoots(next); localStorage.setItem("diamandis.gamehub.libraryRoots", JSON.stringify(next)); }} aria-label={`Remover biblioteca ${root.path}`}>×</button></span>)}</div>}
    <div className="gamehub-controls">
      <label className="gamehub-search"><Search size={15}/><input aria-label="Buscar jogos por nome" value={query} onChange={e => setQuery(e.target.value)} placeholder="Buscar por nome do jogo..."/></label>
      <button className={showFavorites ? "gamehub-favorites active" : "gamehub-favorites"} onClick={() => setShowFavorites(value => !value)}><Heart size={14} fill={showFavorites ? "currentColor" : "none"}/> Favoritos <span>{favorites.length}</span></button>
      <span className="gamehub-results">{filtered.length} {filtered.length === 1 ? "resultado" : "resultados"}</span>
    </div>

    {error && <div className="file-message error-message" role="alert">{error}</div>}
    {loading ? <div className="gamehub-empty"><LoaderCircle className="spin" size={25}/><strong>Mapeando suas bibliotecas...</strong><span>Buscando instalações conhecidas sem abrir seus launchers.</span></div> :
      filtered.length ? <div className="gamehub-grid">{filtered.map(game => <article className="game-card" key={game.path}>
        <div className="game-card-cover"><Gamepad2 size={25}/><span>{game.source}</span></div>
        <div className="game-card-main"><strong title={game.name}>{game.name}</strong><span className="game-card-platform">{game.source} <i/> INSTALADO</span><code title={game.path}>{game.path}</code></div>
        <div className="game-card-actions"><button className={favorites.includes(game.path) ? "game-icon-button favorite" : "game-icon-button"} onClick={() => toggleFavorite(game.path)} title="Adicionar aos favoritos" aria-label={favorites.includes(game.path) ? "Remover dos favoritos" : "Adicionar aos favoritos"}><Heart size={15} fill={favorites.includes(game.path) ? "currentColor" : "none"}/></button><button className="game-icon-button" onClick={() => void copyPath(game.path)} title="Copiar caminho" aria-label="Copiar caminho da instalação"><Copy size={15}/></button><button className="game-icon-button" onClick={() => void openGameFolder(game.path)} title="Abrir pasta de instalação" aria-label="Abrir pasta de instalação" disabled={openingPath === game.path}><FolderOpen size={15}/></button><button className="game-icon-button" onClick={() => void configureExecutable(game)} title="Selecionar executável do jogo" aria-label="Selecionar executável do jogo"><Settings2 size={15}/></button><button className="game-icon-button" onClick={() => void launchGame(game)} title={launchTargets[game.path] ? "Iniciar jogo" : "Configurar executável para iniciar"} aria-label={launchTargets[game.path] ? "Iniciar jogo" : "Configurar executável para iniciar"} disabled={launchingPath === game.path}>{launchingPath === game.path ? <LoaderCircle className="spin" size={15}/> : <Play size={15}/>}</button>{game.source === "Manual" && <button className="game-icon-button" onClick={() => removeManualGame(game.path)} title="Remover da biblioteca" aria-label="Remover jogo manual da biblioteca"><Trash2 size={15}/></button>}</div>
        {copied === game.path && <span className="game-copy-feedback">Caminho copiado</span>}
      </article>)}</div> :
      <div className="gamehub-empty"><Gamepad2 size={31}/><strong>{games.length ? "Nenhum jogo corresponde à busca" : "Sua biblioteca ainda está vazia"}</strong><span>{games.length ? "Experimente outro nome ou remova os filtros." : "O GameHub procura instalações conhecidas de Steam, Epic Games, GOG e XboxGames, além das bibliotecas que você adicionar."}</span>{games.length > 0 && <button className="secondary-button" onClick={() => {setQuery("");setSource("Todos");setShowFavorites(false);}}>Limpar filtros</button>}</div>}
    <div className="gamehub-foot"><ShieldCheck size={14}/> Varredura local somente leitura • Bibliotecas adicionais configuradas por você • Jogos manuais salvos localmente.</div>
  </section>;
}
