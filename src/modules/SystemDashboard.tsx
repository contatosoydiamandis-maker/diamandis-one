import { useCallback, useEffect, useState } from "react";
import { Activity, Cpu, HardDrive, MemoryStick, RefreshCw } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";

type SystemMetrics = {
  cpuUsagePercent: number | null;
  logicalCores: number | null;
  memoryTotalBytes: number;
  memoryAvailableBytes: number;
  storageTotalBytes: number | null;
  storageFreeBytes: number | null;
};

const formatBytes = (bytes: number) => {
  if (!Number.isFinite(bytes) || bytes < 0) return "Indisponível";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit++;
  }
  return `${value.toFixed(unit >= 3 ? 1 : 0)} ${units[unit]}`;
};

function MetricCard({
  icon: Icon,
  label,
  value,
  detail,
  percent,
  unavailable = false
}: {
  icon: typeof Cpu;
  label: string;
  value: string;
  detail: string;
  percent?: number | null;
  unavailable?: boolean;
}) {
  const safePercent = typeof percent === "number" && Number.isFinite(percent)
    ? Math.max(0, Math.min(100, percent))
    : null;

  return (
    <article className={`system-metric-card ${unavailable ? "unavailable" : ""}`}>
      <div className="system-metric-top"><span className="system-metric-icon"><Icon size={17}/></span><span>{label}</span></div>
      <strong className="system-metric-value">{value}</strong>
      <small>{detail}</small>
      {safePercent !== null && <div className="system-metric-track" role="progressbar" aria-label={label} aria-valuemin={0} aria-valuemax={100} aria-valuenow={Math.round(safePercent)}><i style={{ width: `${safePercent}%` }}/></div>}
    </article>
  );
}

export default function SystemDashboard() {
  const [metrics, setMetrics] = useState<SystemMetrics | null>(null);
  const [error, setError] = useState("");
  const [refreshing, setRefreshing] = useState(false);

  const refresh = useCallback(async () => {
    setRefreshing(true);
    try {
      const result = await invoke<SystemMetrics>("get_system_metrics");
      setMetrics(result);
      setError("");
    } catch (reason) {
      setError(String(reason));
    } finally {
      setRefreshing(false);
    }
  }, []);

  useEffect(() => {
    let active = true;
    let timeout: number | undefined;
    const poll = async () => {
      try {
        const result = await invoke<SystemMetrics>("get_system_metrics");
        if (active) {
          setMetrics(result);
          setError("");
        }
      } catch (reason) {
        if (active) setError(String(reason));
      } finally {
        if (active) timeout = window.setTimeout(poll, 2500);
      }
    };
    void poll();
    return () => {
      active = false;
      if (timeout !== undefined) window.clearTimeout(timeout);
    };
  }, []);

  const memoryUsed = metrics ? Math.max(0, metrics.memoryTotalBytes - metrics.memoryAvailableBytes) : 0;
  const memoryPercent = metrics && metrics.memoryTotalBytes > 0
    ? memoryUsed / metrics.memoryTotalBytes * 100
    : null;
  const storageUsed = metrics?.storageTotalBytes !== null && metrics?.storageTotalBytes !== undefined
    && metrics.storageFreeBytes !== null && metrics.storageFreeBytes !== undefined
    ? Math.max(0, metrics.storageTotalBytes - metrics.storageFreeBytes)
    : null;
  const storagePercent = storageUsed !== null && metrics?.storageTotalBytes
    ? storageUsed / metrics.storageTotalBytes * 100
    : null;

  return (
    <section className="system-dashboard">
      <div className="section-title system-dashboard-heading">
        <div><h3>Monitoramento do computador</h3><p>Leituras nativas do Windows, atualizadas automaticamente.</p></div>
        <button className="toolbar-button" onClick={() => void refresh()} disabled={refreshing} title="Atualizar métricas" aria-label="Atualizar métricas"><RefreshCw size={15} className={refreshing ? "spin" : ""}/></button>
      </div>
      {error && <div className="system-metric-error" role="status">{error}. Algumas métricas podem não estar disponíveis neste ambiente.</div>}
      <div className="system-metric-grid">
        <MetricCard icon={Cpu} label="PROCESSADOR" value={metrics?.cpuUsagePercent == null ? "Calculando…" : `${metrics.cpuUsagePercent.toFixed(0)}%`} detail={metrics ? (metrics.logicalCores === null ? "Contagem de processadores indisponível" : `${metrics.logicalCores} processadores lógicos`) : "Aguardando leitura do sistema"} percent={metrics?.cpuUsagePercent}/>
        <MetricCard icon={MemoryStick} label="MEMÓRIA RAM" value={metrics ? formatBytes(memoryUsed) : "Carregando…"} detail={metrics ? `${formatBytes(metrics.memoryAvailableBytes)} disponíveis de ${formatBytes(metrics.memoryTotalBytes)}` : "Aguardando leitura do sistema"} percent={memoryPercent}/>
        <MetricCard icon={HardDrive} label="ARMAZENAMENTO" value={storageUsed === null ? "Indisponível" : formatBytes(storageUsed)} detail={storageUsed === null ? "O Windows não retornou dados da unidade do sistema" : `${formatBytes(metrics!.storageFreeBytes!)} livres de ${formatBytes(metrics!.storageTotalBytes!)}`} percent={storagePercent}/>
        <MetricCard icon={Activity} label="GPU" value="Não disponível" detail="Leitura de GPU será adicionada com suporte por fabricante." unavailable/>
      </div>
      <div className="system-dashboard-foot"><span><i/> Monitoramento local</span><span>CPU • RAM • Unidade do sistema</span></div>
    </section>
  );
}
