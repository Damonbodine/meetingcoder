import React, { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";

type AudioSource = string; // "microphone" or "system:<device>"

interface AudioMetrics {
  buffer_fill_percent: number;
  buffer_capacity_samples: number;
  buffer_size_samples: number;
  device_sample_rate: number;
  resample_ratio: number;
  overwritten_samples?: number;
}

export const StatusBar: React.FC = () => {
  const [metrics, setMetrics] = useState<AudioMetrics | null>(null);
  const [source, setSource] = useState<AudioSource>("-");
  const [modelLoaded, setModelLoaded] = useState<boolean>(false);
  const lastOverwrittenRef = useRef<number>(0);
  const warnedRef = useRef<boolean>(false);

  useEffect(() => {
    let cancelled = false;
    const tick = async () => {
      try {
        const m = await invoke<AudioMetrics>("get_audio_metrics");
        const s = await invoke<string>("get_current_audio_source");
        const ms: any = await invoke("get_model_load_status");
        if (!cancelled) {
          setMetrics(m);
          setSource(s);
          setModelLoaded(Boolean(ms?.is_loaded));

          // High-water alerts: buffer fill >70% or overwritten count increases
          const fill = (m as any)?.buffer_fill_percent ?? 0;
          const overwritten = (m as any)?.overwritten_samples ?? 0;
          const lastOver = lastOverwrittenRef.current;
          const overwrittenIncreased = overwritten > lastOver;
          lastOverwrittenRef.current = overwritten || 0;
          const shouldWarn = fill > 70 || overwrittenIncreased;
          if (shouldWarn && !warnedRef.current) {
            warnedRef.current = true;
            toast.warning(
              "Audio buffer is getting full. Consider increasing System Audio Buffer Size.",
              {
                action: {
                  label: "Open Advanced",
                  onClick: () => {
                    window.dispatchEvent(
                      new CustomEvent("navigate-to-section", { detail: "advanced" })
                    );
                  },
                },
              }
            );
            // Reset warning flag after some time to avoid spamming
            setTimeout(() => { warnedRef.current = false; }, 15000);
          }
        }
      } catch (_) { /* ignore */ }
    };
    tick();
    const id = setInterval(tick, 2000);
    return () => { cancelled = true; clearInterval(id); };
  }, []);

  const deviceLabel = source.startsWith("system:") ? source.replace("system:", "") : source;
  const fill = metrics ? `${metrics.buffer_fill_percent.toFixed(0)}%` : "-";
  const warn = metrics ? (metrics.buffer_fill_percent > 70) : false;
  const stateItems = [
    `Source: ${deviceLabel || "-"}`,
    `Buffer: ${fill}`,
    `Model: ${modelLoaded ? "Loaded" : "Not loaded"}`,
  ];

  return (
    <div className="text-xs opacity-80 flex items-center gap-4">
      {warn && (
        <span className="px-2 py-1 rounded bg-yellow-500/20 border border-yellow-500/40 text-yellow-300">
          Buffer High
        </span>
      )}
      {stateItems.map((t, i) => (
        <span key={i} className="px-2 py-1 rounded bg-mid-gray/10 border border-mid-gray/20">
          {t}
        </span>
      ))}
    </div>
  );
};

export default StatusBar;
