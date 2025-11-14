import React from "react";
import { useSettings } from "../../hooks/useSettings";
import { ToggleSwitch } from "../ui/ToggleSwitch";

interface Props {
  descriptionMode?: "tooltip" | "inline";
  grouped?: boolean;
  disabled?: boolean;
}

export const GitHubEnabled: React.FC<Props> = ({
  descriptionMode = "tooltip",
  grouped = false,
  disabled = false,
}) => {
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
      disabled={disabled}
    />
  );
};
