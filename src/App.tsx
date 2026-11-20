import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { FormEvent } from "react";
import {
  AppWindow, ArrowLeft, ArrowUpRight, Boxes, Check, CheckCircle2, ChevronRight, Copy, Braces, Type,
  Clock3, Command, File, FileSearch, Folder, FolderKanban, FolderPlus, Gamepad2, HardDrive,
  Image, LayoutDashboard, LoaderCircle, RefreshCw, Search, Settings2, ShieldCheck, Sparkles,
  Wrench, Zap
} from "lucide-react";
import type { LucideIcon } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import type { ModuleId } from "./types";
import { setThemePreference } from "./theme";
import GameHub from "./modules/GameHub";
import Settings from "./modules/Settings";
import SaveCenter from "./modules/SaveCenter";

type NavItem = { id: ModuleId; label: string; icon: LucideIcon; group: string };

const nav: NavItem[] = [
  { id: "overview", label: "Visão geral", icon: LayoutDashboard, group: "WORKSPACE" },
  { id: "files", label: "File Manager", icon: FolderKanban, group: "WORKSPACE" },
  { id: "toolbox", label: "Toolbox", icon: Wrench, group: "WORKSPACE" },
  { id: "games", label: "GameHub", icon: Gamepad2, group: "GAMING" },
  { id: "saves", label: "Save Center", icon: HardDrive, group: "GAMING" },
  { id: "settings", label: "Configurações", icon: Settings2, group: "SYSTEM" }
];

const descriptions: Record<ModuleId, { title: string; subtitle: string }> = {
  overview: { title: "Seu espaço. Seu controle.", subtitle: "Tudo organizado, em um só lugar." },
  files: { title: "File Manager", subtitle: "Encontre, organize e entenda seus arquivos." },
  toolbox: { title: "Toolbox", subtitle: "Pequenas ferramentas para tarefas do dia a dia." },
  games: { title: "GameHub", subtitle: "Sua biblioteca gamer em um só lugar." },
  saves: { title: "Save Center", subtitle: "Proteja seus progressos com backups organizados." },
  settings: { title: "Configurações", subtitle: "Personalize sua experiência no DIAMANDIS ONE." }
};

