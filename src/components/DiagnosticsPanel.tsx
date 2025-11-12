import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface AudioMetrics {
  buffer_size_samples: number;
  buffer_capacity_samples: number;
  buffer_fill_percent: number;
  overwritten_samples: number;
  device_name: string;
  device_sample_rate: number;
  resample_ratio: number;
  silent_chunks: number;
  restart_attempts_total: number;
  restarts_last_hour: number;
  restart_cooldown_remaining_secs: number;
  last_restart_error?: string | null;
  is_system_capturing: boolean;
  backlog_seconds_estimate: number;
  queue_queued: number;
  queue_processing: number;
  queue_backlog_seconds: number;
}

interface ModelStatus { is_loaded: boolean; current_model?: string | null }

export const DiagnosticsPanel: React.FC = () => {
  const [metrics, setMetrics] = useState<AudioMetrics | null>(null);
  const [modelStatus, setModelStatus] = useState<ModelStatus | null>(null);
  const [restartAttempts, setRestartAttempts] = useState<number>(0);
  const [lastErrors, setLastErrors] = useState<string[]>([]);

  useEffect(() => {
    let cancel = false;
    const fetchMetrics = async () => {
      try {
        const m = await invoke<AudioMetrics>("get_audio_metrics");
        if (!cancel) setMetrics(m);
        const s = await invoke<any>("get_model_load_status");
        if (!cancel) setModelStatus(s as ModelStatus);
        // Fetch recent errors persisted on backend
        const errs = await invoke<string[]>("get_audio_errors");
        if (!cancel) setLastErrors(errs || []);
      } catch (e) {
        // noop
      }
    };
    fetchMetrics();
    const t = setInterval(fetchMetrics, 2000);
    return () => { cancel = true; clearInterval(t); };
  }, []);

  useEffect(() => {
    const unsubs: (() => void)[] = [];
    listen("audio-stream-restarting", (_ev) => {
      setRestartAttempts((c) => c + 1);
    }).then((u) => unsubs.push(u));
    listen("audio-stream-restart-failed", (ev) => {
      const payload: any = ev.payload;
      const msg = `Restart failed: ${payload?.error ?? "unknown"}`;
      setLastErrors((arr) => [msg, ...arr].slice(0, 10));
    }).then((u) => unsubs.push(u));
    return () => { unsubs.forEach((u) => u()); };
  }, []);

  const fillPct = metrics ? metrics.buffer_fill_percent.toFixed(1) : "-";
  return (
    <div className="max-w-3xl w-full mx-auto space-y-4 p-4">
      <h2 className="text-lg font-semibold">Diagnostics</h2>
      <div className="grid grid-cols-2 gap-4">
        <div className="p-3 rounded border border-mid-gray/30">
          <div className="font-medium mb-2">Buffer</div>
          <div className="text-sm">Size: {metrics?.buffer_size_samples ?? "-"} samples</div>
          <div className="text-sm">Capacity: {metrics?.buffer_capacity_samples ?? "-"} samples</div>
          <div className="text-sm">Fill: {fillPct}%</div>
          <div className="text-sm">Overwritten: {metrics?.overwritten_samples ?? "-"}</div>
          <div className="text-sm">Backlog est.: {metrics ? metrics.backlog_seconds_estimate.toFixed(1) : "-"}s</div>
        </div>
        <div className="p-3 rounded border border-mid-gray/30">
          <div className="font-medium mb-2">Resampler</div>
          <div className="text-sm">Device: {metrics?.device_name ?? "-"}</div>
          <div className="text-sm">Device rate: {metrics?.device_sample_rate ?? "-"} Hz</div>
          <div className="text-sm">Resample ratio: x{metrics?.resample_ratio?.toFixed(3) ?? "-"}</div>
          <div className="text-sm">Capturing: {metrics?.is_system_capturing ? "Yes" : "No"}</div>
        </div>
        <div className="p-3 rounded border border-mid-gray/30">
          <div className="font-medium mb-2">Queue</div>
          <div className="text-sm">Queued: {metrics?.queue_queued ?? 0}</div>
          <div className="text-sm">Processing: {metrics?.queue_processing ?? 0}</div>
          <div className="text-sm">Backlog: {metrics ? metrics.queue_backlog_seconds.toFixed(1) : "-"}s</div>
        </div>
        <div className="p-3 rounded border border-mid-gray/30">
          <div className="font-medium mb-2">Model</div>
          <div className="text-sm">Loaded: {modelStatus?.is_loaded ? "Yes" : "No"}</div>
          <div className="text-sm">Current: {modelStatus?.current_model ?? "-"}</div>
        </div>
        <div className="p-3 rounded border border-mid-gray/30">
          <div className="font-medium mb-2">Restarts</div>
          <div className="text-sm">Attempts (session): {restartAttempts}</div>
          <div className="text-sm">Attempts (total): {metrics?.restart_attempts_total ?? 0}</div>
          <div className="text-sm">Attempts (last hour): {metrics?.restarts_last_hour ?? 0}</div>
          <div className="text-sm">Cooldown remaining: {metrics?.restart_cooldown_remaining_secs ?? 0}s</div>
          <div className="text-sm">Silent chunks: {metrics?.silent_chunks ?? 0}</div>
          <div className="text-sm">Errors:</div>
          <ul className="text-xs list-disc pl-5 space-y-1 max-h-28 overflow-auto">
            {lastErrors.length === 0 && <li>None</li>}
            {lastErrors.map((e, i) => (
              <li key={i}>{e}</li>
            ))}
          </ul>
        </div>
      </div>
    </div>
  );
};

export default DiagnosticsPanel;
