import React from "react";
import { ShowOverlay } from "./ShowOverlay";
import { TranslateToEnglish } from "./TranslateToEnglish";
import { ModelUnloadTimeoutSetting } from "./ModelUnloadTimeout";
import { CustomWords } from "./CustomWords";
import { SettingsGroup } from "../ui/SettingsGroup";
import { StartHidden } from "./StartHidden";
import { PreferWhisperForImports } from "./PreferWhisperForImports";
import { FastImportModeForImports } from "./FastImportModeForImports";
import { UseFixedWindowsForImports } from "./UseFixedWindowsForImports";
import { MinSegmentDurationForImports } from "./MinSegmentDurationForImports";
import { UseFfmpegFallbackForImports } from "./UseFfmpegFallbackForImports";
import { AutostartToggle } from "./AutostartToggle";
import { SystemAudioSilenceThreshold } from "./SystemAudioSilenceThreshold";
import { SystemAudioBufferSeconds } from "./SystemAudioBufferSeconds";

export const AdvancedSettings: React.FC = () => {
  return (
    <div className="max-w-3xl w-full mx-auto space-y-6">
      <SettingsGroup title="Advanced">
        <StartHidden descriptionMode="tooltip" grouped={true} />
        <AutostartToggle descriptionMode="tooltip" grouped={true} />
        <ShowOverlay descriptionMode="tooltip" grouped={true} />
        <TranslateToEnglish descriptionMode="tooltip" grouped={true} />
        <ModelUnloadTimeoutSetting descriptionMode="tooltip" grouped={true} />
        <CustomWords descriptionMode="tooltip" grouped />
        <PreferWhisperForImports descriptionMode="tooltip" />
        <FastImportModeForImports descriptionMode="tooltip" />
        <UseFixedWindowsForImports descriptionMode="tooltip" />
        <MinSegmentDurationForImports descriptionMode="tooltip" />
        <UseFfmpegFallbackForImports descriptionMode="tooltip" />
        <SystemAudioSilenceThreshold descriptionMode="tooltip" />
        <SystemAudioBufferSeconds descriptionMode="tooltip" />
      </SettingsGroup>
    </div>
  );
};