function App() {
  const [active, setActive] = useState<ModuleId>("overview");
  const [search, setSearch] = useState("");
  const searchInputRef = useRef<HTMLInputElement>(null);
  const [diagnostic, setDiagnostic] = useState("Verificação ainda não executada");

  useEffect(() => {
    void invoke<{ theme: "dark" | "light" | "system" }>("load_settings")
      .then(settings => setThemePreference(settings.theme))
      .catch(() => setThemePreference("dark"));
  }, []);

  const runDiagnostic = async () => {
    try {
      const result = await invoke<string>("core_diagnostic");
      setDiagnostic(result);
    } catch {
      setDiagnostic("Modo de pré-visualização: execute pelo Tauri para acessar o núcleo Rust.");
    }
  };

  useEffect(() => {
    const handleShortcut = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        searchInputRef.current?.focus();
      }
      if (event.key === "Escape" && document.activeElement === searchInputRef.current) {
        setSearch("");
        searchInputRef.current?.blur();
      }
    };
    window.addEventListener("keydown", handleShortcut);
    return () => window.removeEventListener("keydown", handleShortcut);
  }, []);

  const current = descriptions[active];
  const filtered = nav.filter(item => item.label.toLowerCase().includes(search.toLowerCase()));

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <div className="brand-mark"><img src="/diamandis-mark.svg" alt="" /></div>
          <div><div className="brand-name">DIAMANDIS<span> ONE</span></div><div className="brand-caption">PERSONAL DESKTOP HUB</div></div>
        </div>

        <div className="sidebar-search">
          <Search size={15} />
          <input value={search} onChange={e => setSearch(e.target.value)} placeholder="Encontrar módulo..." />
          <kbd>Ctrl K</kbd>
        </div>

        {["WORKSPACE", "GAMING", "SYSTEM"].map(group => (
          <div className="nav-group" key={group}>
            <div className="nav-heading">{group}</div>
            {filtered.filter(item => item.group === group).map(item => {
              const Icon = item.icon;
              return <button key={item.id} className={`nav-item ${active === item.id ? "active" : ""}`} onClick={() => setActive(item.id)}>
                <Icon size={17} /><span>{item.label}</span>{active === item.id && <ChevronRight size={14} className="nav-chevron" />}
              </button>;
            })}
          </div>
        ))}

        <div className="sidebar-bottom">
          <div className="local-card"><div className="local-icon"><ShieldCheck size={17} /></div><div><strong>Local-first</strong><span>Seus dados ficam no seu PC</span></div><CheckCircle2 size={14} className="local-check" /></div>
          <div className="version">DIAMANDIS ONE <span>v0.2.0</span></div>
        </div>
      </aside>

      <main className="main-area">
        <header className="topbar">
          <div className="breadcrumb"><span>DIAMANDIS ONE</span><ChevronRight size={13}/><strong>{current.title}</strong></div>
          <div className="top-actions"><span className="system-status"><i/> PRÉ-ALPHA • EM DESENVOLVIMENTO</span><div className="avatar">D</div></div>
        </header>

        <div className="content">
          <section className="welcome">
            <div><div className="eyebrow"><Sparkles size={13}/> CENTRAL DE COMANDO</div><h1>{current.title}</h1><p>{current.subtitle}</p></div>
            <div className="welcome-orb"><div className="orb-core"><Command size={29}/></div></div>
          </section>

          {active === "overview" ? <>
            <section className="hero-card">
              <div className="hero-copy"><div className="hero-tag"><Zap size={13}/> UM HUB. MÚLTIPLAS POSSIBILIDADES.</div><h2>Seu Windows,<br/><em>do seu jeito.</em></h2><p>Organização, produtividade e universo gamer reunidos em uma experiência única.</p><button className="primary-button" onClick={() => setActive("files")}>Explorar módulos <ArrowUpRight size={16}/></button></div>
              <div className="hero-art"><div className="ring ring-one"/><div className="ring ring-two"/><div className="hero-glyph"><Boxes size={62} strokeWidth={1}/></div><div className="float-chip chip-a"><FolderKanban size={15}/> Arquivos</div><div className="float-chip chip-b"><Gamepad2 size={15}/> GameHub</div><div className="float-chip chip-c"><Wrench size={15}/> Toolbox</div></div>
            </section>

            <div className="section-title"><div><h3>Seu workspace</h3><p>Escolha por onde começar</p></div><span className="section-count">03 MÓDULOS</span></div>
            <section className="module-grid">
              <ModuleCard icon={FolderKanban} title="File Manager" desc="Navegue, filtre, copie, mova, renomeie e envie itens à Lixeira." meta="ORGANIZAÇÃO" onClick={() => setActive("files")}/>
              <ModuleCard icon={Wrench} title="Toolbox" desc="Ferramentas locais para transformar textos e formatar JSON." meta="PRODUTIVIDADE" onClick={() => setActive("toolbox")}/>
              <ModuleCard icon={Gamepad2} title="GameHub" desc="Descubra jogos instalados e organize seus favoritos em um só lugar." meta="GAMING" onClick={() => setActive("games")}/>
            </section>

            <div className="section-title lower-title"><div><h3>Centro do sistema</h3><p>Base local e diagnóstico</p></div></div>
            <section className="status-grid">
              <div className="status-card"><div className="status-icon"><HardDrive size={18}/></div><div><span>ARMAZENAMENTO</span><strong>Local-first</strong><small>Sem conta ou nuvem obrigatória</small></div></div>
              <div className="status-card"><div className="status-icon"><ShieldCheck size={18}/></div><div><span>PRIVACIDADE</span><strong>Controle local</strong><small>Sem telemetria invasiva</small></div></div>
              <button className="status-card diagnostic-card" onClick={runDiagnostic}><div className="status-icon"><AppWindow size={18}/></div><div><span>DIAGNÓSTICO DO CORE</span><strong>Executar verificação <ArrowUpRight size={13}/></strong><small>{diagnostic}</small></div></button>
            </section>
          </> : active === "files" ? <FileManager/> : active === "toolbox" ? <Toolbox/> : active === "games" ? <GameHub/> : active === "saves" ? <SaveCenter/> : active === "settings" ? <Settings/> : <ModulePlaceholder id={active} onBack={() => setActive("overview")}/>}
          <footer><span>DIAMANDIS ONE <b>•</b> BUILT FOR YOUR DESKTOP</span><span><Clock3 size={12}/> PRÉ-ALPHA 0.2.0</span></footer>
        </div>
      </main>
    </div>
  );
}


