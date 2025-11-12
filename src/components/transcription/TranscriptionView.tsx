import { useState } from "react";
import { ImportAudio } from "./ImportAudio";
import { TranscriptEditor } from "./TranscriptEditor";
import { TranscriptSegment } from "../../lib/types";

export const TranscriptionView = () => {
  const [transcriptSegments, setTranscriptSegments] = useState<TranscriptSegment[]>([]);

  const handleSegmentUpdate = (index: number, newText: string) => {
    setTranscriptSegments((prev) => {
      const updated = [...prev];
      updated[index] = { ...updated[index], text: newText };
      return updated;
    });
    // TODO: Add backend call to persist changes
    // await invoke("update_transcript_segment", { meetingId, segmentIndex: index, newText });
  };

  return (
    <div className="w-full max-w-4xl space-y-6">
      <div className="text-sm text-muted-foreground mb-4">
        Import audio files or YouTube videos to generate transcriptions. Edit and refine your transcripts as needed.
      </div>
      <ImportAudio />

      {/* Transcript Editor Section */}
      <div className="mt-8">
        <TranscriptEditor
          segments={transcriptSegments}
          onSegmentUpdate={handleSegmentUpdate}
        />
      </div>
    </div>
  );
};
