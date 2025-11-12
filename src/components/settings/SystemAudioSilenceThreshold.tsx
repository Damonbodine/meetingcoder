import React, { useEffect, useState } from "react";
import { useSettings } from "../../hooks/useSettings";
import { Input } from "../ui/Input";
import { SettingContainer } from "../ui/SettingContainer";

interface Props { descriptionMode?: "tooltip" | "inline"; grouped?: boolean }

export const SystemAudioSilenceThreshold: React.FC<Props> = ({ descriptionMode = "inline", grouped = false }) => {
  const { getSetting, updateSetting, isUpdating } = useSettings();
  const current = getSetting("system_audio_silence_threshold") ?? -50;
  const [value, setValue] = useState<string>(String(current));

  useEffect(() => { setValue(String(current)); }, [current]);

  const commit = async () => {
    const parsed = parseFloat(value);
    if (isNaN(parsed)) { setValue(String(current)); return; }
    const clamped = Math.min(0, Math.max(-80, parsed));
    setValue(String(clamped));
    await updateSetting("system_audio_silence_threshold", clamped);
  };

  return (
    <SettingContainer
      title="Silence Threshold (System Audio)"
      description="dBFS threshold for chunk silence detection (-80 to 0)"
      descriptionMode={descriptionMode}
      grouped={grouped}
      layout="horizontal"
    >
      <div className="flex items-center space-x-2">
        <Input
          type="number"
          min="-80"
          max="0"
          step="1"
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onBlur={commit}
          onKeyDown={(e) => { if (e.key === "Enter") commit(); }}
          disabled={isUpdating("system_audio_silence_threshold")}
          className="w-24"
        />
        <span className="text-sm text-text">dBFS</span>
      </div>
    </SettingContainer>
  );
};

