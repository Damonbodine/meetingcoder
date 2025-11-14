import React from "react";
import { useSettings } from "../../hooks/useSettings";
import { ToggleSwitch } from "../ui/ToggleSwitch";

interface Props {
  descriptionMode?: "tooltip" | "inline";
  grouped?: boolean;
  disabled?: boolean;
}

export const AutoTriggerToggle: React.FC<Props> = ({
  descriptionMode = "tooltip",
  grouped = false,
  disabled = false,
}) => {
  const { getSetting, updateSetting, isUpdating } = useSettings();
  const value = getSetting("auto_trigger_meeting_command") ?? false;

  return (
    <ToggleSwitch
      checked={!!value}
      onChange={(v) => updateSetting("auto_trigger_meeting_command", v)}
      isUpdating={isUpdating("auto_trigger_meeting_command")}
      label="Auto-trigger /meeting"
      description="Prepare automation to trigger Claude's /meeting command periodically (Phase 3)."
      descriptionMode={descriptionMode}
      grouped={grouped}
      disabled={disabled}
    />
  );
};
