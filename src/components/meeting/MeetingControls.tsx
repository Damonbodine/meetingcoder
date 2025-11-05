import { useState } from "react";
import { Play, Square, Pause } from "lucide-react";
import { Button } from "../ui/Button";
import { Input } from "../ui/Input";
import { SettingsGroup } from "../ui/SettingsGroup";
import { useSettingsStore } from "../../stores/settingsStore";

interface MeetingControlsProps {
  isActive: boolean;
  meetingName: string;
  isStarting: boolean;
  isEnding: boolean;
  onStart: (name: string) => void;
  onEnd: () => void;
  onPause: () => void;
  onResume: () => void;
}

export const MeetingControls = ({
  isActive,
  meetingName,
  isStarting,
  isEnding,
  onStart,
  onEnd,
  onPause,
  onResume,
}: MeetingControlsProps) => {
  const [nameInput, setNameInput] = useState("");
  const [isPaused, setIsPaused] = useState(false);
  const chunkSeconds =
    useSettingsStore((s) => s.settings?.transcription_chunk_seconds) ?? 10;

  const handleStart = () => {
    if (nameInput.trim()) {
      onStart(nameInput.trim());
      setNameInput("");
    }
  };

  const handlePauseResume = () => {
    if (isPaused) {
      onResume();
    } else {
      onPause();
    }
    setIsPaused(!isPaused);
  };

  return (
    <SettingsGroup title="Meeting Controls" description="Start and manage your meeting sessions">
      <div className="space-y-4">
        {!isActive ? (
          <div className="flex gap-2">
            <Input
              placeholder="Meeting name (e.g., Stakeholder Call)"
              value={nameInput}
              onChange={(e) => setNameInput(e.target.value)}
              onKeyPress={(e) => e.key === "Enter" && handleStart()}
              disabled={isStarting}
              className="flex-1"
            />
            <Button
              onClick={handleStart}
              disabled={!nameInput.trim() || isStarting}
              className="bg-green-600 hover:bg-green-700 text-white px-6"
            >
              <Play className="w-4 h-4 mr-2" />
              {isStarting ? "Starting..." : "Start Meeting"}
            </Button>
          </div>
        ) : (
          <div className="space-y-4">
            <div className="flex items-center justify-between p-4 bg-gray-100 dark:bg-gray-800 rounded-lg">
              <div className="flex items-center gap-3">
                <div className="w-3 h-3 bg-red-500 rounded-full animate-pulse" />
                <div>
                  <p className="font-semibold">{meetingName}</p>
                  <p className="text-sm text-gray-600 dark:text-gray-400">
                    {isPaused ? "Paused" : "Recording..."}
                  </p>
                </div>
              </div>

              <div className="flex gap-2">
                <Button
                  onClick={handlePauseResume}
                  className="bg-yellow-600 hover:bg-yellow-700 text-white"
                >
                  <Pause className="w-4 h-4 mr-2" />
                  {isPaused ? "Resume" : "Pause"}
                </Button>
                <Button
                  onClick={onEnd}
                  disabled={isEnding}
                  className="bg-red-600 hover:bg-red-700 text-white"
                >
                  <Square className="w-4 h-4 mr-2" />
                  {isEnding ? "Ending..." : "End Meeting"}
                </Button>
              </div>
            </div>

            <div className="text-sm text-gray-600 dark:text-gray-400">
              <p>• Transcription happens every ~{chunkSeconds} seconds</p>
              <p>• Transcript is automatically saved when meeting ends</p>
              <p>• Saved to: ~/MeetingCoder/meetings/</p>
            </div>
          </div>
        )}
      </div>
    </SettingsGroup>
  );
};
