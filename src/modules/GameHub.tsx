import { useCallback, useEffect, useState } from "react";
import { FolderOpen, Gamepad2, LoaderCircle, RefreshCw, ShieldCheck } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";

type Game = { name: string; source: string; path: string };

export default function GameHub() {
  const [games, setGames] = useState<Game[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const scan = useCallback(async () => {
    setLoading(true); setError("");
    try { setGames(await invoke<Game[]>("scan_games")); }
    catch (e) { setError(String(e)); }
    finally { setLoading(false); }
  }, []);
  useEffect(() => { void scan(); }, [scan]);
  const openFolder = async (path: string) => {
    try {
      await navigator.clipboard.writeText(path);
    } catch {
      setError("Não foi possível copiar o caminho. Você ainda pode selecioná-lo na ficha do jogo.");
    }
  };
  return <section className="gamehub-page">
    <div className="gamehub-summary">
      <div className="gamehub-summary-icon"><Gamepad2 size={24}/></div>
      <div><span className="tool-kicker">BIBLIOTECA LOCAL</span><h3>{games.length} {games.length === 1 ? "jogo encontrado" : "jogos encontrados"}</h3><p>Detecção somente leitura. Nenhum launcher ou jogo será iniciado automaticamente.</p></div>
      <button className="toolbar-button" onClick={() => void scan()} disabled={loading} title="Atualizar biblioteca"><RefreshCw size={16} className={loading ? "spin" : ""}/></button>
    </div>
    {error && <div className="file-message error-message">{error}</div>}
    {loading ? <div className="gamehub-empty"><LoaderCircle className="spin" size={24}/><strong>Procurando jogos instalados...</strong><span>Verificando bibliotecas conhecidas.</span></div> :
      games.length ? <div className="gamehub-grid">{games.map(game => <article className="game-card" key={game.path}><div className="game-card-icon"><Gamepad2 size={21}/></div><div className="game-card-main"><strong>{game.name}</strong><span>{game.source}</span><code title={game.path}>{game.path}</code></div><button className="toolbar-button" onClick={() => void openFolder(game.path)} title="Copiar caminho"><FolderOpen size={15}/></button></article>)}</div> :
      <div className="gamehub-empty"><Gamepad2 size={30}/><strong>Nenhum jogo detectado ainda</strong><span>O GameHub procura instalações conhecidas de Steam, Epic Games e GOG.</span></div>}
    <div className="gamehub-foot"><ShieldCheck size={14}/> A busca é local e somente leitura. Bibliotecas Xbox e launchers adicionais serão adicionados após testes específicos.</div>
  </section>;
}
