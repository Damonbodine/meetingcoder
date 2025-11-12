import React, { useState } from "react";
import { Edit2, Save, X } from "lucide-react";
import { Modal } from "../ui/Modal";
import { TranscriptSegment } from "../../lib/types";

interface TranscriptEditorProps {
  segments: TranscriptSegment[];
  onSegmentUpdate?: (index: number, newText: string) => void;
  readOnly?: boolean;
}

export const TranscriptEditor: React.FC<TranscriptEditorProps> = ({
  segments,
  onSegmentUpdate,
  readOnly = false,
}) => {
  const [editingIndex, setEditingIndex] = useState<number | null>(null);
  const [editText, setEditText] = useState("");

  const handleEditClick = (index: number, currentText: string) => {
    setEditingIndex(index);
    setEditText(currentText);
  };

  const handleSave = () => {
    if (editingIndex !== null && onSegmentUpdate) {
      onSegmentUpdate(editingIndex, editText);
    }
    setEditingIndex(null);
    setEditText("");
  };

  const handleCancel = () => {
    setEditingIndex(null);
    setEditText("");
  };

  const formatTimestamp = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, "0")}`;
  };

  return (
    <div className="space-y-4">
      <div className="text-sm font-medium mb-2">Transcript Segments</div>
      <div className="space-y-2">
        {segments.length === 0 ? (
          <div className="text-sm text-muted-foreground text-center py-8">
            No transcript segments available. Import an audio file or YouTube video to generate a transcription.
          </div>
        ) : (
          segments.map((segment, index) => (
            <div
              key={index}
              className="rounded-md border border-border p-3 hover:border-logo-primary/30 transition-colors"
            >
              <div className="flex items-start justify-between gap-3">
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 mb-1">
                    <span className="text-xs font-medium text-logo-primary">
                      {segment.speaker || "Speaker"}
                    </span>
                    <span className="text-xs text-muted-foreground">
                      {formatTimestamp(segment.start_time)} -{" "}
                      {formatTimestamp(segment.end_time)}
                    </span>
                    {segment.confidence && (
                      <span className="text-xs text-muted-foreground">
                        ({Math.round(segment.confidence * 100)}% confidence)
                      </span>
                    )}
                  </div>
                  <p className="text-sm leading-relaxed whitespace-pre-wrap break-words">
                    {segment.text}
                  </p>
                </div>
                {!readOnly && (
                  <button
                    onClick={() => handleEditClick(index, segment.text)}
                    className="flex-shrink-0 p-1.5 rounded-md hover:bg-mid-gray/20 transition-colors"
                    aria-label="Edit segment"
                  >
                    <Edit2 size={16} />
                  </button>
                )}
              </div>
            </div>
          ))
        )}
      </div>

      {/* Edit Modal */}
      <Modal
        isOpen={editingIndex !== null}
        onClose={handleCancel}
        title="Edit Transcript Segment"
      >
        <div className="space-y-4">
          <div>
            <label className="text-sm font-medium mb-2 block">
              Segment Text
            </label>
            <textarea
              className="w-full rounded border border-border bg-transparent p-3 text-sm min-h-[120px] resize-y"
              value={editText}
              onChange={(e) => setEditText(e.target.value)}
              placeholder="Enter transcript text..."
              autoFocus
            />
          </div>
          <div className="flex justify-end gap-2">
            <button
              onClick={handleCancel}
              className="flex items-center gap-2 rounded border border-border px-4 py-2 text-sm hover:bg-mid-gray/20 transition-colors"
            >
              <X size={16} />
              Cancel
            </button>
            <button
              onClick={handleSave}
              className="flex items-center gap-2 rounded bg-logo-primary px-4 py-2 text-sm text-white hover:bg-logo-primary/80 transition-colors"
            >
              <Save size={16} />
              Save Changes
            </button>
          </div>
        </div>
      </Modal>
    </div>
  );
};
