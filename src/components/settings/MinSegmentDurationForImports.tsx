import React from "react";
import { useSettingsStore } from "../../stores/settingsStore";

interface Props {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const MinSegmentDurationForImports: React.FC<Props> = ({ descriptionMode = "inline" }) => {
  const settings = useSettingsStore((s) => s.settings);
  const updateSetting = useSettingsStore((s) => s.updateSetting);
  const value = settings?.min_segment_duration_for_imports ?? 10;

  const onChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const v = Math.max(5, Math.min(15, parseInt(e.target.value, 10)));
    await updateSetting("min_segment_duration_for_imports", v);
  };

  return (
    <div className="flex items-start justify-between py-2">
      <div className="w-full">
        <div className="font-medium">Min segment duration (sec)</div>
        {descriptionMode === "inline" && (
          <div className="text-sm text-neutral-500">
            After VAD, merge short segments up to this minimum to reduce overhead. Range 5–15s.
          </div>
        )}
      </div>
      <div className="flex items-center gap-3">
        <input type="range" min={5} max={15} value={value} onChange={onChange} />
        <div className="text-sm w-8 text-right">{value}s</div>
      </div>
    </div>
  );
};

