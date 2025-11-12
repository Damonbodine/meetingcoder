import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AudioSourceSelector } from "../settings/AudioSourceSelector";
import { AlertCircle, CheckCircle2, Info, ExternalLink } from "lucide-react";

export const AudioSetupSection: React.FC = () => {
  const [currentSource, setCurrentSource] = useState<string>("");
  const [showSetupGuide, setShowSetupGuide] = useState(false);

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

  const isSystemAudio =
    (typeof currentSource === "string" &&
     (currentSource.includes("system:") || currentSource.includes("SystemAudio") ||
      currentSource.includes("BlackHole"))) ||
    (typeof currentSource === "object" && (currentSource as any).SystemAudio);

  const isMicrophone = currentSource === "microphone" ||
    (typeof currentSource === "string" && currentSource.toLowerCase().includes("microphone"));

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
