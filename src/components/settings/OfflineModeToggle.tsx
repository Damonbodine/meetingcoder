import React from "react";
import { useSettings } from "../../hooks/useSettings";
import { ToggleSwitch } from "../ui/ToggleSwitch";

interface Props {
  descriptionMode?: "tooltip" | "inline";
  grouped?: boolean;
}

export const OfflineModeToggle: React.FC<Props> = ({
  descriptionMode = "tooltip",
  grouped = false,
}) => {
  const { getSetting, updateSetting, isUpdating } = useSettings();
  const value = getSetting("offline_mode_enabled") ?? false;

  return (
    <div className="space-y-2">
      <ToggleSwitch
        checked={value}
        onChange={(enabled) => updateSetting("offline_mode_enabled", enabled)}
        isUpdating={isUpdating("offline_mode_enabled")}
        label="Offline Mode"
        description="Cuts all network integrations. Claude, GitHub, and YouTube import are paused until you reconnect."
        descriptionMode={descriptionMode}
        grouped={grouped}
      />
      <div className="px-4 text-xs text-muted-foreground">
        When enabled, MeetingCoder keeps everything on-device and skips any commands that require internet or cloud credentials.
      </div>
      <ul className="px-6 list-disc text-xs text-muted-foreground space-y-1">
        <li>Disables Claude/API summarization and PR automation.</li>
        <li>Blocks GitHub pushes, repo cloning, and device authentication.</li>
        <li>Hides YouTube import and other network-based import sources.</li>
      </ul>
    </div>
  );
};

