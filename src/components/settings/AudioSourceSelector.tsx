import React, { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Dropdown } from "../ui/Dropdown";
import { SettingContainer } from "../ui/SettingContainer";

interface VirtualDeviceInfo {
  name: string;
  available: boolean;
  device_id: string;
  sample_rate: number;
  channels: number;
}

interface AudioSourceSelectorProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const AudioSourceSelector: React.FC<AudioSourceSelectorProps> = ({
  descriptionMode = "tooltip",
  grouped = false,
}) => {
  const [devices, setDevices] = useState<VirtualDeviceInfo[]>([]);
  const [currentSource, setCurrentSource] = useState<string>("");
  const [loading, setLoading] = useState<boolean>(false);
  const [updating, setUpdating] = useState<boolean>(false);

  const refresh = useCallback(async () => {
    try {
      setLoading(true);
      const [devs, source] = await Promise.all([
        invoke<VirtualDeviceInfo[]>("list_system_audio_devices"),
        invoke<string>("get_current_audio_source"),
      ]);
      setDevices(devs);
      setCurrentSource(source);
    } catch (e) {
      console.error("Failed to refresh audio source info:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const selectedValue =
    currentSource === "microphone"
      ? "microphone"
      : currentSource.startsWith("system:")
      ? currentSource.slice("system:".length)
      : null;

  const handleSelect = async (value: string) => {
    try {
      setUpdating(true);
      if (value === "microphone") {
        await invoke("set_microphone_source");
        setCurrentSource("microphone");
      } else {
        await invoke("set_system_audio_source", { deviceName: value });
        setCurrentSource(`system:${value}`);
      }
    } catch (e) {
      console.error("Failed to switch audio source:", e);
    } finally {
      setUpdating(false);
    }
  };

  const options = [
    { value: "microphone", label: "Microphone (use selected input)" },
    ...devices.map((d) => ({ value: d.name, label: `System: ${d.name}` })),
  ];

  return (
    <SettingContainer
      title="Audio Source"
      description="Choose to capture from your microphone or a system audio device (e.g., BlackHole on macOS)"
      descriptionMode={descriptionMode}
      grouped={grouped}
    >
      <div className="flex items-center space-x-1">
        <Dropdown
          options={options}
          selectedValue={selectedValue}
          onSelect={handleSelect}
          placeholder={loading ? "Loading..." : "Select audio source..."}
          disabled={loading || updating || options.length === 0}
          onRefresh={refresh}
        />
      </div>
    </SettingContainer>
  );
};

