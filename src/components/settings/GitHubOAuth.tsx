import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";

interface DeviceCodeResponse {
  device_code: string;
  user_code: string;
  verification_uri: string;
  expires_in: number;
  interval: number;
}

export const GitHubOAuth: React.FC = () => {
  const [isAuthenticating, setIsAuthenticating] = useState(false);
  const [deviceCode, setDeviceCode] = useState<DeviceCodeResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  // Poll for token
  useEffect(() => {
    if (!deviceCode) return;

    const pollInterval = setInterval(async () => {
      try {
        const token = await invoke<string | null>("github_poll_device_token", {
          deviceCode: deviceCode.device_code,
        });

        if (token) {
          // Token received! Save it
          await invoke("set_github_token", { token });
          setSuccess(true);
          setIsAuthenticating(false);
          setDeviceCode(null);
          clearInterval(pollInterval);
        }
      } catch (err) {
        console.error("Poll error:", err);
        setError(String(err));
        setIsAuthenticating(false);
        setDeviceCode(null);
        clearInterval(pollInterval);
      }
    }, (deviceCode.interval || 5) * 1000);

    // Cleanup on unmount or when deviceCode changes
    return () => clearInterval(pollInterval);
  }, [deviceCode]);

  const handleBeginAuth = async () => {
    setIsAuthenticating(true);
    setError(null);
    setSuccess(false);

    try {
      const response = await invoke<DeviceCodeResponse>("github_begin_device_auth");
      setDeviceCode(response);

      // Open verification URL in browser
      await openUrl(response.verification_uri);
    } catch (err) {
      console.error("Failed to begin device auth:", err);
      setError(String(err));
      setIsAuthenticating(false);
    }
  };

  const handleCancel = () => {
    setIsAuthenticating(false);
    setDeviceCode(null);
    setError(null);
  };

  const copyUserCode = () => {
    if (deviceCode) {
      navigator.clipboard.writeText(deviceCode.user_code);
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-medium">GitHub Authentication</h3>
          <p className="text-sm text-gray-500">
            Connect your GitHub account using OAuth (recommended)
          </p>
        </div>
        {!isAuthenticating && !success && (
          <button
            onClick={handleBeginAuth}
            className="px-4 py-2 bg-gray-900 text-white rounded-md hover:bg-gray-800 transition-colors"
          >
            Connect with GitHub
          </button>
        )}
      </div>

      {deviceCode && isAuthenticating && (
        <div className="p-6 bg-blue-50 border border-blue-200 rounded-lg space-y-4">
          <div className="text-center">
            <p className="text-sm text-gray-700 mb-2">
              To complete authentication, copy this code:
            </p>
            <div className="flex items-center justify-center gap-3">
              <code className="text-2xl font-mono font-bold bg-white px-6 py-3 rounded-md border-2 border-blue-400 tracking-wider">
                {deviceCode.user_code}
              </code>
              <button
                onClick={copyUserCode}
                className="px-3 py-2 bg-blue-600 text-white text-sm rounded-md hover:bg-blue-700"
              >
                Copy
              </button>
            </div>
            <p className="text-sm text-gray-600 mt-3">
              Then paste it at{" "}
              <a
                href={deviceCode.verification_uri}
                target="_blank"
                rel="noopener noreferrer"
                className="text-blue-600 hover:underline font-medium"
              >
                {deviceCode.verification_uri}
              </a>
            </p>
          </div>

          <div className="flex items-center justify-center space-x-2">
            <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-blue-600"></div>
            <span className="text-sm text-gray-600">
              Waiting for authorization...
            </span>
          </div>

          <div className="text-center">
            <button
              onClick={handleCancel}
              className="text-sm text-gray-500 hover:text-gray-700 underline"
            >
              Cancel
            </button>
          </div>
        </div>
      )}

      {success && (
        <div className="p-4 bg-green-50 border border-green-200 rounded-lg">
          <div className="flex items-center gap-2">
            <svg
              className="w-5 h-5 text-green-600"
              fill="none"
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth="2"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path d="M5 13l4 4L19 7"></path>
            </svg>
            <span className="text-green-800 font-medium">
              Successfully connected to GitHub!
            </span>
          </div>
        </div>
      )}

      {error && (
        <div className="p-4 bg-red-50 border border-red-200 rounded-lg">
          <p className="text-red-800 text-sm">
            <strong>Error:</strong> {error}
          </p>
        </div>
      )}
    </div>
  );
};
