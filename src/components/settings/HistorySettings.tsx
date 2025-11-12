import React, { useState, useEffect, useCallback } from "react";
import { SettingsGroup } from "../ui/SettingsGroup";
import { AudioPlayer } from "../ui/AudioPlayer";
import { Copy, Star, Check, Trash2, FolderOpen, FileText } from "lucide-react";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { MeetingHistoryEntry } from "../../lib/types";
import { toast } from "sonner";

interface HistoryEntry {
  id: number;
  file_name: string;
  timestamp: number;
  saved: boolean;
  title: string;
  transcription_text: string;
}

export const HistorySettings: React.FC = () => {
  const [historyEntries, setHistoryEntries] = useState<HistoryEntry[]>([]);
  const [meetingHistory, setMeetingHistory] = useState<MeetingHistoryEntry[]>([]);
  const [loading, setLoading] = useState(true);

  const loadHistoryEntries = useCallback(async () => {
    try {
      const entries = await invoke<HistoryEntry[]>("get_history_entries");
      setHistoryEntries(entries);
    } catch (error) {
      console.error("Failed to load history entries:", error);
    }
  }, []);

  const loadMeetingHistory = useCallback(async () => {
    try {
      const meetings = await invoke<MeetingHistoryEntry[]>("list_saved_meetings");
      setMeetingHistory(meetings);
    } catch (error) {
      console.error("Failed to load meeting history:", error);
    }
  }, []);

  const loadAllHistory = useCallback(async () => {
    setLoading(true);
    await Promise.all([loadHistoryEntries(), loadMeetingHistory()]);
    setLoading(false);
  }, [loadHistoryEntries, loadMeetingHistory]);

  useEffect(() => {
    loadAllHistory();

    // Listen for history update events
    const setupListener = async () => {
      const unlisten = await listen("history-updated", () => {
        console.log("History updated, reloading entries...");
        loadAllHistory();
      });

      // Return cleanup function
      return unlisten;
    };

    let unlistenPromise = setupListener();

    return () => {
      unlistenPromise.then((unlisten) => {
        if (unlisten) {
          unlisten();
        }
      });
    };
  }, [loadAllHistory]);

  const toggleSaved = async (id: number) => {
    try {
      await invoke("toggle_history_entry_saved", { id });
      // No need to reload here - the event listener will handle it
    } catch (error) {
      console.error("Failed to toggle saved status:", error);
    }
  };

  const copyToClipboard = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
    } catch (error) {
      console.error("Failed to copy to clipboard:", error);
    }
  };

  const getAudioUrl = async (fileName: string) => {
    try {
      const filePath = await invoke<string>("get_audio_file_path", {
        fileName,
      });

      return convertFileSrc(`${filePath}`, "asset");
    } catch (error) {
      console.error("Failed to get audio file path:", error);
      return null;
    }
  };

  const deleteAudioEntry = async (id: number) => {
    try {
      await invoke("delete_history_entry", { id });
    } catch (error) {
      console.error("Failed to delete audio entry:", error);
      throw error;
    }
  };

  if (loading) {
    return (
      <div className="max-w-3xl w-full mx-auto space-y-6">
        <SettingsGroup title="History">
          <div className="px-4 py-3 text-center text-text/60">
            Loading history...
          </div>
        </SettingsGroup>
      </div>
    );
  }

  const hasAnyHistory = meetingHistory.length > 0 || historyEntries.length > 0;

  if (!hasAnyHistory) {
    return (
      <div className="max-w-3xl w-full mx-auto space-y-6">
        <SettingsGroup title="History">
          <div className="px-4 py-3 text-center text-text/60">
            No transcriptions yet. Start a meeting or recording to build your history!
          </div>
        </SettingsGroup>
      </div>
    );
  }

  return (
    <div className="max-w-3xl w-full mx-auto space-y-6">
      <SettingsGroup title="History">
        <div className="space-y-4">
          {/* Meeting History Section */}
          {meetingHistory.length > 0 && (
            <div className="space-y-3">
              <h3 className="text-sm font-semibold px-4 text-text/80">Meetings & Transcriptions</h3>
              {meetingHistory.map((meeting) => (
                <MeetingHistoryEntryComponent
                  key={meeting.dir_name}
                  meeting={meeting}
                  onDelete={() => {
                    invoke("delete_saved_meeting", { dirName: meeting.dir_name })
                      .then(() => {
                        toast.success("Meeting deleted");
                        loadMeetingHistory();
                      })
                      .catch((e) => toast.error(`Failed to delete: ${e}`));
                  }}
                />
              ))}
            </div>
          )}

          {/* Old Quick Recordings Section */}
          {historyEntries.length > 0 && (
            <div className="space-y-3">
              {meetingHistory.length > 0 && <div className="border-t border-mid-gray/20 my-4"></div>}
              <h3 className="text-sm font-semibold px-4 text-text/80">Quick Recordings</h3>
              {historyEntries.map((entry) => (
                <HistoryEntryComponent
                  key={entry.id}
                  entry={entry}
                  onToggleSaved={() => toggleSaved(entry.id)}
                  onCopyText={() => copyToClipboard(entry.transcription_text)}
                  getAudioUrl={getAudioUrl}
                  deleteAudio={deleteAudioEntry}
                />
              ))}
            </div>
          )}
        </div>
      </SettingsGroup>
    </div>
  );
};

