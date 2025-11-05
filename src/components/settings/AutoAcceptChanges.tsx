import React from "react";
import { useSettings } from "../../hooks/useSettings";
import { ToggleSwitch } from "../ui/ToggleSwitch";

export const AutoAcceptChanges: React.FC<{ descriptionMode?: "tooltip" | "inline"; grouped?: boolean }> = ({
  descriptionMode = "tooltip",
  grouped = false,
}) => {
  const { getSetting, updateSetting, isUpdating } = useSettings();
  const value = getSetting("auto_accept_changes") ?? false;

  return (
    <ToggleSwitch
      checked={!!value}
      onChange={(v) => updateSetting("auto_accept_changes", v)}
      isUpdating={isUpdating("auto_accept_changes")}
      label="Auto-accept Claude changes"
      description="After sending /meeting, automatically send 'y' + Return to accept changes. Requires macOS Accessibility permission."
      descriptionMode={descriptionMode}
      grouped={grouped}
    />
  );
};

