import React, { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { listen } from "@tauri-apps/api/event";
import ProgressBar from "../shared/ProgressBar";
import { ImportToolStatus } from "../../lib/types";
import { useSettings } from "../../hooks/useSettings";

interface InstallHint {
  label: string;
  command: string;
}

const YT_DLP_HINTS: InstallHint[] = [
  { label: "macOS (Homebrew)", command: "brew install yt-dlp" },
  { label: "Windows (winget)", command: "winget install yt-dlp.yt-dlp" },
  { label: "Linux (pipx)", command: "pipx install yt-dlp" },
];

const FFMPEG_HINTS: InstallHint[] = [
  { label: "macOS (Homebrew)", command: "brew install ffmpeg" },
  { label: "Windows (winget)", command: "winget install Gyan.FFmpeg" },
  { label: "Ubuntu/Debian", command: "sudo apt install ffmpeg" },
];

const StatusPill = ({ ready, label }: { ready: boolean; label: string }) => (
  <span
    className={`text-xs px-2 py-0.5 rounded-full border ${
      ready
        ? "border-green-500/60 text-green-300 bg-green-500/10"
        : "border-red-500/60 text-red-300 bg-red-500/10"
    }`}
  >
    {label}
  </span>
);

const InstallList = ({ hints }: { hints: InstallHint[] }) => (
  <ul className="mt-2 space-y-1 text-xs">
    {hints.map((hint) => (
      <li key={hint.label} className="flex flex-col sm:flex-row sm:items-center sm:gap-2">
        <span className="text-muted-foreground">{hint.label}</span>
        <code className="font-mono bg-black/30 px-2 py-0.5 rounded border border-border text-[11px]">
          {hint.command}
        </code>
      </li>
    ))}
  </ul>
);

export const ImportAudio: React.FC = () => {
  const { getSetting } = useSettings();
  const offlineMode = getSetting("offline_mode_enabled") ?? false;
  const [meetingName, setMeetingName] = useState("");
  const [isImporting, setIsImporting] = useState(false);
  const [ytUrl, setYtUrl] = useState("");
  const [filePath, setFilePath] = useState("");
  const [progressStage, setProgressStage] = useState<string | null>(null);
  const [progressPercent, setProgressPercent] = useState<number | null>(null);
  const [lastTranscriptDir, setLastTranscriptDir] = useState<string | null>(null);
  const [toolStatus, setToolStatus] = useState<ImportToolStatus | null>(null);
  const [toolError, setToolError] = useState<string | null>(null);
  const [checkingTools, setCheckingTools] = useState(true);

  useEffect(() => {
    let unlisten: (() => void) | null = null;
    (async () => {
      try {
        unlisten = await listen<{ stage: string; percent?: number }>(
          "import-progress",
          (ev) => {
            const { stage, percent } = ev.payload;
            setProgressStage(stage);
            setProgressPercent(
              typeof percent === "number" ? Math.max(0, Math.min(100, percent)) : null,
            );
          },
        );
      } catch (e) {
        console.error("Failed to listen for import-progress:", e);
      }
    })();
    return () => {
      if (unlisten) {
        try {
          unlisten();
        } catch (_) {
          /* ignore */
        }
      }
    };
  }, []);

  const refreshToolStatus = async () => {
    setCheckingTools(true);
    try {
      const status = await invoke<ImportToolStatus>("get_import_tool_status");
      setToolStatus(status);
      setToolError(null);
    } catch (e) {
      setToolError(String(e));
    } finally {
      setCheckingTools(false);
    }
  };

  useEffect(() => {
    refreshToolStatus();
  }, []);

  const toolsOffline = offlineMode || (toolStatus?.offline_mode ?? false);
  const ytAvailable = !toolsOffline && !!toolStatus?.yt_dlp.installed;
  const ffmpegReady = !!toolStatus?.ffmpeg.installed;

  const youtubeDisableReason = useMemo(() => {
    if (toolsOffline) return "Offline mode disables YouTube import";
    if (toolStatus && !toolStatus.yt_dlp.installed) {
      return "Install yt-dlp to enable YouTube import";
    }
    return null;
  }, [toolsOffline, toolStatus]);

  const fileDisableReason = useMemo(() => {
    if (!toolStatus) return "Checking ffmpeg…";
    if (!ffmpegReady) return "Install ffmpeg to enable file imports";
    return null;
  }, [ffmpegReady, toolStatus]);

  const importFromPath = async () => {
    if (!ffmpegReady) {
      toast.error("Install ffmpeg to import files");
      return;
    }
    try {
      if (!meetingName.trim()) {
        toast.error("Enter a meeting name");
        return;
      }
      const path = filePath.trim();
      if (!path) {
        toast.error("Enter an audio file path");
        return;
      }
      setIsImporting(true);
      const summary = await invoke<any>("import_audio_as_meeting", {
        meetingName,
        filePath: path,
      });
      try {
        const transcriptDir = await invoke<string>("get_transcript_dir_for", {
          meetingName,
          startTime: summary.start_time,
        });
        setLastTranscriptDir(transcriptDir);
        toast.success("Imported meeting", {
          description: `Segments: ${summary.total_segments}\nSaved to: ${transcriptDir}`,
        });
      } catch {
        toast.success("Imported meeting", {
          description: `Segments: ${summary.total_segments}`,
        });
      }
      setProgressStage(null);
      setProgressPercent(null);
    } catch (e) {
      console.error(e);
      toast.error("Import failed", { description: String(e) });
    } finally {
      setIsImporting(false);
    }
  };

  const importYoutube = async () => {
    if (!ytAvailable) {
      toast.error("YouTube import is disabled", {
        description: youtubeDisableReason ?? "Install prerequisites first.",
      });
      return;
    }
    try {
      if (!meetingName.trim()) {
        toast.error("Enter a meeting name");
        return;
      }
      if (!ytUrl.trim()) {
        toast.error("Enter a YouTube URL");
        return;
      }
      setIsImporting(true);
      const summary = await invoke<any>("import_youtube_as_meeting", {
        meetingName,
        url: ytUrl,
      });
      try {
        const transcriptDir = await invoke<string>("get_transcript_dir_for", {
          meetingName,
          startTime: summary.start_time,
        });
        setLastTranscriptDir(transcriptDir);
        toast.success("Imported from YouTube", {
          description: `Segments: ${summary.total_segments}\nSaved to: ${transcriptDir}`,
        });
      } catch {
        toast.success("Imported from YouTube", {
          description: `Segments: ${summary.total_segments}`,
        });
      }
      setProgressStage(null);
      setProgressPercent(null);
    } catch (e) {
      console.error(e);
      const msg = String(e);
      toast.error("YouTube import failed", { description: msg });
    } finally {
      setIsImporting(false);
    }
  };

  const browseAndSetPath = async () => {
    try {
      const picked = await invoke<string | null>("pick_audio_file");
      if (picked) setFilePath(picked);
    } catch (e) {
      console.error("File picker failed:", e);
      toast.error("Could not open file picker", { description: String(e) });
    }
  };

  const renderToolCard = (
    title: string,
    ready: boolean,
    description: string,
    hints: InstallHint[],
    extra?: React.ReactNode,
  ) => (
    <div className="rounded border border-border p-3">
      <div className="flex items-center justify-between">
        <div>
          <div className="text-sm font-medium">{title}</div>
          <p className="text-xs text-muted-foreground">{description}</p>
        </div>
        <StatusPill ready={ready} label={ready ? "Ready" : "Needs setup"} />
      </div>
      {!ready && <InstallList hints={hints} />}
      {extra}
    </div>
  );

  return (
    <div className="rounded-md border border-border p-4 space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <div className="text-sm font-medium">Import Audio into MeetingCoder</div>
          <p className="text-xs text-muted-foreground">
            Keep yt-dlp and ffmpeg handy so imports stay reliable.
          </p>
        </div>
        <button
          onClick={refreshToolStatus}
          disabled={checkingTools}
          className="text-xs px-3 py-1 rounded border border-border hover:bg-background-ui disabled:opacity-50"
        >
          {checkingTools ? "Checking…" : "Re-check tools"}
        </button>
      </div>

      {toolError && (
        <div className="text-xs text-red-400">
          {toolError}
        </div>
      )}

      <div className="space-y-2">
        {renderToolCard(
          "yt-dlp (YouTube import)",
          ytAvailable,
          toolsOffline
            ? "Offline mode detected — network imports paused"
            : toolStatus?.yt_dlp.version
                ? `Detected ${toolStatus.yt_dlp.version}`
                : "Used to download audio from YouTube before transcription",
          YT_DLP_HINTS,
          youtubeDisableReason && (
            <p className="text-xs text-yellow-300 mt-2">{youtubeDisableReason}</p>
          ),
        )}
        {renderToolCard(
          "ffmpeg (file decoding)",
          ffmpegReady,
          toolStatus?.ffmpeg.version
            ? `Detected ${toolStatus.ffmpeg.version}`
            : "Converts MP4/M4A recordings into 16 kHz mono audio",
          FFMPEG_HINTS,
          fileDisableReason && (
            <p className="text-xs text-yellow-300 mt-2">{fileDisableReason}</p>
          ),
        )}
      </div>

      <div className="grid grid-cols-1 gap-3 md:grid-cols-3 md:items-center">
        <label className="text-sm text-muted-foreground">Meeting name</label>
        <input
          className="col-span-2 w-full rounded border border-border bg-transparent p-2 text-sm"
          placeholder="e.g., Customer Interview #3"
          value={meetingName}
          onChange={(e) => setMeetingName(e.target.value)}
        />
      </div>
      <div className="grid grid-cols-1 gap-2 md:grid-cols-3 md:items-center">
        <label className="text-sm text-muted-foreground">Audio file path</label>
        <div className="col-span-2 flex items-center gap-2">
          <input
            className="w-full rounded border border-border bg-transparent p-2 text-sm"
            placeholder="/Users/you/Downloads/zoom_meeting.mp4"
            value={filePath}
            onChange={(e) => setFilePath(e.target.value)}
          />
          <button
            disabled={isImporting || !ffmpegReady}
            className="rounded border border-border px-3 py-1 text-sm hover:bg-background-ui disabled:opacity-50"
            title={fileDisableReason ?? undefined}
            onClick={importFromPath}
          >
            Import
          </button>
          <button
            disabled={isImporting}
            className="rounded border border-border px-3 py-1 text-sm hover:bg-background-ui disabled:opacity-50"
            onClick={browseAndSetPath}
          >
            Browse…
          </button>
        </div>
      </div>
      <div className="grid grid-cols-1 gap-2 md:grid-cols-3 md:items-center">
        <label className="text-sm text-muted-foreground">YouTube URL</label>
        <div className="col-span-2 flex items-center gap-2">
          <input
            className="w-full rounded border border-border bg-transparent p-2 text-sm"
            placeholder="https://www.youtube.com/watch?v=…"
            value={ytUrl}
            onChange={(e) => setYtUrl(e.target.value)}
          />
          <button
            disabled={isImporting || !ytAvailable}
            className="rounded border border-border px-3 py-1 text-sm hover:bg-background-ui disabled:opacity-50"
            title={youtubeDisableReason ?? undefined}
            onClick={importYoutube}
          >
            Import
          </button>
        </div>
      </div>

      {isImporting && (
        <div className="flex items-center gap-3 text-xs text-muted-foreground">
          <ProgressBar
            progress={[
              {
                id: "import",
                percentage: progressPercent ?? 0,
                label: progressStage ?? "importing",
              },
            ]}
            size="large"
            showLabel
          />
          <span className="capitalize">{progressStage?.replace(/-/g, " ") || "importing"}</span>
        </div>
      )}

      {!isImporting && lastTranscriptDir && (
        <div className="flex items-center justify-between rounded border border-border p-2 text-xs">
          <div className="truncate pr-2">
            Saved to: <span className="font-mono">{lastTranscriptDir}</span>
          </div>
          <div className="flex gap-2">
            <button
              className="rounded border border-border px-2 py-0.5 hover:bg-background-ui"
              onClick={async () => {
                try {
                  await invoke("open_path_in_file_manager", { path: lastTranscriptDir });
                } catch (e) {
                  toast.error("Open failed", { description: String(e) });
                }
              }}
            >
              Open Folder
            </button>
            <button
              className="rounded border border-border px-2 py-0.5 hover:bg-background-ui"
              onClick={async () => {
                try {
                  await invoke("open_path_in_file_manager", { path: `${lastTranscriptDir}/transcript.md` });
                } catch (e) {
                  toast.error("Open failed", { description: String(e) });
                }
              }}
            >
              Open Transcript
            </button>
            <button
              className="rounded border border-border px-2 py-0.5 hover:bg-background-ui"
              onClick={async () => {
                try {
                  await invoke("open_path_in_file_manager", { path: `${lastTranscriptDir}/summary.md` });
                } catch (e) {
                  toast.error("Open failed", { description: String(e) });
                }
              }}
            >
              Open Summary
            </button>
          </div>
        </div>
      )}
    </div>
  );
};
