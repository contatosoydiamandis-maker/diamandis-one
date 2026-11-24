import { useCallback, useEffect, useState } from "react";
import { Activity, Cpu, HardDrive, MemoryStick, RefreshCw, Usb } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";

type StorageDrive = {
  letter: string;
  name: string;
  driveType: string;
  totalBytes: number;
  freeBytes: number;
  capacityAvailable: boolean;
};

type SystemMetrics = {
  cpuUsagePercent: number | null;
  logicalCores: number | null;
  memoryTotalBytes: number;
  memoryAvailableBytes: number;
  storageTotalBytes: number | null;
  storageFreeBytes: number | null;
  storageDrives: StorageDrive[];
};

let metricsRequest: Promise<SystemMetrics> | null = null;

function loadMetrics(): Promise<SystemMetrics> {
  if (!metricsRequest) {
    metricsRequest = invoke<SystemMetrics>("get_system_metrics").finally(() => {
      metricsRequest = null;
    });
  }
  return metricsRequest;
}

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

function DriveCard({ drive }: { drive: StorageDrive }) {
  const usedBytes = Math.max(0, drive.totalBytes - drive.freeBytes);
  const percent = drive.totalBytes > 0 ? Math.min(100, usedBytes / drive.totalBytes * 100) : 0;
  const Icon = drive.driveType === "Removível" ? Usb : HardDrive;

  return (
    <article className="system-drive-card">
      <div className="system-drive-heading">
        <span className="system-metric-icon"><Icon size={16}/></span>
        <div><strong>{drive.name}</strong><small>{drive.letter} · {drive.driveType}</small></div>
        <span className="system-drive-percent">{Math.round(percent)}%</span>
      </div>
      {drive.capacityAvailable ? <>
        <div className="system-drive-track" role="progressbar" aria-label={`Espaço usado em ${drive.letter}`} aria-valuemin={0} aria-valuemax={100} aria-valuenow={Math.round(percent)}>
          <i style={{ width: `${percent}%` }}/>
        </div>
        <div className="system-drive-capacity">
          <span>{formatBytes(usedBytes)} usados</span>
          <span>{formatBytes(drive.freeBytes)} livres de {formatBytes(drive.totalBytes)}</span>
        </div>
      </> : <div className="system-drive-capacity"><span>Capacidade indisponível no momento</span><span>{drive.driveType}</span></div>}
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
      const result = await loadMetrics();
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
        const result = await loadMetrics();
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
  const drives = metrics?.storageDrives ?? [];
  const storageTotal = metrics?.storageTotalBytes ?? null;
  const storageFree = metrics?.storageFreeBytes ?? null;
  const storageUsed = storageTotal !== null && storageFree !== null
    ? Math.max(0, storageTotal - storageFree)
    : null;
  const storagePercent = storageUsed !== null && storageTotal
    ? storageUsed / storageTotal * 100
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
        <MetricCard icon={HardDrive} label="ARMAZENAMENTO" value={storageUsed === null ? "Indisponível" : formatBytes(storageUsed)} detail={storageUsed === null ? "Nenhuma unidade local disponível" : `${drives.length} unidades · ${formatBytes(storageFree!)} livres no total`} percent={storagePercent}/>
        <MetricCard icon={Activity} label="GPU" value="Não disponível" detail="Leitura de GPU será adicionada com suporte por fabricante." unavailable/>
      </div>
      {drives.length > 0 && (
        <div className="system-drives">
          <div className="system-drives-heading"><h4>Unidades de armazenamento</h4><span>{drives.length} {drives.length === 1 ? "unidade detectada" : "unidades detectadas"}</span></div>
          <div className="system-drive-grid">{drives.map((drive) => <DriveCard key={drive.letter} drive={drive}/>)}</div>
        </div>
      )}
      <div className="system-dashboard-foot"><span><i/> Monitoramento local</span><span>CPU • RAM • {drives.length} unidades</span></div>
    </section>
  );
}
