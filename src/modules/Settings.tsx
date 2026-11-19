import { useEffect, useState } from "react";
import { Check, LoaderCircle, Moon, Monitor, Sun } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";

type SettingsValue = {
  theme: "dark" | "light" | "system";
  defaultFolder: string;
  confirmBeforeDelete: boolean;
  openLastFolder: boolean;
  lastFolder: string;
};

const defaults: SettingsValue = {
  theme: "dark",
  defaultFolder: "",
  confirmBeforeDelete: true,
  openLastFolder: false,
  lastFolder: ""
};

export default function Settings() {
  const [value, setValue] = useState<SettingsValue>(defaults);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");

  useEffect(() => {
    let active = true;
    invoke<SettingsValue>("load_settings")
      .then(settings => { if (active) setValue(settings); })
      .catch(() => { if (active) setMessage("As preferências padrão estão em uso."); })
      .finally(() => { if (active) setLoading(false); });
    return () => { active = false; };
  }, []);

  useEffect(() => {
    document.documentElement.dataset.theme = value.theme;
  }, [value.theme]);

  const update = <K extends keyof SettingsValue>(key: K, next: SettingsValue[K]) =>
    setValue(current => ({ ...current, [key]: next }));

  const save = async () => {
    setSaving(true); setError(""); setMessage("");
    try {
      await invoke("save_settings", { settings: value });
      setMessage("Preferências salvas neste computador.");
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  };

  if (loading) return <div className="settings-loading"><LoaderCircle className="spin"/> Carregando preferências...</div>;

  return <section className="settings-page">
    <article className="settings-card">
      <div className="settings-heading"><div><span className="tool-kicker">APARÊNCIA</span><h3>Tema do aplicativo</h3><p>Escolha como o DIAMANDIS ONE deve aparecer.</p></div></div>
      <div className="theme-options">
        <button className={value.theme === "dark" ? "theme-option selected" : "theme-option"} onClick={() => update("theme", "dark")}><Moon size={19}/><strong>Escuro</strong><span>Contraste suave</span></button>
        <button className={value.theme === "light" ? "theme-option selected" : "theme-option"} onClick={() => update("theme", "light")}><Sun size={19}/><strong>Claro</strong><span>Ambiente iluminado</span></button>
        <button className={value.theme === "system" ? "theme-option selected" : "theme-option"} onClick={() => update("theme", "system")}><Monitor size={19}/><strong>Sistema</strong><span>Seguir o Windows</span></button>
      </div>
    </article>

    <article className="settings-card">
      <div className="settings-heading"><div><span className="tool-kicker">FILE MANAGER</span><h3>Preferências de arquivos</h3><p>Defina o comportamento padrão do explorador.</p></div></div>
      <label className="settings-field"><span>Pasta inicial preferida</span><input value={value.defaultFolder} onChange={e => update("defaultFolder", e.target.value)} placeholder="Ex.: C:\\Users\\Você\\Documents"/><small>Deixe vazio para abrir sua pasta de usuário. O caminho será validado ao abrir.</small></label>
      <label className="settings-toggle"><span><strong>Confirmar antes de enviar à Lixeira</strong><small>Evita exclusões acidentais.</small></span><input type="checkbox" checked={value.confirmBeforeDelete} onChange={e => update("confirmBeforeDelete", e.target.checked)}/></label>
      <label className="settings-toggle"><span><strong>Reabrir a última pasta utilizada</strong><small>Retoma o local de trabalho na próxima sessão.</small></span><input type="checkbox" checked={value.openLastFolder} onChange={e => update("openLastFolder", e.target.checked)}/></label>
    </article>

    <div className="settings-footer">
      <div className={error ? "settings-feedback error" : "settings-feedback"}>{error || message}</div>
      <button className="primary-button compact" onClick={() => void save()} disabled={saving}>{saving ? <LoaderCircle className="spin" size={14}/> : <Check size={14}/>} Salvar preferências</button>
    </div>
  </section>;
}