function FileManager() {
  type Entry = { name: string; path: string; isDir: boolean; size: number | null };
  type Listing = { path: string; parent: string | null; total: number; offset: number; limit: number; entries: Entry[] };
  const PAGE_SIZE = 150;
  type Preferences = { theme: string; defaultFolder: string; confirmBeforeDelete: boolean; openLastFolder: boolean; lastFolder: string };
  type ActionMode = "rename" | "copy" | "move" | "delete";
  const [listing, setListing] = useState<Listing | null>(null);
  const [path, setPath] = useState("");
  const [page, setPage] = useState(0);
  const [filter, setFilter] = useState("");
  const [sortBy, setSortBy] = useState<"name" | "size" | "type">("name");
  const [kindFilter, setKindFilter] = useState<"all" | "folders" | "files">("all");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [newName, setNewName] = useState("");
  const [showNew, setShowNew] = useState(false);
  const [selected, setSelected] = useState<Entry | null>(null);
  const [action, setAction] = useState<ActionMode | null>(null);
  const [actionValue, setActionValue] = useState("");
  const [preferences, setPreferences] = useState<Preferences>({theme:"dark",defaultFolder:"",confirmBeforeDelete:true,openLastFolder:false,lastFolder:""});
  const [settingsLoaded, setSettingsLoaded] = useState(false);
  const preferencesRef = useRef(preferences);
  useEffect(() => { preferencesRef.current = preferences; }, [preferences]);

  const load = useCallback(async (target?: string, requestedPage = 0) => {
    setBusy(true); setError(""); setSelected(null);
    try {
      const data = await invoke<Listing>("list_directory", { path: target || null, offset: requestedPage * PAGE_SIZE, limit: PAGE_SIZE });
      setListing(data); setPath(data.path); setPage(requestedPage);
      if (preferencesRef.current.openLastFolder && preferencesRef.current.lastFolder !== data.path) {
        const next = { ...preferencesRef.current, lastFolder: data.path };
        preferencesRef.current = next;
        setPreferences(next);
        void invoke("save_settings", { settings: next }).catch(() => {});
      }
    } catch (e) { setError(String(e)); }
    finally { setBusy(false); }
  }, []);

  useEffect(() => {
    let active = true;
    invoke<Preferences>("load_settings").then(settings => {
      if (!active) return;
      setPreferences(settings);
      preferencesRef.current = settings;
      const initial = settings.openLastFolder && settings.lastFolder ? settings.lastFolder : settings.defaultFolder;
      setSettingsLoaded(true);
      void load(initial || undefined);
    }).catch(() => { if (active) { setSettingsLoaded(true); void load(); } });
    return () => { active = false; };
  }, []);

  const pageCount = Math.max(1, Math.ceil((listing?.total ?? 0) / PAGE_SIZE));
  const visible = useMemo(() => {
    const items = (listing?.entries ?? []).filter(item =>
      item.name.toLocaleLowerCase().includes(filter.toLocaleLowerCase()) &&
      (kindFilter === "all" || (kindFilter === "folders" ? item.isDir : !item.isDir))
    );
    return items.sort((a, b) => {
      if (sortBy === "type") return Number(b.isDir) - Number(a.isDir) || a.name.localeCompare(b.name, "pt-BR");
      if (sortBy === "size") return (b.size ?? -1) - (a.size ?? -1);
      return Number(b.isDir) - Number(a.isDir) || a.name.localeCompare(b.name, "pt-BR");
    });
  }, [listing, filter, kindFilter, sortBy]);
  const create = async (event: FormEvent) => {
    event.preventDefault(); if (!listing || !newName.trim()) return;
    try { await invoke("create_directory", { parent: listing.path, name: newName.trim() }); setNewName(""); setShowNew(false); await load(listing.path); }
    catch (e) { setError(String(e)); }
  };
  const runAction = async () => {
    if (!selected || !action) return;
    setError("");
    try {
      if (action === "rename") await invoke("rename_item", { path: selected.path, newName: actionValue });
      if (action === "copy") await invoke("copy_file", { path: selected.path, destinationFolderPath: actionValue });
      if (action === "move") await invoke("move_item", { path: selected.path, destinationFolderPath: actionValue });
      if (action === "delete") await invoke("delete_item", { path: selected.path });
      setAction(null); setActionValue(""); await load(listing?.path);
    } catch (e) { setError(String(e)); }
  };
  const openAction = (mode: ActionMode) => {
    if (!selected) return;
    if (mode === "delete" && !preferences.confirmBeforeDelete) {
      setError("");
      void invoke("delete_item", { path: selected.path })
        .then(() => load(listing?.path))
        .catch(e => setError(String(e)));
      return;
    }
    setAction(mode);
    setActionValue(mode === "rename" ? selected.name : mode === "delete" ? "" : listing?.path ?? "");
  };
  const size = (n: number | null) => n === null ? "—" : n < 1024 ? n + " B" : n < 1048576 ? (n/1024).toFixed(1)+" KB" : n < 1073741824 ? (n/1048576).toFixed(1)+" MB" : (n/1073741824).toFixed(2)+" GB";
  return <section className="file-manager">
    <div className="file-toolbar"><div className="file-toolbar-left">
      <button className="toolbar-button" disabled={!listing?.parent || busy} onClick={() => listing?.parent && void load(listing.parent)} title="Pasta anterior"><ArrowLeft size={16}/></button>
      <form className="path-form" onSubmit={e => {e.preventDefault(); void load(path);}}><HardDrive size={15}/><input aria-label="Caminho da pasta" value={path} onChange={e => setPath(e.target.value)} placeholder="Caminho da pasta..."/></form>
      <button className="toolbar-button" disabled={busy} onClick={() => void load(path || undefined)} title="Atualizar"><RefreshCw size={15}/></button>
    </div><button className="primary-button compact" onClick={() => setShowNew(v => !v)}><FolderPlus size={15}/> Nova pasta</button></div>
    {showNew && <form className="new-folder-form" onSubmit={create}><FolderPlus size={16}/><input autoFocus value={newName} onChange={e => setNewName(e.target.value)} placeholder="Nome da nova pasta"/><button className="primary-button compact" disabled={!newName.trim()}>Criar pasta</button><button type="button" className="toolbar-button" onClick={() => setShowNew(false)}>Cancelar</button></form>}
    {selected && <div className="file-selection-bar"><span><strong>{selected.name}</strong> selecionado</span><div>
      <button onClick={() => openAction("rename")}>Renomear</button>
      {!selected.isDir && <button onClick={() => openAction("copy")}>Copiar</button>}
      <button onClick={() => openAction("move")}>Mover</button>
      <button className="danger-action" onClick={() => openAction("delete")}>Enviar à Lixeira</button>
    </div></div>}
    {error && <div className="file-message error-message">{error}</div>}
    <div className="file-list-header"><div><h3>Arquivos e pastas</h3><p>{listing?.path ?? (settingsLoaded ? "Escolha uma pasta para começar" : "Carregando preferências...")}</p></div><div className="file-list-actions"><span className="file-count">{listing?.total ?? 0} ITENS • PÁGINA {page + 1}/{pageCount}</span><select className="file-sort" aria-label="Ordenar página" value={sortBy} onChange={e => setSortBy(e.target.value as "name" | "size" | "type")}><option value="name">Nome</option><option value="type">Tipo</option><option value="size">Tamanho</option></select><select className="file-sort" aria-label="Filtrar tipo" value={kindFilter} onChange={e => setKindFilter(e.target.value as "all" | "folders" | "files")}><option value="all">Tudo</option><option value="folders">Pastas</option><option value="files">Arquivos</option></select><label className="file-search"><Search size={14}/><input value={filter} onChange={e => setFilter(e.target.value)} placeholder="Filtrar nesta página"/></label></div></div>
    <div className="file-table"><div className="file-table-head"><span>NOME</span><span>TIPO</span><span>TAMANHO</span></div>
      {busy ? <div className="file-empty"><LoaderCircle className="spin" size={24}/><strong>Carregando pasta...</strong></div> : visible.length ? visible.map(item => <div role="button" tabIndex={0} className={selected?.path === item.path ? "file-row selected" : "file-row"} key={item.path} onClick={() => setSelected(item)} onDoubleClick={() => item.isDir && void load(item.path)} onKeyDown={e => { if (e.key === "Enter") item.isDir && void load(item.path); if (e.key === "Delete") { setSelected(item); setAction("delete"); setActionValue(""); } }}>
        <span className="file-name">{item.isDir ? <Folder size={18} className="folder-color"/> : <File size={17}/>}<strong>{item.name}</strong></span><span>{item.isDir ? "Pasta" : "Arquivo"}</span><span>{size(item.size)}</span>
      </div>) : <div className="file-empty"><Folder size={25}/><strong>{filter ? "Nenhum resultado" : "Esta pasta está vazia"}</strong><span>Os itens desta localização aparecerão aqui.</span></div>}
    </div>
    <div className="file-pagination"><span>Mostrando {listing?.entries.length ?? 0} de {listing?.total ?? 0} itens nesta pasta</span><div><button className="toolbar-button" disabled={busy || page <= 0} onClick={() => listing && void load(listing.path, page - 1)}>Anterior</button><strong>{page + 1} / {pageCount}</strong><button className="toolbar-button" disabled={busy || page + 1 >= pageCount} onClick={() => listing && void load(listing.path, page + 1)}>Próxima</button></div></div>
    <div className="file-manager-foot"><ShieldCheck size={14}/> Exclusões vão para a Lixeira. Cópias e movimentos não sobrescrevem arquivos existentes.</div>
    {action && selected && <div className="file-modal-backdrop" role="presentation" onMouseDown={e => { if (e.target === e.currentTarget) setAction(null); }}><div className="file-modal" role="dialog" aria-modal="true" aria-labelledby="file-modal-title">
      <h3 id="file-modal-title">{action === "rename" ? "Renomear item" : action === "copy" ? "Copiar arquivo" : action === "move" ? "Mover item" : "Enviar para a Lixeira"}</h3>
      <p>{action === "delete" ? <>O item <strong>{selected.name}</strong> será enviado à Lixeira do Windows. Ele não será excluído permanentemente.</> : action === "rename" ? "Informe o novo nome. A extensão pode ser mantida ou alterada." : action === "copy" ? "Informe o caminho completo da pasta de destino. Arquivos existentes não serão sobrescritos." : "Informe o caminho completo da pasta de destino. A operação não atravessa discos diferentes."}</p>
      {action !== "delete" && <input autoFocus value={actionValue} onChange={e => setActionValue(e.target.value)} placeholder={action === "rename" ? "Novo nome" : "C:\\Caminho\\da\\pasta"}/>}
      <div className="file-modal-actions"><button className="toolbar-button" onClick={() => setAction(null)}>Cancelar</button><button className={action === "delete" ? "danger-button" : "primary-button compact"} onClick={() => void runAction()} disabled={action !== "delete" && !actionValue.trim()}>{action === "delete" ? "Enviar à Lixeira" : "Confirmar"}</button></div>
    </div></div>}
  </section>;
}

