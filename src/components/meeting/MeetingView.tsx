import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { MeetingControls } from "./MeetingControls";
import { LiveTranscript } from "./LiveTranscript";
import { MeetingUpdates } from "./MeetingUpdates";
import { GitHubActions } from "./GitHubActions";
import { MeetingChecklist } from "./MeetingChecklist";
import { ImportAudio } from "./ImportAudio";
import { TranscriptSegment, MeetingSummary } from "../../lib/types";
import { toast } from "sonner";

export const MeetingView = () => {
  const [activeMeetingId, setActiveMeetingId] = useState<string | null>(null);
  const [meetingName, setMeetingName] = useState("");
  const [transcriptSegments, setTranscriptSegments] = useState<TranscriptSegment[]>([]);
  const [isStarting, setIsStarting] = useState(false);
  const [isEnding, setIsEnding] = useState(false);

  // Listen for new transcript segments
  useEffect(() => {
    if (!activeMeetingId) return;

    let unlisten: (() => void) | undefined;
    let isMounted = true;

    const setupListener = async () => {
      try {
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
        if (isMounted) {
          unlisten = unlistenFn;
        } else {
          // Component unmounted before listener was set up
          unlistenFn();
        }
      } catch (error) {
        console.error("Failed to setup event listener:", error);
      }
    };

    setupListener();

    return () => {
      isMounted = false;
      if (unlisten) {
        try {
          unlisten();
        } catch (error) {
          console.error("Error cleaning up listener:", error);
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
        toast.error("Please set audio source to BlackHole first", {
          description: "Go to Debug > System Audio Testing",
        });
        setIsStarting(false);
        return;
      }

      const meetingId = await invoke<string>("start_meeting", {
        meetingName: name,
      });

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
      <ImportAudio />
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
          <div className="mt-6">
            <GitHubActions meetingId={activeMeetingId} meetingName={meetingName} />
          </div>
        </>
      )}
    </div>
  );
};
