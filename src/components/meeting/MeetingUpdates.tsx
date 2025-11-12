import React, { useEffect, useMemo, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { readTextFile } from "@tauri-apps/plugin-fs";
import { useSettings } from "../../hooks/useSettings";

type SummaryRecord = {
  meeting_id: string;
  meeting_name: string;
  model: string;
  source: string;
  update_id: string; // format: uN
  timestamp: string;
  segment_range: [number, number];
  new_features: string[];
  technical_decisions: string[];
  questions: string[];
};

export const MeetingUpdates: React.FC<{ meetingId: string; meetingName: string }> = ({
  meetingId,
  meetingName,
}) => {
  const { getSetting } = useSettings();
  const [projectPath, setProjectPath] = useState<string | null>(null);
  const [updates, setUpdates] = useState<SummaryRecord[]>([]);
  const [opening, setOpening] = useState(false);
  const [lastAutomation, setLastAutomation] = useState<string | null>(null);

  const updatesPath = useMemo(() => {
    if (!projectPath) return null;
    return `${projectPath}/.meeting-updates.jsonl`;
  }, [projectPath]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let mounted = true;

    const init = async () => {
      try {
        const path = await invoke<string | null>("get_meeting_project_path", {
          meetingId,
        });
        if (!mounted) return;
        setProjectPath(path || null);

        if (path) {
          await loadUpdates(path);
        }

        const off = await listen<{ update_id: number; meeting_id: string }>(
          "meeting-update-appended",
          async (evt) => {
            if (evt.payload.meeting_id !== meetingId) return;
            if (path) {
              await loadUpdates(path);
            }
          },
        );
        unlisten = off;
      } catch (e) {
        console.error("MeetingUpdates init error:", e);
      }
    };

    const loadUpdates = async (ppath: string) => {
      try {
        const file = `${ppath}/.meeting-updates.jsonl`;
        const text = await readTextFile(file);
        const lines = text
          .split("\n")
          .map((l) => l.trim())
          .filter(Boolean);
        const parsed: SummaryRecord[] = lines
          .map((l) => {
            try {
              return JSON.parse(l);
            } catch {
              return null;
            }
          })
          .filter((v): v is SummaryRecord => !!v && v.meeting_id === meetingId);

        // Sort by numeric suffix of update_id asc and keep last 50
        const num = (id: string) => parseInt(id.replace(/^u/i, ""), 10) || 0;
        parsed.sort((a, b) => num(a.update_id) - num(b.update_id));
        const recent = parsed.slice(-50);
        setUpdates(recent);
        // Try reading automation state for status
        try {
          const autoState = await readTextFile(`${ppath}/.claude/.automation-state.json`);
          const parsedState = JSON.parse(autoState) as { last_trigger_time?: string };
          setLastAutomation(parsedState.last_trigger_time ?? null);
        } catch {
          setLastAutomation(null);
        }
      } catch (e) {
        console.error("Failed to read updates:", e);
      }
    };

    init();

    return () => {
      mounted = false;
      if (unlisten) {
        try {
          unlisten();
        } catch (e) {
          console.error("unlisten error:", e);
        }
      }
    };
  }, [meetingId]);

  if (!projectPath) {
    return (
      <div className="rounded-md border border-border p-3 text-sm text-muted-foreground">
        Resolving project path for “{meetingName}”…
      </div>
    );
  }

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between">
        <div className="text-sm text-muted-foreground">
          Writing updates to <code className="font-mono">{updatesPath}</code>
        </div>
        <div className="flex items-center gap-2">
          {lastAutomation && (
            <div className="text-xs text-muted-foreground mr-2" title="Last automation trigger time">
              Last auto: {new Date(lastAutomation).toLocaleTimeString()} ({getSetting("auto_accept_changes") ? "auto-accept on" : "auto-accept off"})
            </div>
          )}
          <button
            className="rounded border border-border px-2 py-1 text-xs hover:bg-background-ui disabled:opacity-50"
            disabled={opening}
            onClick={async () => {
              try {
                setOpening(true);
                await invoke("open_meeting_terminal", { meetingId });
              } catch (e) {
                console.error("open terminal error", e);
              } finally {
                setOpening(false);
              }
            }}
            title="Open Terminal in project folder"
          >
            Open Terminal
          </button>
          <button
            className="rounded border border-border px-2 py-1 text-xs hover:bg-background-ui"
            onClick={async () => {
              try {
                await invoke("trigger_meeting_command_now", { meetingId });
              } catch (e) {
                console.error("trigger /meeting error", e);
              }
            }}
            title="Send /meeting to Terminal"
          >
            Run /meeting
          </button>
        </div>
      </div>
      <div className="grid grid-cols-1 gap-3">
        {updates.map((u) => (
          <div key={u.update_id} className="rounded-md border border-border p-3">
            <div className="flex items-center justify-between text-xs text-muted-foreground">
              <span>Update #{u.update_id}</span>
              <span>{new Date(u.timestamp).toLocaleTimeString()}</span>
            </div>
            <div className="mt-1 text-xs text-muted-foreground">
              Segments {u.segment_range[0]}–{u.segment_range[1]} · {u.source} · {u.model || "model"}
            </div>
            {u.new_features.length > 0 && (
              <div className="mt-2">
                <div className="text-xs font-medium">Features</div>
                <ul className="mt-1 list-disc pl-5 text-sm">
                  {u.new_features.map((f, i) => (
                    <li key={i}>{f}</li>
                  ))}
                </ul>
              </div>
            )}
            {u.technical_decisions.length > 0 && (
              <div className="mt-2">
                <div className="text-xs font-medium">Decisions</div>
                <ul className="mt-1 list-disc pl-5 text-sm">
                  {u.technical_decisions.map((d, i) => (
                    <li key={i}>{d}</li>
                  ))}
                </ul>
              </div>
            )}
            {u.questions.length > 0 && (
              <div className="mt-2">
                <div className="text-xs font-medium">Questions</div>
                <ul className="mt-1 list-disc pl-5 text-sm">
                  {u.questions.map((q, i) => (
                    <li key={i}>{q}</li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        ))}
        {updates.length === 0 && (
          <div className="rounded-md border border-border p-3 text-sm text-muted-foreground">
            Waiting for first update…
          </div>
        )}
      </div>
    </div>
  );
};
