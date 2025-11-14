import React, { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AudioSourceSelector } from "../settings/AudioSourceSelector";
import { AlertCircle, CheckCircle2, Info, ExternalLink, RefreshCw, Music2 } from "lucide-react";

type BackgroundMusicStatus = {
  supported: boolean;
  installed: boolean;
  running: boolean;
  install_paths: string[];
};

type BackgroundMusicState = "checking" | "running" | "installed" | "missing" | "unsupported";

export const AudioSetupSection: React.FC = () => {
  const [currentSource, setCurrentSource] = useState<string>("");
  const [showSetupGuide, setShowSetupGuide] = useState(false);
  const [bgMusicStatus, setBgMusicStatus] = useState<BackgroundMusicStatus | null>(null);
  const [bgMusicLoading, setBgMusicLoading] = useState(false);

  const refreshBackgroundMusicStatus = useCallback(async () => {
    setBgMusicLoading(true);
    try {
      const status = await invoke<BackgroundMusicStatus>("get_background_music_status");
      setBgMusicStatus(status);
    } catch (err) {
      console.error("Failed to fetch Background Music status", err);
    } finally {
      setBgMusicLoading(false);
    }
  }, []);

  useEffect(() => {
    const checkAudioSource = async () => {
      try {
        const source = await invoke<string>("get_current_audio_source");
        setCurrentSource(source);
      } catch (e) {
        console.error("Failed to get audio source:", e);
      }
    };

    checkAudioSource();
    const interval = setInterval(checkAudioSource, 2000);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    refreshBackgroundMusicStatus();
    const interval = setInterval(refreshBackgroundMusicStatus, 10000);
    return () => clearInterval(interval);
  }, [refreshBackgroundMusicStatus]);

  const isSystemAudio =
    (typeof currentSource === "string" &&
     (currentSource.includes("system:") || currentSource.includes("SystemAudio") ||
      currentSource.includes("BlackHole"))) ||
    (typeof currentSource === "object" && (currentSource as any).SystemAudio);

  const isMicrophone = currentSource === "microphone" ||
    (typeof currentSource === "string" && currentSource.toLowerCase().includes("microphone"));

  const backgroundMusicState: BackgroundMusicState = (() => {
    if (!bgMusicStatus) return "checking";
    if (!bgMusicStatus.supported) return "unsupported";
    if (bgMusicStatus.running) return "running";
    if (bgMusicStatus.installed) return "installed";
    return "missing";
  })();

  const backgroundMusicLabelMap: Record<BackgroundMusicState, string> = {
    checking: "Checking…",
    running: "Running",
    installed: "Installed (launch to control volume)",
    missing: "Not installed",
    unsupported: "macOS only",
  };

  const backgroundMusicBadgeClasses: Record<BackgroundMusicState, string> = {
    checking: "bg-gray-100 text-gray-600 border border-gray-200",
    running: "bg-green-100 text-green-700 border border-green-200",
    installed: "bg-yellow-100 text-yellow-700 border border-yellow-200",
    missing: "bg-red-100 text-red-700 border border-red-200",
    unsupported: "bg-gray-100 text-gray-500 border border-gray-200",
  };

  const backgroundMusicDescription = (() => {
    switch (backgroundMusicState) {
      case "running":
        return "Background Music is active. Use its per-app sliders to boost Zoom while MeetingCoder keeps BlackHole selected.";
      case "installed":
        return "Launch Background Music from the menu bar to unlock per-app volume control while routing audio through BlackHole.";
      case "missing":
        return "Install the free Background Music app to mirror audio to your headphones and keep system volume adjustable.";
      case "checking":
        return "Detecting Background Music installation…";
      case "unsupported":
      default:
        return "Background Music is only available on macOS.";
    }
  })();

  return (
    <div className="space-y-4">
      {/* Status Banner */}
      <div
        className={`p-4 rounded-lg border ${
          isSystemAudio
            ? "bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-800"
            : "bg-yellow-50 dark:bg-yellow-900/20 border-yellow-200 dark:border-yellow-800"
        }`}
      >
        <div className="flex items-start gap-3">
          {isSystemAudio ? (
            <CheckCircle2 className="w-5 h-5 text-green-600 dark:text-green-400 flex-shrink-0 mt-0.5" />
          ) : (
            <AlertCircle className="w-5 h-5 text-yellow-600 dark:text-yellow-400 flex-shrink-0 mt-0.5" />
          )}
          <div className="flex-1">
            <h3 className="text-sm font-semibold mb-1">
              {isSystemAudio
                ? "✓ Ready for Zoom/Google Meet"
                : isMicrophone
                ? "⚠️ Currently Using Microphone"
                : "⚠️ Audio Setup Required"}
            </h3>
            <p className="text-xs text-gray-700 dark:text-gray-300">
              {isSystemAudio ? (
                <>
                  System audio capture is active. You can now start your meeting and it will
                  transcribe Zoom/Google Meet audio.
                </>
              ) : isMicrophone ? (
                <>
                  Microphone mode only captures YOUR voice, not Zoom/Meet participants. Switch to{" "}
                  <strong>BlackHole 2ch</strong> below to capture the full meeting.
                </>
              ) : (
                <>
                  For live Zoom/Google Meet transcription, select <strong>BlackHole 2ch</strong>{" "}
                  below.
                </>
              )}
            </p>
          </div>
        </div>
      </div>

      {/* Audio Source Selector */}
      <div className="bg-white dark:bg-gray-900 p-4 rounded-lg border border-gray-200 dark:border-gray-700">
        <AudioSourceSelector descriptionMode="inline" grouped={false} />
      </div>

      {/* Background Music Helper */}
      {backgroundMusicState !== "unsupported" && (
        <div className="bg-white dark:bg-gray-900 p-4 rounded-lg border border-gray-200 dark:border-gray-700">
          <div className="flex flex-col gap-3">
            <div className="flex flex-col lg:flex-row lg:items-start lg:justify-between gap-3">
              <div className="flex items-start gap-3">
                <div className={`p-2 rounded-full ${
                  backgroundMusicState === "running"
                    ? "bg-green-100 text-green-700"
                    : backgroundMusicState === "installed"
                    ? "bg-yellow-100 text-yellow-700"
                    : "bg-gray-100 text-gray-600"
                }`}>
                  <Music2 className="w-4 h-4" />
                </div>
                <div>
                  <div className="flex flex-wrap items-center gap-2">
                    <p className="text-sm font-semibold">Background Music (free volume helper)</p>
                    <span
                      className={`px-2 py-0.5 text-xs rounded-full ${backgroundMusicBadgeClasses[backgroundMusicState]}`}
                    >
                      {backgroundMusicLabelMap[backgroundMusicState]}
                    </span>
                  </div>
                  <p className="text-xs text-gray-700 dark:text-gray-300 mt-1">
                    {backgroundMusicDescription}
                  </p>
                  {bgMusicStatus?.install_paths?.length ? (
                    <p className="text-[11px] text-gray-500 mt-1">
                      Detected at {bgMusicStatus.install_paths[0]}
                      {bgMusicStatus.install_paths.length > 1 ? " (multiple copies found)" : ""}
                    </p>
                  ) : null}
                </div>
              </div>
              <button
                onClick={refreshBackgroundMusicStatus}
                disabled={bgMusicLoading}
                className="inline-flex items-center gap-2 px-3 py-1.5 text-xs border rounded-md hover:bg-gray-50 dark:hover:bg-gray-800 disabled:opacity-60"
              >
                <RefreshCw className={`w-3.5 h-3.5 ${bgMusicLoading ? "animate-spin" : ""}`} />
                {bgMusicLoading ? "Checking" : "Re-check"}
              </button>
            </div>
            <div className="flex flex-wrap gap-2 text-xs text-blue-600 dark:text-blue-400">
              <a
                href="https://github.com/kyleneideck/BackgroundMusic"
                target="_blank"
                rel="noopener noreferrer"
                className="inline-flex items-center gap-1 px-2 py-1 rounded-md bg-blue-50 dark:bg-blue-900/20 hover:bg-blue-100 dark:hover:bg-blue-900/40"
              >
                Download Background Music
                <ExternalLink className="w-3 h-3" />
              </a>
              <div className="flex items-center gap-1 px-2 py-1 rounded-md bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-200">
                <code>brew install --cask background-music</code>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Setup Guide Toggle */}
      {!isSystemAudio && (
        <button
          onClick={() => setShowSetupGuide(!showSetupGuide)}
          className="text-sm text-blue-600 dark:text-blue-400 hover:underline flex items-center gap-1"
        >
          <Info className="w-4 h-4" />
          {showSetupGuide ? "Hide" : "Show"} BlackHole Setup Guide
        </button>
      )}

      {/* Setup Guide */}
      {showSetupGuide && (
        <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4 space-y-3">
          <h4 className="text-sm font-semibold flex items-center gap-2">
            <Info className="w-4 h-4" />
            Quick Setup Guide for Zoom/Google Meet
          </h4>

          <div className="text-xs space-y-3">
            <div>
              <p className="font-semibold mb-1">Step 1: Install BlackHole</p>
              <a
                href="https://existential.audio/blackhole/"
                target="_blank"
                rel="noopener noreferrer"
                className="text-blue-600 dark:text-blue-400 hover:underline flex items-center gap-1"
              >
                Download BlackHole 2ch (Free)
                <ExternalLink className="w-3 h-3" />
              </a>
            </div>

            <div>
              <p className="font-semibold mb-1">Step 2: Create Multi-Output Device</p>
              <ol className="list-decimal list-inside space-y-1 text-gray-700 dark:text-gray-300 ml-2">
                <li>Open "Audio MIDI Setup" (in Applications/Utilities)</li>
                <li>Click "+" button → "Create Multi-Output Device"</li>
                <li>Check BOTH "BlackHole 2ch" AND your speakers/headphones</li>
                <li>Right-click Multi-Output → "Use This Device For Sound Output"</li>
              </ol>
            </div>

            <div>
              <p className="font-semibold mb-1">Optional: Background Music for Volume Control</p>
              <p className="text-gray-700 dark:text-gray-300 ml-2">
                Install the free, open-source <strong>Background Music</strong> app to keep BlackHole
                selected while still adjusting Zoom/system volume with per-app sliders.
              </p>
              <a
                href="https://github.com/kyleneideck/BackgroundMusic"
                target="_blank"
                rel="noopener noreferrer"
                className="text-blue-600 dark:text-blue-400 hover:underline flex items-center gap-1 ml-2"
              >
                Download Background Music (Free)
                <ExternalLink className="w-3 h-3" />
              </a>
              <ol className="list-decimal list-inside space-y-1 text-gray-700 dark:text-gray-300 ml-2 mt-2">
                <li>
                  <code>brew install --cask background-music</code>
                </li>
                <li>Launch Background Music and raise the Zoom/system slider to a comfortable level</li>
                <li>Leave BlackHole selected in MeetingCoder; Background Music handles monitoring volume</li>
              </ol>
            </div>

            <div>
              <p className="font-semibold mb-1">Step 3: Configure Zoom/Meet</p>
              <p className="text-gray-700 dark:text-gray-300 ml-2">
                In your meeting app's audio settings, set the <strong>output device</strong> to
                the Multi-Output Device you just created.
              </p>
            </div>

            <div>
              <p className="font-semibold mb-1">Step 4: Select BlackHole Above</p>
              <p className="text-gray-700 dark:text-gray-300 ml-2">
                In the dropdown above, select <strong>"System: BlackHole 2ch"</strong>
              </p>
            </div>

            <div className="pt-2 border-t border-blue-200 dark:border-blue-700">
              <p className="font-semibold text-green-700 dark:text-green-400">
                ✓ That's it! You'll hear audio through your headphones AND MeetingCoder will
                transcribe it.
              </p>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
