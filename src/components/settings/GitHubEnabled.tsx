import React from "react";
import { useSettings } from "../../hooks/useSettings";
import { ToggleSwitch } from "../ui/ToggleSwitch";

export const GitHubEnabled: React.FC<{
  descriptionMode?: "tooltip" | "inline";
  grouped?: boolean;
}> = ({ descriptionMode = "tooltip", grouped = false }) => {
  const { getSetting, updateSetting, isUpdating } = useSettings();
  const value = getSetting("github_enabled") ?? false;

  return (
    <ToggleSwitch
      checked={!!value}
      onChange={(v) => updateSetting("github_enabled", v)}
      isUpdating={isUpdating("github_enabled")}
      label="Enable GitHub Integration"
      description="Enable automatic push and PR creation for meeting updates."
      descriptionMode={descriptionMode}
      grouped={grouped}
    />
  );
};
