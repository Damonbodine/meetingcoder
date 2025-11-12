import React, { useEffect, useState } from "react";
import { useSettings } from "../../hooks/useSettings";
import { Input } from "../ui/Input";
import { SettingContainer } from "../ui/SettingContainer";

interface Props { descriptionMode?: "tooltip" | "inline"; grouped?: boolean }

export const SystemAudioBufferSeconds: React.FC<Props> = ({ descriptionMode = "inline", grouped = false }) => {
  const { getSetting, updateSetting, isUpdating } = useSettings();
  // Default to 90s if not yet set
  const current = getSetting("system_audio_buffer_seconds") ?? 90;
  const [value, setValue] = useState<string>(String(current));

  useEffect(() => { setValue(String(current)); }, [current]);

  const commit = async () => {
    const parsed = parseInt(value, 10);
    if (isNaN(parsed)) { setValue(String(current)); return; }
    const clamped = Math.min(600, Math.max(30, parsed));
    setValue(String(clamped));
    await updateSetting("system_audio_buffer_seconds", clamped);
  };

  return (
    <SettingContainer
      title="System Audio Buffer Size"
      description="Capacity for system audio ring buffer in seconds (30–600). Larger sizes reduce risk of drops at the cost of RAM."
      descriptionMode={descriptionMode}
      grouped={grouped}
      layout="horizontal"
    >
      <div className="flex items-center space-x-2">
        <Input
          type="number"
          min="30"
          max="600"
          step="10"
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onBlur={commit}
          onKeyDown={(e) => { if (e.key === "Enter") commit(); }}
          disabled={isUpdating("system_audio_buffer_seconds")}
          className="w-24"
        />
        <span className="text-sm text-text">seconds</span>
      </div>
    </SettingContainer>
  );
};
