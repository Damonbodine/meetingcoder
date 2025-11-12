import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { MeetingControls } from "./MeetingControls";
import { LiveTranscript } from "./LiveTranscript";
import { MeetingUpdates } from "./MeetingUpdates";
import { MeetingChecklist } from "./MeetingChecklist";
import { AudioSetupSection } from "./AudioSetupSection";
import { TranscriptSegment, MeetingSummary } from "../../lib/types";
import { toast } from "sonner";

export const MeetingView = () => {
  const [activeMeetingId, setActiveMeetingId] = useState<string | null>(null);
  const [meetingName, setMeetingName] = useState("");
  const [transcriptSegments, setTranscriptSegments] = useState<TranscriptSegment[]>([]);
  const [isStarting, setIsStarting] = useState(false);
  const [isEnding, setIsEnding] = useState(false);

  // Reattach to an active meeting if user navigates away and back
  useEffect(() => {
    let cancelled = false;
    const reattach = async () => {
      try {
        const active = await invoke<string[]>("get_active_meetings");
        if (cancelled) return;
        if (active && active.length > 0) {
          const id = active[0];
          const info = await invoke<any>("get_meeting_info", { meetingId: id });
          if (cancelled) return;
          setActiveMeetingId(id);
          setMeetingName(info?.name || "");
          // Optionally fetch current transcript so UI shows existing lines
          try {
            const segments = await invoke<TranscriptSegment[]>("get_live_transcript", { meetingId: id });
            if (!cancelled) setTranscriptSegments(segments || []);
          } catch (_) { /* ignore */ }
        }
      } catch (_) {
        // ignore
      }
    };
    reattach();
    return () => { cancelled = true; };
  }, []);

  // Listen for new transcript segments
  useEffect(() => {
    if (!activeMeetingId) return;

    let unlisten: (() => void) | undefined;
    let unlistenWarning: (() => void) | undefined;
    let unlistenRestarting: (() => void) | undefined;
    let unlistenRestartSuccess: (() => void) | undefined;
    let unlistenRestartFailed: (() => void) | undefined;
    let isMounted = true;

    const setupListeners = async () => {
      try {
        // Listen for transcript segments
        const unlistenFn = await listen<{ meeting_id: string; segment: TranscriptSegment }>(
          "transcript-segment-added",
          (event) => {
            console.log("Received transcript segment:", event.payload);
            if (event.payload.meeting_id === activeMeetingId && isMounted) {
              setTranscriptSegments((prev) => [...prev, event.payload.segment]);
              toast.success("New segment transcribed", {
                description: event.payload.segment.text.substring(0, 50) + "...",
              });
            }
          }
        );

        // Listen for audio stream warnings
        const unlistenWarningFn = await listen<{
          meeting_id: string;
          message: string;
          consecutive_empty_chunks: number;
        }>(
          "audio-stream-warning",
          (event) => {
            console.error("Audio stream warning:", event.payload);
            if (event.payload.meeting_id === activeMeetingId && isMounted) {
              toast.error("Audio stream issue detected", {
                description: event.payload.message,
                duration: 10000,
              });
            }
          }
        );

        // Listen for audio stream restart attempts
        const unlistenRestartingFn = await listen<{
          meeting_id: string;
          attempt: number;
          max_attempts: number;
        }>(
          "audio-stream-restarting",
          (event) => {
            console.log("Audio stream restarting:", event.payload);
            if (event.payload.meeting_id === activeMeetingId && isMounted) {
              toast.info("Restarting audio stream", {
                description: `Attempt ${event.payload.attempt}/${event.payload.max_attempts}. Please wait...`,
                duration: 5000,
              });
            }
          }
        );

        // Listen for successful audio stream restart
        const unlistenRestartSuccessFn = await listen<{
          meeting_id: string;
        }>(
          "audio-stream-restart-success",
          (event) => {
            console.log("Audio stream restart successful:", event.payload);
            if (event.payload.meeting_id === activeMeetingId && isMounted) {
              toast.success("Audio stream recovered", {
                description: "Recording will continue automatically.",
                duration: 5000,
              });
            }
          }
        );

        // Listen for failed audio stream restart
        const unlistenRestartFailedFn = await listen<{
          meeting_id: string;
          error: string;
          attempts_remaining: number;
        }>(
          "audio-stream-restart-failed",
          (event) => {
            console.error("Audio stream restart failed:", event.payload);
            if (event.payload.meeting_id === activeMeetingId && isMounted) {
              if (event.payload.attempts_remaining > 0) {
                toast.warning("Audio restart failed", {
                  description: `${event.payload.attempts_remaining} attempts remaining. Will retry automatically.`,
                  duration: 5000,
                });
              } else {
                toast.error("Audio restart failed", {
                  description: "No attempts remaining. Please restart the meeting manually.",
                  duration: 10000,
                });
              }
            }
          }
        );

        if (isMounted) {
          unlisten = unlistenFn;
          unlistenWarning = unlistenWarningFn;
          unlistenRestarting = unlistenRestartingFn;
          unlistenRestartSuccess = unlistenRestartSuccessFn;
          unlistenRestartFailed = unlistenRestartFailedFn;
        } else {
          // Component unmounted before listeners were set up
          unlistenFn();
          unlistenWarningFn();
          unlistenRestartingFn();
          unlistenRestartSuccessFn();
          unlistenRestartFailedFn();
        }
      } catch (error) {
        console.error("Failed to setup event listeners:", error);
      }
    };

    setupListeners();

    return () => {
      isMounted = false;
      const listeners = [
        { fn: unlisten, name: "segment" },
        { fn: unlistenWarning, name: "warning" },
        { fn: unlistenRestarting, name: "restarting" },
        { fn: unlistenRestartSuccess, name: "restart-success" },
        { fn: unlistenRestartFailed, name: "restart-failed" },
      ];

      for (const { fn, name } of listeners) {
        if (fn) {
          try {
            fn();
          } catch (error) {
            console.error(`Error cleaning up ${name} listener:`, error);
          }
        }
      }
    };
  }, [activeMeetingId]);

  const handleStartMeeting = async (name: string) => {
    try {
      setIsStarting(true);
      setMeetingName(name);

      // Ensure system audio is active
      const audioSource = await invoke<any>("get_current_audio_source");
      console.log("Current audio source:", audioSource);

      // Check if it's system audio (format can be: "system:device_name", { SystemAudio: "device" }, or "SystemAudio")
      const isSystemAudio =
        (typeof audioSource === "string" && (audioSource.includes("system:") || audioSource.includes("SystemAudio"))) ||
        (typeof audioSource === "object" && audioSource.SystemAudio);

      if (!isSystemAudio) {
        toast.error("Please select BlackHole as audio source", {
          description: "Use the 'Audio Setup' section above to configure system audio capture",
          duration: 5000,
        });
        setIsStarting(false);
        return;
      }

      const meetingId = await invoke<string>("start_meeting", { meetingName: name });

      setActiveMeetingId(meetingId);
      setTranscriptSegments([]);
      toast.success("Meeting started", {
        description: "Transcription will begin in 30 seconds",
      });
    } catch (error) {
      console.error("Failed to start meeting:", error);
      toast.error("Failed to start meeting", {
        description: String(error),
      });
    } finally {
      setIsStarting(false);
    }
  };

  const handleEndMeeting = async () => {
    if (!activeMeetingId) return;

    try {
      setIsEnding(true);
      const summary = await invoke<MeetingSummary>("end_meeting", {
        meetingId: activeMeetingId,
      });

      const minutes = Math.floor(summary.duration_seconds / 60);
      const seconds = summary.duration_seconds % 60;

      toast.success("Meeting ended", {
        description: `Duration: ${minutes}m ${seconds}s, Segments: ${summary.total_segments}`,
      });

      setActiveMeetingId(null);
      setTranscriptSegments([]);
      setMeetingName("");
    } catch (error) {
      console.error("Failed to end meeting:", error);
      toast.error("Failed to end meeting", {
        description: String(error),
      });
    } finally {
      setIsEnding(false);
    }
  };

  const handlePauseMeeting = async () => {
    if (!activeMeetingId) return;

    try {
      await invoke("pause_meeting", { meetingId: activeMeetingId });
      toast.success("Meeting paused");
    } catch (error) {
      console.error("Failed to pause meeting:", error);
      toast.error("Failed to pause meeting");
    }
  };

  const handleResumeMeeting = async () => {
    if (!activeMeetingId) return;

    try {
      await invoke("resume_meeting", { meetingId: activeMeetingId });
      toast.success("Meeting resumed");
    } catch (error) {
      console.error("Failed to resume meeting:", error);
      toast.error("Failed to resume meeting");
    }
  };

  return (
    <div className="w-full max-w-4xl space-y-6">
      <div className="text-sm text-muted-foreground mb-4">
        Start a live meeting to record and transcribe in real-time. For importing audio files or YouTube videos, use the Transcription section.
      </div>

      {/* Audio Setup Section - Prominent at top */}
      {!activeMeetingId && (
        <div className="mb-6">
          <h3 className="text-base font-semibold mb-3">Audio Setup</h3>
          <AudioSetupSection />
        </div>
      )}

      <MeetingControls
        isActive={!!activeMeetingId}
        meetingName={meetingName}
        isStarting={isStarting}
        isEnding={isEnding}
        onStart={handleStartMeeting}
        onEnd={handleEndMeeting}
        onPause={handlePauseMeeting}
        onResume={handleResumeMeeting}
      />

      {/* Checklist always visible to guide setup */}
      <div className="mt-4">
        <MeetingChecklist meetingId={activeMeetingId} meetingName={meetingName} />
      </div>

      {activeMeetingId && (
        <>
          <LiveTranscript
            meetingId={activeMeetingId}
            meetingName={meetingName}
            segments={transcriptSegments}
          />
          <div className="mt-6">
            <h3 className="mb-2 text-sm font-medium">Meeting Updates</h3>
            <MeetingUpdates meetingId={activeMeetingId} meetingName={meetingName} />
          </div>
        </>
      )}
    </div>
  );
};
