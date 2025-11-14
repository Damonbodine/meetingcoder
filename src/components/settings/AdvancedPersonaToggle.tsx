import React from "react";
import { useSettings } from "../../hooks/useSettings";
import { ToggleSwitch } from "../ui/ToggleSwitch";

interface Props {
  descriptionMode?: "tooltip" | "inline";
  grouped?: boolean;
}

export const AdvancedPersonaToggle: React.FC<Props> = ({
  descriptionMode = "tooltip",
  grouped = false,
}) => {
  const { getSetting, updateSetting, isUpdating } = useSettings();
  const value = getSetting("advanced_features_enabled") ?? false;

  return (
    <div className="space-y-2">
      <ToggleSwitch
        checked={value}
        onChange={(enabled) => updateSetting("advanced_features_enabled", enabled)}
        isUpdating={isUpdating("advanced_features_enabled")}
        label="Enable Advanced Automations"
        description="Recorder mode stays local. Advanced mode unlocks GitHub pushes, Claude summaries, IDE launchers, and automation hooks."
        descriptionMode={descriptionMode}
        grouped={grouped}
      />
      <div className="px-4 text-xs text-muted-foreground">
        {value ? "Advanced mode is active." : "Recorder mode is active."} Advanced mode may request repo write access and run scripts on your behalf. Use it only on trusted machines and repos.
      </div>
      <ul className="px-6 list-disc text-xs text-muted-foreground space-y-1">
        <li>Recorder mode: live transcription, local imports, summaries.</li>
        <li>Advanced mode additionally enables GitHub automation, device flow auth, `/meeting` triggers, and PR creation.</li>
      </ul>
    </div>
  );
};