interface MeetingHistoryEntryProps {
  meeting: MeetingHistoryEntry;
  onDelete: () => void;
}

const MeetingHistoryEntryComponent: React.FC<MeetingHistoryEntryProps> = ({
  meeting,
  onDelete,
}) => {
  const formatDate = (isoString: string) => {
    const date = new Date(isoString);
    return date.toLocaleDateString(undefined, {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  };

  const formatDuration = (seconds: number) => {
    const minutes = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${minutes}m ${secs}s`;
  };

  const handleOpenFolder = async () => {
    try {
      await invoke("open_meeting_folder", { dirPath: meeting.dir_path });
    } catch (error) {
      toast.error(`Failed to open folder: ${error}`);
    }
  };

  return (
    <div className="px-4 py-3 pb-4 flex flex-col gap-3 border-b border-mid-gray/10">
      <div className="flex justify-between items-start">
        <div className="flex-1">
          <p className="text-sm font-medium">{meeting.metadata.name}</p>
          <p className="text-xs text-text/60 mt-1">
            {formatDate(meeting.metadata.start_time)} • {formatDuration(meeting.metadata.duration_seconds)}
          </p>
          {meeting.metadata.participants.length > 0 && (
            <p className="text-xs text-text/50 mt-1">
              Participants: {meeting.metadata.participants.join(", ")}
            </p>
          )}
        </div>
        <div className="flex items-center gap-1">
          <button
            onClick={handleOpenFolder}
            className="p-2 rounded text-text/50 hover:text-logo-primary hover:bg-background-ui transition-colors cursor-pointer"
            title="Open folder"
          >
            <FolderOpen width={16} height={16} />
          </button>
          <button
            onClick={onDelete}
            className="p-2 rounded text-text/50 hover:text-red-500 hover:bg-background-ui transition-colors cursor-pointer"
            title="Delete meeting"
          >
            <Trash2 width={16} height={16} />
          </button>
        </div>
      </div>
      <div className="flex gap-2 text-xs">
        <span className="px-2 py-1 bg-background-ui rounded text-text/70 font-mono">
          {meeting.dir_name}
        </span>
      </div>
    </div>
  );
};

interface HistoryEntryProps {
  entry: HistoryEntry;
  onToggleSaved: () => void;
  onCopyText: () => void;
  getAudioUrl: (fileName: string) => Promise<string | null>;
  deleteAudio: (id: number) => Promise<void>;
}

const HistoryEntryComponent: React.FC<HistoryEntryProps> = ({
  entry,
  onToggleSaved,
  onCopyText,
  getAudioUrl,
  deleteAudio,
}) => {
  const [audioUrl, setAudioUrl] = useState<string | null>(null);
  const [showCopied, setShowCopied] = useState(false);

  useEffect(() => {
    const loadAudio = async () => {
      const url = await getAudioUrl(entry.file_name);
      setAudioUrl(url);
    };
    loadAudio();
  }, [entry.file_name, getAudioUrl]);

  const handleCopyText = () => {
    onCopyText();
    setShowCopied(true);
    setTimeout(() => setShowCopied(false), 2000);
  };

  const handleDeleteEntry = async () => {
    try {
      await deleteAudio(entry.id);
    } catch (error) {
      console.error("Failed to delete entry:", error);
      alert("Failed to delete entry. Please try again.");
    }
  };

  return (
    <div className="px-4 py-2 pb-5 flex flex-col gap-3">
      <div className="flex justify-between items-center">
        <p className="text-sm font-medium">{entry.title}</p>
        <div className="flex items-center gap-1">
          <button
            onClick={handleCopyText}
            className="text-text/50 hover:text-logo-primary  hover:border-logo-primary transition-colors cursor-pointer"
            title="Copy transcription to clipboard"
          >
            {showCopied ? (
              <Check width={16} height={16} />
            ) : (
              <Copy width={16} height={16} />
            )}
          </button>
          <button
            onClick={onToggleSaved}
            className={`p-2 rounded  transition-colors cursor-pointer ${
              entry.saved
                ? "text-logo-primary hover:text-logo-primary/80"
                : "text-text/50 hover:text-logo-primary"
            }`}
            title={entry.saved ? "Remove from saved" : "Save transcription"}
          >
            <Star
              width={16}
              height={16}
              fill={entry.saved ? "currentColor" : "none"}
            />
          </button>
          <button
            onClick={handleDeleteEntry}
            className="text-text/50 hover:text-logo-primary transition-colors cursor-pointer"
            title="Delete entry"
          >
            <Trash2 width={16} height={16} />
          </button>
        </div>
      </div>
      <p className="italic text-text/90 text-sm pb-2">
        {entry.transcription_text}
      </p>
      {audioUrl && <AudioPlayer src={audioUrl} className="w-full" />}
    </div>
  );
};
