import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { listen } from "@tauri-apps/api/event";
import ProgressBar from "../shared/ProgressBar";

export const ImportAudio: React.FC = () => {
  const [meetingName, setMeetingName] = useState("");
  const [isImporting, setIsImporting] = useState(false);
  const [ytUrl, setYtUrl] = useState("");
  const [filePath, setFilePath] = useState("");
  const [progressStage, setProgressStage] = useState<string | null>(null);
  const [progressPercent, setProgressPercent] = useState<number | null>(null);
  const [lastTranscriptDir, setLastTranscriptDir] = useState<string | null>(null);

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
              typeof percent === "number" ? Math.max(0, Math.min(100, percent)) : null
            );
          }
        );
      } catch (e) {
        console.error("Failed to listen for import-progress:", e);
      }
    })();
    return () => {
      if (unlisten) {
        try { unlisten(); } catch {}
      }
    };
  }, []);

  const importFromPath = async () => {
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
      const msg = String(e);
      toast.error("Import failed", { description: msg });
    } finally {
      setIsImporting(false);
    }
  };

  const importYoutube = async () => {
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
      if (msg.toLowerCase().includes("yt-dlp not found")) {
        const hint = navigator.userAgent.includes("Mac")
          ? "Install via: brew install yt-dlp"
          : "Install yt-dlp from https://github.com/yt-dlp/yt-dlp#installation";
        toast.error("yt-dlp not found", { description: hint });
      } else if (msg.toLowerCase().includes("network")) {
        toast.error("Network required for YouTube", {
          description: "Check your connection and try again.",
        });
      } else {
        toast.error("YouTube import failed", { description: msg });
      }
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

  return (
    <div className="rounded-md border border-border p-4 space-y-3">
      <div className="text-sm font-medium">Import Audio into MeetingCoder</div>
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
            disabled={isImporting}
            className="rounded border border-border px-3 py-1 text-sm hover:bg-background-ui disabled:opacity-50"
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
            disabled={isImporting}
            className="rounded border border-border px-3 py-1 text-sm hover:bg-background-ui disabled:opacity-50"
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
                try { await invoke("open_path_in_file_manager", { path: lastTranscriptDir }); }
                catch (e) { toast.error("Open failed", { description: String(e) }); }
              }}
            >
              Open Folder
            </button>
            <button
              className="rounded border border-border px-2 py-0.5 hover:bg-background-ui"
              onClick={async () => {
                try { await invoke("open_path_in_file_manager", { path: `${lastTranscriptDir}/transcript.md` }); }
                catch (e) { toast.error("Open failed", { description: String(e) }); }
              }}
            >
              Open Transcript
            </button>
            <button
              className="rounded border border-border px-2 py-0.5 hover:bg-background-ui"
              onClick={async () => {
                try { await invoke("open_path_in_file_manager", { path: `${lastTranscriptDir}/summary.md` }); }
                catch (e) { toast.error("Open failed", { description: String(e) }); }
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
