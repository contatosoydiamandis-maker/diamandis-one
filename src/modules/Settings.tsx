import { useEffect, useState } from "react";
import { Check, LoaderCircle, Moon, Monitor, Sun } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { setThemePreference } from "../theme";
import { validatePluginManifest } from "../plugins/manifest";

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
  const [pluginJson, setPluginJson] = useState('{"id":"com.example.sample","name":"Plugin de exemplo","version":"1.0.0","apiVersion":1,"description":"Uma extensão local de exemplo.","permissions":["ui.panel"]}');
  const [pluginResult, setPluginResult] = useState("");
  const [pluginValid, setPluginValid] = useState<boolean | null>(null);

  useEffect(() => {
    let active = true;
    invoke<SettingsValue>("load_settings")
      .then(settings => { if (active) setValue(settings); })
      .catch(() => { if (active) setMessage("As preferências padrão estão em uso."); })
      .finally(() => { if (active) setLoading(false); });
    return () => { active = false; };
  }, []);

  useEffect(() => {
    setThemePreference(value.theme);
  }, [value.theme]);

  const update = <K extends keyof SettingsValue>(key: K, next: SettingsValue[K]) =>
    setValue(current => ({ ...current, [key]: next }));

  const validatePlugin = () => {
    try {
      const result = validatePluginManifest(JSON.parse(pluginJson));
      setPluginValid(result.valid);
      setPluginResult(result.valid
        ? `Manifesto válido: ${result.manifest.name} • API ${result.manifest.apiVersion} • ${result.manifest.permissions.length} permissão(ões) solicitada(s).`
        : result.error);
    } catch {
      setPluginValid(false);
      setPluginResult("JSON inválido. Revise a sintaxe do manifesto.");
    }
  };

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
        <button className={value.theme === "dark" ? "theme-option selected" : "theme-option"} aria-pressed={value.theme === "dark"} onClick={() => update("theme", "dark")}><Moon size={19}/><strong>Escuro</strong><span>Contraste suave</span></button>
        <button className={value.theme === "light" ? "theme-option selected" : "theme-option"} aria-pressed={value.theme === "light"} onClick={() => update("theme", "light")}><Sun size={19}/><strong>Claro</strong><span>Ambiente iluminado</span></button>
        <button className={value.theme === "system" ? "theme-option selected" : "theme-option"} aria-pressed={value.theme === "system"} onClick={() => update("theme", "system")}><Monitor size={19}/><strong>Sistema</strong><span>Seguir o Windows</span></button>
      </div>
    </article>

    <article className="settings-card">
      <div className="settings-heading"><div><span className="tool-kicker">FILE MANAGER</span><h3>Preferências de arquivos</h3><p>Defina o comportamento padrão do explorador.</p></div></div>
      <label className="settings-field"><span>Pasta inicial preferida</span><input value={value.defaultFolder} onChange={e => update("defaultFolder", e.target.value)} placeholder="Ex.: C:\\Users\\Você\\Documents"/><small>Deixe vazio para abrir sua pasta de usuário. O caminho será validado ao abrir.</small></label>
      <label className="settings-toggle"><span><strong>Confirmar antes de enviar à Lixeira</strong><small>Evita exclusões acidentais.</small></span><input type="checkbox" checked={value.confirmBeforeDelete} onChange={e => update("confirmBeforeDelete", e.target.checked)}/></label>
      <label className="settings-toggle"><span><strong>Reabrir a última pasta utilizada</strong><small>Retoma o local de trabalho na próxima sessão.</small></span><input type="checkbox" checked={value.openLastFolder} onChange={e => update("openLastFolder", e.target.checked)}/></label>
    </article>

    <article className="settings-card">
      <div className="settings-heading"><div><span className="tool-kicker">EXTENSIBILIDADE • API V1</span><h3>Validador de plugins</h3><p>Confira o manifesto JSON de uma extensão antes de integrá-la ao ecossistema DIAMANDIS ONE.</p></div></div>
      <label className="settings-field"><span>Manifesto JSON</span><textarea value={pluginJson} onChange={e => { setPluginJson(e.target.value); setPluginValid(null); setPluginResult(""); }} rows={7} spellCheck={false}/><small>Permissões reconhecidas: ui.panel, tool.action, filesystem.read, filesystem.write e process.launch.</small></label>
      <div className="settings-actions"><button className="secondary-button" onClick={() => setPluginJson('{"id":"com.example.sample","name":"Plugin de exemplo","version":"1.0.0","apiVersion":1,"description":"Uma extensão local de exemplo.","permissions":["ui.panel"]}')}>Exemplo</button><button className="primary-button compact" onClick={validatePlugin}>Validar manifesto</button></div>
      {pluginResult && <div className={pluginValid ? "settings-feedback" : "settings-feedback error"} role="status">{pluginResult}</div>}
      <small className="settings-hint">A validação de manifestos está disponível. O carregamento e a execução dinâmica de plugins externos permanecem desativados nesta pré-alpha por segurança.</small>
    </article>

    <div className="settings-footer">
      <div className={error ? "settings-feedback error" : "settings-feedback"} role={error ? "alert" : "status"}>{error || message}</div>
      <button className="primary-button compact" onClick={() => void save()} disabled={saving}>{saving ? <LoaderCircle className="spin" size={14}/> : <Check size={14}/>} Salvar preferências</button>
    </div>
  </section>;
}
