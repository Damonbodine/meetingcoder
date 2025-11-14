import React from "react";
import { MicrophoneSelector } from "./MicrophoneSelector";
import { AudioSourceSelector } from "./AudioSourceSelector";
import { LanguageSelector } from "./LanguageSelector";
import { AppShortcut } from "./AppShortcut";
import { SettingsGroup } from "../ui/SettingsGroup";
import { OutputDeviceSelector } from "./OutputDeviceSelector";
import { PushToTalk } from "./PushToTalk";
import { AudioFeedback } from "./AudioFeedback";
import { useSettings } from "../../hooks/useSettings";
import { VolumeSlider } from "./VolumeSlider";
import { ChunkDuration } from "./ChunkDuration";
import { UpdateInterval } from "./UpdateInterval";
import { AutoTriggerToggle } from "./AutoTriggerToggle";
import { AutoAcceptChanges } from "./AutoAcceptChanges";
import { AutomationDebounce } from "./AutomationDebounce";
import { AdvancedPersonaToggle } from "./AdvancedPersonaToggle";
import { OfflineModeToggle } from "./OfflineModeToggle";

export const GeneralSettings: React.FC = () => {
  const { audioFeedbackEnabled, getSetting } = useSettings();
  const advancedEnabled = getSetting("advanced_features_enabled") ?? false;
  const offlineMode = getSetting("offline_mode_enabled") ?? false;
  const automationsDisabled = !advancedEnabled || offlineMode;

  return (
    <div className="max-w-3xl w-full mx-auto space-y-6">
      <SettingsGroup title="Modes">
        <AdvancedPersonaToggle descriptionMode="inline" grouped={true} />
        <OfflineModeToggle descriptionMode="inline" grouped={true} />
      </SettingsGroup>
      <SettingsGroup title="General">
        <AppShortcut descriptionMode="tooltip" grouped={true} />
        <LanguageSelector descriptionMode="tooltip" grouped={true} />
        <PushToTalk descriptionMode="tooltip" grouped={true} />
        <ChunkDuration descriptionMode="tooltip" grouped={true} />
        <UpdateInterval descriptionMode="tooltip" grouped={true} />
        <AutoTriggerToggle
          descriptionMode="tooltip"
          grouped={true}
          disabled={automationsDisabled}
        />
        <AutomationDebounce
          descriptionMode="tooltip"
          grouped={true}
          disabled={automationsDisabled}
        />
        <AutoAcceptChanges
          descriptionMode="tooltip"
          grouped={true}
          disabled={automationsDisabled}
        />
        {automationsDisabled && (
          <p className="px-4 text-xs text-muted-foreground">
            Automation controls are disabled in Recorder/offline mode. Enable Advanced mode and go online to control Claude or GitHub workflows.
          </p>
        )}
      </SettingsGroup>
      <SettingsGroup title="Sound">
        <AudioSourceSelector descriptionMode="tooltip" grouped={true} />
        <MicrophoneSelector descriptionMode="tooltip" grouped={true} />
        <AudioFeedback descriptionMode="tooltip" grouped={true} />
        <OutputDeviceSelector
          descriptionMode="tooltip"
          grouped={true}
          disabled={!audioFeedbackEnabled}
        />
        <VolumeSlider disabled={!audioFeedbackEnabled} />
      </SettingsGroup>
      {/* GitHub moved under Integrations */}
    </div>
  );
};