function Toolbox() {
  const [textValue, setTextValue] = useState("");
  const [jsonValue, setJsonValue] = useState("");
  const [jsonResult, setJsonResult] = useState("");
  const [jsonError, setJsonError] = useState("");
  const [copied, setCopied] = useState("");
  const transform = async (value: string, key: string) => {
    setTextValue(value);
    try { await navigator.clipboard.writeText(value); setCopied(key); window.setTimeout(() => setCopied(""), 1600); }
    catch { setCopied("error"); }
  };
  const formatJson = () => {
    try { setJsonResult(JSON.stringify(JSON.parse(jsonValue), null, 2)); setJsonError(""); }
    catch (e) { setJsonResult(""); setJsonError(e instanceof Error ? e.message : "JSON inválido."); }
  };
  const copyJson = async () => {
    try { await navigator.clipboard.writeText(jsonResult); setCopied("json"); window.setTimeout(() => setCopied(""), 1600); }
    catch { setJsonError("Não foi possível copiar para a área de transferência."); }
  };
  const words = textValue.trim() ? textValue.trim().split(/\s+/).length : 0;
  const copyLabel = (key: string) => copied === key ? <><Check size={14}/> Copiado</> : <><Copy size={14}/> Copiar</>;
  return <section className="toolbox">
    <div className="toolbox-intro"><div className="module-icon"><Type size={21}/></div><div><h3>Ferramentas de texto</h3><p>Utilidades rápidas que funcionam localmente, sem enviar seus dados para a internet.</p></div></div>
    <div className="toolbox-grid">
      <article className="tool-card text-tool-card">
        <div className="tool-card-heading"><div><span className="tool-kicker">01 / TEXTO</span><h3>Transformador de texto</h3></div><Type size={18}/></div>
        <textarea value={textValue} onChange={e => setTextValue(e.target.value)} placeholder="Digite ou cole seu texto aqui..." rows={7}/>
        <div className="text-stats"><span>{words} palavras</span><span>{textValue.length} caracteres</span><span>{textValue.split(/\n/).length} linhas</span></div>
        <div className="tool-actions">
          <button onClick={() => void transform(textValue.toUpperCase(),"upper")}>MAIÚSCULAS</button>
          <button onClick={() => void transform(textValue.toLowerCase(),"lower")}>minúsculas</button>
          <button onClick={() => void transform(textValue.replace(/\s+/g," ").trim(),"clean")}>Limpar espaços</button>
          <button className="copy-action" onClick={() => void transform(textValue,"text")}>{copyLabel("text")}</button>
        </div>
        {copied === "error" && <small className="tool-error">A área de transferência não está disponível.</small>}
      </article>
      <article className="tool-card json-tool-card">
        <div className="tool-card-heading"><div><span className="tool-kicker">02 / DESENVOLVIMENTO</span><h3>Formatador JSON</h3></div><Braces size={18}/></div>
        <textarea value={jsonValue} onChange={e => setJsonValue(e.target.value)} placeholder={'Cole seu JSON aqui...'} rows={5}/>
        <div className="tool-actions"><button className="primary-button compact" onClick={formatJson} disabled={!jsonValue.trim()}>Formatar JSON</button><button onClick={() => {setJsonValue("");setJsonResult("");setJsonError("");}}>Limpar</button></div>
        {jsonError && <div className="tool-error">{jsonError}</div>}
        {jsonResult && <div className="json-output"><div><span>RESULTADO FORMATADO</span><button onClick={() => void copyJson()}>{copyLabel("json")}</button></div><pre>{jsonResult}</pre></div>}
      </article>
    </div>
    <div className="toolbox-note"><ShieldCheck size={15}/> Todo o processamento acontece no próprio aplicativo. Nada é enviado para servidores.</div>
  </section>;
}

