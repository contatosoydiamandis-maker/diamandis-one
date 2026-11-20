import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Copy, FolderOpen, Gamepad2, Heart, LoaderCircle, RefreshCw, Search, ShieldCheck, Sparkles } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";

type Game = { name: string; source: string; path: string };
type SourceFilter = "Todos" | "Steam" | "Epic Games" | "GOG";
let cachedGames: Game[] | null = null;
let cachedAt = 0;
let scanPromise: Promise<Game[]> | null = null;

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
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [query, setQuery] = useState("");
  const [source, setSource] = useState<SourceFilter>("Todos");
  const [favorites, setFavorites] = useState<string[]>(readFavorites);
  const [showFavorites, setShowFavorites] = useState(false);
  const [copied, setCopied] = useState("");
  const [openingPath, setOpeningPath] = useState("");
  const copyTimer = useRef<number | null>(null);

  useEffect(() => () => {
    if (copyTimer.current !== null) window.clearTimeout(copyTimer.current);
  }, []);

  const scan = useCallback(async (force = false) => {
    if (!force && cachedGames && Date.now() - cachedAt < 45_000) {
      setGames(cachedGames);
      return;
    }
    setLoading(true);
    setError("");
    const request = scanPromise ?? invoke<Game[]>("scan_games");
    scanPromise = request;
    try {
      const result = await request;
      cachedGames = result;
      cachedAt = Date.now();
      setGames(result);
    } catch (e) {
      setError(String(e));
    } finally {
      if (scanPromise === request) scanPromise = null;
      setLoading(false);
    }
  }, []);

  useEffect(() => { void scan(); }, [scan]);

  const filtered = useMemo(() => games.filter(game => {
    const matchesQuery = game.name.toLocaleLowerCase().includes(query.toLocaleLowerCase());
    const matchesSource = source === "Todos" || game.source === source;
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

  const counts = useMemo(() => ({
    steam: games.filter(game => game.source === "Steam").length,
    epic: games.filter(game => game.source === "Epic Games").length,
    gog: games.filter(game => game.source === "GOG").length
  }), [games]);

  return <section className="gamehub-page">
    <div className="gamehub-banner">
      <div className="gamehub-banner-art"><div className="gamehub-banner-ring"/><Gamepad2 size={48}/><Sparkles size={17}/></div>
      <div className="gamehub-banner-copy"><span className="tool-kicker">DIAMANDIS GAMING NETWORK</span><h2>Sua biblioteca. <em>Suas regras.</em></h2><p>Um ponto de encontro para os jogos instalados no seu PC, sem depender de um launcher específico.</p></div>
      <div className="gamehub-total"><strong>{games.length.toString().padStart(2, "0")}</strong><span>JOGOS LOCALIZADOS</span></div>
    </div>

    <div className="gamehub-platforms">
      <button className={source === "Todos" ? "platform-stat active" : "platform-stat"} onClick={() => setSource("Todos")}><span className="platform-mark all"><Gamepad2 size={16}/></span><span><strong>{games.length}</strong><small>Todos os jogos</small></span></button>
      <button className={source === "Steam" ? "platform-stat active" : "platform-stat"} onClick={() => setSource("Steam")}><span className="platform-mark steam">S</span><span><strong>{counts.steam}</strong><small>Steam</small></span></button>
      <button className={source === "Epic Games" ? "platform-stat active" : "platform-stat"} onClick={() => setSource("Epic Games")}><span className="platform-mark epic">E</span><span><strong>{counts.epic}</strong><small>Epic Games</small></span></button>
      <button className={source === "GOG" ? "platform-stat active" : "platform-stat"} onClick={() => setSource("GOG")}><span className="platform-mark gog">G</span><span><strong>{counts.gog}</strong><small>GOG</small></span></button>
    </div>

    <div className="gamehub-library-heading"><div><span className="tool-kicker">COLEÇÃO LOCAL</span><h3>Biblioteca de jogos</h3><p>Os jogos são detectados no computador. O GameHub não altera instalações.</p></div>
      <button className="primary-button compact" onClick={() => void scan(true)} disabled={loading}><RefreshCw size={14} className={loading ? "spin" : ""}/> Atualizar biblioteca</button>
    </div>

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
        <div className="game-card-actions"><button className={favorites.includes(game.path) ? "game-icon-button favorite" : "game-icon-button"} onClick={() => toggleFavorite(game.path)} title="Adicionar aos favoritos" aria-label={favorites.includes(game.path) ? "Remover dos favoritos" : "Adicionar aos favoritos"}><Heart size={15} fill={favorites.includes(game.path) ? "currentColor" : "none"}/></button><button className="game-icon-button" onClick={() => void copyPath(game.path)} title="Copiar caminho" aria-label="Copiar caminho da instalação"><Copy size={15}/></button><button className="game-icon-button" onClick={() => void openGameFolder(game.path)} title="Abrir pasta de instalação" aria-label="Abrir pasta de instalação" disabled={openingPath === game.path}><FolderOpen size={15}/></button></div>
        {copied === game.path && <span className="game-copy-feedback">Caminho copiado</span>}
      </article>)}</div> :
      <div className="gamehub-empty"><Gamepad2 size={31}/><strong>{games.length ? "Nenhum jogo corresponde à busca" : "Sua biblioteca ainda está vazia"}</strong><span>{games.length ? "Experimente outro nome ou remova os filtros." : "O GameHub procura instalações conhecidas de Steam, Epic Games e GOG."}</span>{games.length > 0 && <button className="secondary-button" onClick={() => {setQuery("");setSource("Todos");setShowFavorites(false);}}>Limpar filtros</button>}</div>}
    <div className="gamehub-foot"><ShieldCheck size={14}/> Varredura local somente leitura • Favoritos armazenados neste computador • Mais launchers serão adicionados gradualmente.</div>
  </section>;
}
