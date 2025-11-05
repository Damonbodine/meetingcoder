import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { Check, X, Loader2, RefreshCw, AlertCircle } from "lucide-react";

interface VirtualDeviceInfo {
  name: string;
  available: boolean;
  device_id: string;
  sample_rate: number;
  channels: number;
}

interface DetectDeviceResponse {
  available: boolean;
  device: VirtualDeviceInfo | null;
}

export default function SystemAudioTest() {
  const [isSupported, setIsSupported] = useState<boolean | null>(null);
  const [detectedDevice, setDetectedDevice] = useState<DetectDeviceResponse | null>(null);
  const [allDevices, setAllDevices] = useState<VirtualDeviceInfo[]>([]);
  const [setupInstructions, setSetupInstructions] = useState<string>("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [currentSource, setCurrentSource] = useState<string>("");
  const [switchingSource, setSwitchingSource] = useState(false);
  const [bufferSize, setBufferSize] = useState<number>(0);
  const [isRecording, setIsRecording] = useState(false);
  const [savedFilePath, setSavedFilePath] = useState<string | null>(null);

  const checkSupport = async () => {
    try {
      const supported = await invoke<boolean>("is_system_audio_supported");
      setIsSupported(supported);
    } catch (err) {
      setError(`Failed to check support: ${err}`);
    }
  };

  const detectDevice = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<DetectDeviceResponse>("detect_virtual_audio_device");
      setDetectedDevice(result);
    } catch (err) {
      setError(`Failed to detect device: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const listDevices = async () => {
    setLoading(true);
    setError(null);
    try {
      const devices = await invoke<VirtualDeviceInfo[]>("list_system_audio_devices");
      setAllDevices(devices);
    } catch (err) {
      setError(`Failed to list devices: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const getInstructions = async () => {
    try {
      const instructions = await invoke<string>("get_system_audio_setup_instructions");
      setSetupInstructions(instructions);
    } catch (err) {
      setError(`Failed to get instructions: ${err}`);
    }
  };

  const getCurrentSource = async () => {
    try {
      const source = await invoke<string>("get_current_audio_source");
      setCurrentSource(source);
    } catch (err) {
      console.error("Failed to get current source:", err);
    }
  };

  const getBufferSize = async () => {
    try {
      const size = await invoke<number>("get_system_audio_buffer_size");
      setBufferSize(size);
    } catch (err) {
      console.error("Failed to get buffer size:", err);
    }
  };

  const startTestRecording = async () => {
    setError(null);
    setSavedFilePath(null);
    try {
      // Clear buffer first
      await invoke("clear_system_audio_buffer");
      setBufferSize(0);
      setIsRecording(true);
    } catch (err) {
      setError(`Failed to start recording: ${err}`);
    }
  };

  const stopAndSaveRecording = async () => {
    setError(null);
    try {
      const timestamp = new Date().toISOString().replace(/[:.]/g, "-");
      const filename = `test_recording_${timestamp}.wav`;

      const filePath = await invoke<string>("save_system_audio_buffer_to_wav", {
        filename,
      });

      setSavedFilePath(filePath);
      setIsRecording(false);

      alert(`Recording saved to Desktop!\n\nFile: ${filename}\n\nSamples: ${bufferSize} (${(bufferSize / 16000).toFixed(1)}s)\n\nCheck your Desktop for the WAV file.`);
    } catch (err) {
      setError(`Failed to save recording: ${err}`);
      setIsRecording(false);
    }
  };

  const clearBuffer = async () => {
    try {
      await invoke("clear_system_audio_buffer");
      setBufferSize(0);
      setSavedFilePath(null);
    } catch (err) {
      setError(`Failed to clear buffer: ${err}`);
    }
  };

  const switchToMicrophone = async () => {
    setSwitchingSource(true);
    setError(null);
    try {
      await invoke("set_microphone_source");
      await getCurrentSource();
    } catch (err) {
      setError(`Failed to switch to microphone: ${err}`);
    } finally {
      setSwitchingSource(false);
    }
  };

  const switchToSystemAudio = async (deviceName: string) => {
    setSwitchingSource(true);
    setError(null);
    try {
      await invoke("set_system_audio_source", { deviceName });
      await getCurrentSource();
    } catch (err) {
      setError(`Failed to switch to system audio: ${err}`);
    } finally {
      setSwitchingSource(false);
    }
  };

  const runAllTests = async () => {
    await checkSupport();
    await detectDevice();
    await listDevices();
    await getInstructions();
    await getCurrentSource();
  };

  useEffect(() => {
    runAllTests();
  }, []);

  // Poll buffer size while recording
  useEffect(() => {
    if (!isRecording) return;

    const interval = setInterval(() => {
      getBufferSize();
    }, 500); // Update every 500ms

    return () => clearInterval(interval);
  }, [isRecording]);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold">System Audio Capture Test</h2>
          <p className="text-sm text-mid-gray">
            Test system audio capture functionality for meeting mode
          </p>
        </div>
        <button
          onClick={runAllTests}
          disabled={loading}
          className="flex items-center gap-2 px-4 py-2 bg-logo-primary/80 hover:bg-logo-primary rounded-lg transition-colors disabled:opacity-50"
        >
          {loading ? (
            <Loader2 className="w-4 h-4 animate-spin" />
          ) : (
            <RefreshCw className="w-4 h-4" />
          )}
          Refresh Tests
        </button>
      </div>

      {error && (
        <div className="flex items-start gap-2 p-4 bg-red-500/10 border border-red-500/20 rounded-lg">
          <AlertCircle className="w-5 h-5 text-red-500 flex-shrink-0 mt-0.5" />
          <div className="flex-1">
            <p className="text-sm font-medium text-red-500">Error</p>
            <p className="text-sm text-red-400 mt-1">{error}</p>
          </div>
        </div>
      )}

      {/* Platform Support */}
      <div className="border border-mid-gray/20 rounded-lg p-4">
        <div className="flex items-center justify-between mb-2">
          <h3 className="font-medium">Platform Support</h3>
          {isSupported !== null && (
            <div className="flex items-center gap-2">
              {isSupported ? (
                <>
                  <Check className="w-5 h-5 text-green-500" />
                  <span className="text-sm text-green-500">Supported</span>
                </>
              ) : (
                <>
                  <X className="w-5 h-5 text-red-500" />
                  <span className="text-sm text-red-500">Not Supported</span>
                </>
              )}
            </div>
          )}
        </div>
        <p className="text-sm text-mid-gray">
          System audio capture is {isSupported ? "available" : "not available"} on this platform
        </p>
      </div>

      {/* Virtual Device Detection */}
      <div className="border border-mid-gray/20 rounded-lg p-4">
        <div className="flex items-center justify-between mb-2">
          <h3 className="font-medium">Virtual Device Detection</h3>
          {detectedDevice && (
            <div className="flex items-center gap-2">
              {detectedDevice.available ? (
                <>
                  <Check className="w-5 h-5 text-green-500" />
                  <span className="text-sm text-green-500">Detected</span>
                </>
              ) : (
                <>
                  <X className="w-5 h-5 text-yellow-500" />
                  <span className="text-sm text-yellow-500">Not Found</span>
                </>
              )}
            </div>
          )}
        </div>

        {loading && !detectedDevice ? (
          <div className="flex items-center gap-2 text-sm text-mid-gray">
            <Loader2 className="w-4 h-4 animate-spin" />
            Detecting...
          </div>
        ) : detectedDevice?.device ? (
          <div className="space-y-2 mt-3">
            <div className="bg-mid-gray/10 rounded p-3 space-y-1">
              <p className="text-sm">
                <span className="font-medium">Name:</span> {detectedDevice.device.name}
              </p>
              <p className="text-sm">
                <span className="font-medium">Device ID:</span> {detectedDevice.device.device_id}
              </p>
              <p className="text-sm">
                <span className="font-medium">Sample Rate:</span> {detectedDevice.device.sample_rate} Hz
              </p>
              <p className="text-sm">
                <span className="font-medium">Channels:</span> {detectedDevice.device.channels}
              </p>
            </div>
          </div>
        ) : (
          <p className="text-sm text-mid-gray mt-2">
            No virtual audio device detected. Install BlackHole or Loopback to capture system audio.
          </p>
        )}
      </div>

      {/* Test Recording */}
      <div className="border border-logo-primary/30 bg-logo-primary/5 rounded-lg p-4">
        <h3 className="font-medium mb-3">Test Recording</h3>

        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <div className="flex-1">
              <p className="text-sm text-mid-gray mb-1">Buffer Status:</p>
              <div className="flex items-center gap-3">
                <p className="text-2xl font-mono font-bold">
                  {bufferSize.toLocaleString()}
                </p>
                <p className="text-sm text-mid-gray">
                  samples ({(bufferSize / 16000).toFixed(1)}s)
                </p>
                {isRecording && (
                  <div className="flex items-center gap-2 text-red-500">
                    <div className="w-3 h-3 bg-red-500 rounded-full animate-pulse" />
                    <span className="text-sm font-medium">RECORDING</span>
                  </div>
                )}
              </div>
            </div>
          </div>

          <div className="flex items-center gap-2">
            {!isRecording ? (
              <button
                onClick={startTestRecording}
                disabled={!currentSource.startsWith("system:")}
                className="px-4 py-2 bg-green-600 hover:bg-green-700 rounded-lg transition-colors disabled:opacity-50 text-sm font-medium"
              >
                Start Test Recording
              </button>
            ) : (
              <button
                onClick={stopAndSaveRecording}
                className="px-4 py-2 bg-red-600 hover:bg-red-700 rounded-lg transition-colors text-sm font-medium"
              >
                Stop & Save Recording
              </button>
            )}

            <button
              onClick={clearBuffer}
              disabled={bufferSize === 0 || isRecording}
              className="px-4 py-2 bg-mid-gray/20 hover:bg-mid-gray/30 rounded-lg transition-colors disabled:opacity-50 text-sm"
            >
              Clear Buffer
            </button>
          </div>

          {!currentSource.startsWith("system:") && (
            <p className="text-sm text-yellow-500">
              ⚠️ Please switch to a system audio device first
            </p>
          )}

          {savedFilePath && (
            <div className="bg-green-500/10 border border-green-500/20 rounded-lg p-3">
              <p className="text-sm text-green-500 font-medium">
                ✓ Recording saved successfully!
              </p>
              <p className="text-xs text-green-400 mt-1 font-mono break-all">
                {savedFilePath}
              </p>
            </div>
          )}
        </div>
      </div>

      {/* Current Audio Source */}
      <div className="border border-mid-gray/20 rounded-lg p-4">
        <h3 className="font-medium mb-3">Current Audio Source</h3>
        <div className="flex items-center gap-4">
          <div className="flex-1">
            <p className="text-sm text-mid-gray mb-2">Active source:</p>
            <p className="text-sm font-mono bg-mid-gray/10 rounded px-3 py-2">
              {currentSource || "Loading..."}
            </p>
          </div>
          <button
            onClick={switchToMicrophone}
            disabled={switchingSource || currentSource === "microphone"}
            className="px-4 py-2 bg-logo-primary/80 hover:bg-logo-primary rounded-lg transition-colors disabled:opacity-50 text-sm"
          >
            Switch to Microphone
          </button>
        </div>
      </div>

      {/* All Devices List */}
      <div className="border border-mid-gray/20 rounded-lg p-4">
        <h3 className="font-medium mb-3">All Available Devices ({allDevices.length})</h3>

        {loading && allDevices.length === 0 ? (
          <div className="flex items-center gap-2 text-sm text-mid-gray">
            <Loader2 className="w-4 h-4 animate-spin" />
            Loading devices...
          </div>
        ) : allDevices.length > 0 ? (
          <div className="space-y-2">
            {allDevices.map((device, index) => (
              <div
                key={index}
                className="bg-mid-gray/10 rounded p-3"
              >
                <div className="flex items-center justify-between mb-2">
                  <div className="flex-1">
                    <p className="text-sm font-medium">{device.name}</p>
                    <p className="text-xs text-mid-gray mt-1">ID: {device.device_id}</p>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="text-xs px-2 py-1 bg-logo-primary/20 rounded">
                      {device.channels} ch @ {device.sample_rate / 1000}kHz
                    </span>
                    <button
                      onClick={() => switchToSystemAudio(device.name)}
                      disabled={switchingSource || currentSource === `system:${device.name}`}
                      className="px-3 py-1 bg-logo-primary/80 hover:bg-logo-primary rounded text-xs transition-colors disabled:opacity-50"
                    >
                      {currentSource === `system:${device.name}` ? "Active" : "Use"}
                    </button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <p className="text-sm text-mid-gray">No devices found</p>
        )}
      </div>

      {/* Setup Instructions */}
      {setupInstructions && (
        <div className="border border-mid-gray/20 rounded-lg p-4">
          <h3 className="font-medium mb-3">Setup Instructions</h3>
          <pre className="text-xs bg-mid-gray/10 rounded p-3 overflow-x-auto whitespace-pre-wrap font-mono">
            {setupInstructions}
          </pre>
        </div>
      )}

      {/* Test Summary */}
      <div className="border border-logo-primary/30 bg-logo-primary/5 rounded-lg p-4">
        <h3 className="font-medium mb-2">Test Summary</h3>
        <ul className="space-y-1 text-sm">
          <li className="flex items-center gap-2">
            {isSupported ? (
              <Check className="w-4 h-4 text-green-500" />
            ) : (
              <X className="w-4 h-4 text-red-500" />
            )}
            Platform support check
          </li>
          <li className="flex items-center gap-2">
            {detectedDevice?.available ? (
              <Check className="w-4 h-4 text-green-500" />
            ) : (
              <AlertCircle className="w-4 h-4 text-yellow-500" />
            )}
            Virtual device detection
          </li>
          <li className="flex items-center gap-2">
            {allDevices.length > 0 ? (
              <Check className="w-4 h-4 text-green-500" />
            ) : (
              <X className="w-4 h-4 text-red-500" />
            )}
            Device enumeration ({allDevices.length} devices)
          </li>
          <li className="flex items-center gap-2">
            {setupInstructions ? (
              <Check className="w-4 h-4 text-green-500" />
            ) : (
              <X className="w-4 h-4 text-red-500" />
            )}
            Setup instructions loaded
          </li>
        </ul>
      </div>
    </div>
  );
}
