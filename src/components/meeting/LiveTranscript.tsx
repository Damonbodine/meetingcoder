import { useEffect, useRef } from "react";
import { TranscriptSegment } from "../../lib/types";
import { SettingsGroup } from "../ui/SettingsGroup";
import { Clock } from "lucide-react";

interface LiveTranscriptProps {
  meetingId: string;
  meetingName: string;
  segments: TranscriptSegment[];
}

export const LiveTranscript = ({ meetingId, meetingName, segments }: LiveTranscriptProps) => {
  const transcriptEndRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom when new segments arrive
  useEffect(() => {
    transcriptEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [segments]);

  const formatTime = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, "0")}`;
  };

  const getSpeakerColor = (speaker: string) => {
    // Simple hash function to get consistent colors for speakers
    const hash = speaker.split("").reduce((acc, char) => acc + char.charCodeAt(0), 0);
    const colors = [
      "text-blue-600 dark:text-blue-400",
      "text-green-600 dark:text-green-400",
      "text-purple-600 dark:text-purple-400",
      "text-orange-600 dark:text-orange-400",
      "text-pink-600 dark:text-pink-400",
      "text-cyan-600 dark:text-cyan-400",
    ];
    return colors[hash % colors.length];
  };

  return (
    <SettingsGroup
      title="Live Transcript"
      description={`Meeting: ${meetingName} • Segments: ${segments.length}`}
    >
      <div className="space-y-4">
        <div className="max-h-[600px] overflow-y-auto border border-gray-300 dark:border-gray-700 rounded-lg p-4 bg-white dark:bg-gray-900">
          {segments.length === 0 ? (
            <div className="text-center text-gray-500 dark:text-gray-400 py-12">
              <Clock className="w-12 h-12 mx-auto mb-3 opacity-50" />
              <p className="font-medium">Waiting for first transcription...</p>
              <p className="text-sm mt-2">First segment will appear in ~30 seconds</p>
            </div>
          ) : (
            <div className="space-y-4">
              {segments.map((segment, index) => (
                <div
                  key={index}
                  className="border-l-4 border-gray-300 dark:border-gray-700 pl-4 py-2 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors"
                >
                  <div className="flex items-baseline justify-between mb-1">
                    <span className={`font-semibold ${getSpeakerColor(segment.speaker)}`}>
                      {segment.speaker}
                    </span>
                    <span className="text-xs text-gray-500 dark:text-gray-400 flex items-center gap-1">
                      <Clock className="w-3 h-3" />
                      {formatTime(segment.start_time)} - {formatTime(segment.end_time)}
                    </span>
                  </div>
                  <p className="text-gray-800 dark:text-gray-200 leading-relaxed">
                    {segment.text}
                  </p>
                  <div className="flex items-center justify-between mt-1">
                    <span className="text-xs text-gray-400">
                      Confidence: {(segment.confidence * 100).toFixed(0)}%
                    </span>
                  </div>
                </div>
              ))}
              <div ref={transcriptEndRef} />
            </div>
          )}
        </div>

        {segments.length > 0 && (
          <div className="text-sm text-gray-600 dark:text-gray-400 flex items-center justify-between">
            <span>Total words: {segments.reduce((sum, s) => sum + s.text.split(" ").length, 0)}</span>
            <span>Meeting ID: {meetingId.substring(0, 8)}...</span>
          </div>
        )}
      </div>
    </SettingsGroup>
  );
};
