import React, { useEffect, useState } from "react";
import { useSettings } from "../../hooks/useSettings";
import { Input } from "../ui/Input";
import { SettingContainer } from "../ui/SettingContainer";

interface UpdateIntervalProps {
  descriptionMode?: "tooltip" | "inline";
  grouped?: boolean;
}

export const UpdateInterval: React.FC<UpdateIntervalProps> = ({
  descriptionMode = "inline",
  grouped = false,
}) => {
  const { getSetting, updateSetting, isUpdating } = useSettings();
  const current = getSetting("meeting_update_interval_seconds") ?? 20;
  const [value, setValue] = useState<string>(String(current));

  useEffect(() => {
    setValue(String(current));
  }, [current]);

  const commit = async () => {
    const parsed = parseInt(value, 10);
    if (isNaN(parsed)) {
      setValue(String(current));
      return;
    }
    const clamped = Math.min(300, Math.max(5, parsed));
    setValue(String(clamped));
    await updateSetting("meeting_update_interval_seconds", clamped);
  };

  return (
    <SettingContainer
      title="Meeting Update Interval"
      description="Seconds between context updates (5–300)"
      descriptionMode={descriptionMode}
      grouped={grouped}
      layout="horizontal"
    >
      <div className="flex items-center space-x-2">
        <Input
          type="number"
          min="5"
          max="300"
          step="1"
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onBlur={commit}
          onKeyDown={(e) => {
            if (e.key === "Enter") commit();
          }}
          disabled={isUpdating("meeting_update_interval_seconds")}
          className="w-20"
        />
        <span className="text-sm text-text">seconds</span>
      </div>
    </SettingContainer>
  );
};

