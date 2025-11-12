import React, { useEffect, useState } from "react";
import { useSettings } from "../../hooks/useSettings";
import { Input } from "../ui/Input";
import { SettingContainer } from "../ui/SettingContainer";

interface ChunkDurationProps {
  descriptionMode?: "tooltip" | "inline";
  grouped?: boolean;
}

export const ChunkDuration: React.FC<ChunkDurationProps> = ({
  descriptionMode = "inline",
  grouped = false,
}) => {
  const { getSetting, updateSetting, isUpdating } = useSettings();
  const current = getSetting("transcription_chunk_seconds") ?? 10;
  const [value, setValue] = useState<string>(String(current));

  useEffect(() => {
    // Keep local input in sync with settings when not actively editing
    setValue(String(current));
  }, [current]);

  const commit = async () => {
    const parsed = parseInt(value, 10);
    if (isNaN(parsed)) {
      // Revert to current setting if invalid/empty
      setValue(String(current));
      return;
    }
    const clamped = Math.min(60, Math.max(2, parsed));
    setValue(String(clamped));
    await updateSetting("transcription_chunk_seconds", clamped);
  };

  return (
    <SettingContainer
      title="Transcription Chunk Duration"
      description="Seconds per chunk for meeting transcription (2–60)"
      descriptionMode={descriptionMode}
      grouped={grouped}
      layout="horizontal"
    >
      <div className="flex items-center space-x-2">
        <Input
          type="number"
          min="2"
          max="60"
          step="1"
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onBlur={commit}
          onKeyDown={(e) => {
            if (e.key === "Enter") commit();
          }}
          disabled={isUpdating("transcription_chunk_seconds")}
          className="w-20"
        />
        <span className="text-sm text-text">seconds</span>
      </div>
    </SettingContainer>
  );
};