function ModuleCard({ icon: Icon, title, desc, meta, onClick }: { icon: LucideIcon; title: string; desc: string; meta: string; onClick: () => void }) {
  return <button className="module-card" onClick={onClick}><div className="module-card-top"><div className="module-icon"><Icon size={21}/></div><ArrowUpRight size={16} className="card-arrow"/></div><div className="module-meta">{meta}</div><h4>{title}</h4><p>{desc}</p><div className="card-link">ABRIR MÓDULO <ChevronRight size={13}/></div></button>;
}

function ModulePlaceholder({ id, onBack }: { id: ModuleId; onBack: () => void }) {
  const info: Record<ModuleId, { icon: LucideIcon; label: string; items: string[] }> = {
    overview: { icon: LayoutDashboard, label: "Visão geral", items: [] },
    files: { icon: FileSearch, label: "File Manager", items: ["Explorador de arquivos", "Localizador de duplicados", "Análise de arquivos grandes", "Renomeação em lote"] },
    toolbox: { icon: Image, label: "Toolbox", items: ["Conversor e redimensionador de imagens", "Gerador de QR Codes", "Ferramentas de texto", "Conversor de unidades"] },
    games: { icon: Gamepad2, label: "GameHub", items: ["Biblioteca unificada", "Detecção de launchers", "Favoritos e categorias", "Histórico de sessões"] },
    saves: { icon: HardDrive, label: "Save Center", items: ["Perfis de salvamento", "Backups manuais", "Histórico de versões", "Restauração assistida"] },
    settings: { icon: Settings2, label: "Configurações", items: ["Aparência e tema", "Preferências de inicialização", "Privacidade e permissões", "Sobre o aplicativo"] }
  };
  const data = info[id]; const Icon = data.icon;
  return <section className="placeholder"><div className="placeholder-icon"><Icon size={30}/></div><div className="eyebrow">MÓDULO PREPARADO</div><h2>{data.label}</h2><p>Esta área já está conectada à navegação. As funcionalidades serão implementadas em etapas, com testes próprios.</p><div className="planned-list">{data.items.map(item => <div key={item}><CheckCircle2 size={15}/>{item}</div>)}</div><button className="secondary-button" onClick={onBack}>Voltar ao dashboard</button></section>;
}

export default App;