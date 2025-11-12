import React from "react";
import { useSettingsStore } from "../../stores/settingsStore";

interface Props {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const UseFixedWindowsForImports: React.FC<Props> = ({ descriptionMode = "inline" }) => {
  const settings = useSettingsStore((s) => s.settings);
  const updateSetting = useSettingsStore((s) => s.updateSetting);
  const value = settings?.use_fixed_windows_for_imports ?? false;

  const onToggle = async (e: React.ChangeEvent<HTMLInputElement>) => {
    await updateSetting("use_fixed_windows_for_imports", e.target.checked);
  };

  return (
    <div className="flex items-start justify-between py-2">
      <div>
        <div className="font-medium">Use fixed 45s windows for imports</div>
        {descriptionMode === "inline" && (
          <div className="text-sm text-neutral-500">
            Bypass VAD segmentation and transcribe 45s windows with ~0.9s overlap.
          </div>
        )}
      </div>
      <label className="inline-flex items-center cursor-pointer">
        <input type="checkbox" className="sr-only peer" checked={value} onChange={onToggle} />
        <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-blue-300 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600 relative" />
      </label>
    </div>
  );
};

