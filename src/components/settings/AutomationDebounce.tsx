import React, { useEffect, useState } from "react";
import { useSettings } from "../../hooks/useSettings";
import { Input } from "../ui/Input";
import { SettingContainer } from "../ui/SettingContainer";

export const AutomationDebounce: React.FC<{ descriptionMode?: "tooltip" | "inline"; grouped?: boolean }> = ({
  descriptionMode = "inline",
  grouped = false,
}) => {
  const { getSetting, updateSetting, isUpdating } = useSettings();
  const current = getSetting("auto_trigger_min_interval_seconds") ?? 75;
  const [value, setValue] = useState(String(current));

  useEffect(() => {
    setValue(String(current));
  }, [current]);

  const commit = async () => {
    const parsed = parseInt(value, 10);
    if (isNaN(parsed)) {
      setValue(String(current));
      return;
    }
    const clamped = Math.min(600, Math.max(30, parsed));
    setValue(String(clamped));
    await updateSetting("auto_trigger_min_interval_seconds", clamped);
  };

  return (
    <SettingContainer
      title="Automation Debounce Interval"
      description="Minimum seconds between automated /meeting triggers (30–600). Prevents spam."
      descriptionMode={descriptionMode}
      grouped={grouped}
      layout="horizontal"
    >
      <div className="flex items-center space-x-2">
        <Input
          type="number"
          min={30}
          max={600}
          step={1}
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onBlur={commit}
          onKeyDown={(e) => {
            if (e.key === "Enter") commit();
          }}
          disabled={isUpdating("auto_trigger_min_interval_seconds")}
          className="w-20"
        />
        <span className="text-sm text-text">seconds</span>
      </div>
    </SettingContainer>
  );
};

