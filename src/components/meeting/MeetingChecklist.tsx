import React, { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { useSettings } from "../../hooks/useSettings";
import { GitHubRepoStatus } from "../../lib/types";
import { GitHubRepoPicker } from "../settings/GitHubRepoPicker";
import { GitHubEnabled } from "../settings/GitHubEnabled";
import { GitHubToken } from "../settings/GitHubToken";

type Props = {
  meetingId: string | null;
  meetingName: string;
};

export const MeetingChecklist: React.FC<Props> = ({ meetingId, meetingName }) => {
  const { getSetting } = useSettings();
  const githubEnabled = getSetting("github_enabled") ?? false;
  const repoOwner = getSetting("github_repo_owner") as string | undefined;
  const repoName = getSetting("github_repo_name") as string | undefined;
  const defaultBranch = (getSetting("github_default_branch") as string) || "main";
  const branchPattern = (getSetting("github_branch_pattern") as string) || "meeting/{meeting_id}";

  const [status, setStatus] = useState<GitHubRepoStatus | null>(null);
  const [loading, setLoading] = useState(false);
  const [showRepoPicker, setShowRepoPicker] = useState(false);

  const expectedBranch = useMemo(() => {
    if (!meetingId) return branchPattern.replace("{meeting_id}", "<meeting_id>").replace("{meeting_name}", meetingName.toLowerCase().replace(/\s+/g, "-"));
    const safeName = meetingName.toLowerCase().replace(/[^a-z0-9-]+/g, "-");
    return branchPattern.replace("{meeting_id}", meetingId).replace("{meeting_name}", safeName);
  }, [branchPattern, meetingId, meetingName]);

  const refresh = async () => {
    if (!meetingId) {
      setStatus(null);
      return;
    }
    try {
      setLoading(true);
      const s = await invoke<GitHubRepoStatus>("get_github_repo_status", { meetingId });
      setStatus(s);
    } catch (e) {
      console.error("Failed to get repo status:", e);
      toast.error("Failed to load GitHub status");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    refresh();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [meetingId]);

  const testConnection = async () => {
    try {
      const res = await invoke<{ success: boolean; username?: string; error?: string }>("test_github_connection");
      if (res.success) {
        toast.success("GitHub connected", { description: `@${res.username}` });
      } else {
        toast.error("GitHub test failed", { description: res.error || "Unknown error" });
      }
    } catch (e) {
      toast.error("GitHub test failed", { description: String(e) });
    }
  };

  const missingRepo = !repoOwner || !repoName;
  const hasToken = status?.has_token ?? false;

  return (
    <div className="p-3 border border-mid-gray/30 rounded-md bg-black/10">
      <div className="flex items-center justify-between mb-2">
        <h3 className="text-sm font-semibold">Pre-flight Checklist</h3>
        <div className="flex items-center gap-2">
          {meetingId && (
            <>
              <button
                onClick={() => invoke("open_meeting_vscode_with_meeting", { meetingId }).catch((e) => toast.error(String(e)))}
                className="px-2 py-1 text-xs rounded bg-blue-600 text-white hover:bg-blue-700"
              >
                Open VS Code + Claude
              </button>
              <button
                onClick={() => invoke("open_meeting_cursor_with_meeting", { meetingId }).catch((e) => toast.error(String(e)))}
                className="px-2 py-1 text-xs rounded bg-indigo-600 text-white hover:bg-indigo-700"
              >
                Open Cursor + Claude
              </button>
              <button
                onClick={() => invoke("open_meeting_terminal", { meetingId }).catch((e) => toast.error(String(e)))}
                className="px-2 py-1 text-xs rounded bg-gray-700 text-white hover:bg-gray-800"
              >
                Open Terminal
              </button>
            </>
          )}
          <button
            onClick={refresh}
            disabled={loading}
            className="px-2 py-1 text-xs rounded bg-gray-200 text-black hover:bg-gray-300 disabled:opacity-50"
          >
            {loading ? "Refreshing..." : "Refresh"}
          </button>
        </div>
      </div>

      <ul className="space-y-2">
        <li className="flex items-start gap-2">
          <span className={`mt-1 h-2 w-2 rounded-full ${githubEnabled ? "bg-green-500" : "bg-red-500"}`} />
          <div className="flex-1">
            <div className="text-sm font-medium">GitHub integration {githubEnabled ? "enabled" : "disabled"}</div>
            {!githubEnabled && (
              <div className="mt-2"><GitHubEnabled grouped descriptionMode="tooltip" /></div>
            )}
          </div>
        </li>

        <li className="flex items-start gap-2">
          <span className={`mt-1 h-2 w-2 rounded-full ${hasToken ? "bg-green-500" : "bg-red-500"}`} />
          <div className="flex-1">
            <div className="text-sm font-medium">GitHub token {hasToken ? "detected" : "not set"}</div>
            <div className="mt-2 flex items-center gap-2">
              <button onClick={testConnection} className="px-2 py-1 text-xs rounded bg-gray-200 hover:bg-gray-300">Test Connection</button>
            </div>
            {!hasToken && (
              <div className="mt-2"><GitHubToken grouped descriptionMode="tooltip" /></div>
            )}
          </div>
        </li>

        <li className="flex items-start gap-2">
          <span className={`mt-1 h-2 w-2 rounded-full ${!missingRepo ? "bg-green-500" : "bg-red-500"}`} />
          <div className="flex-1">
            <div className="text-sm font-medium">Repository {missingRepo ? "not selected" : `${repoOwner}/${repoName}`}</div>
            {missingRepo ? (
              <div className="mt-2">
                <button
                  onClick={() => setShowRepoPicker(true)}
                  className="px-3 py-1 text-xs rounded bg-blue-600 text-white hover:bg-blue-700"
                >
                  Select Repository
                </button>
              </div>
            ) : (
              <div className="mt-1 text-xs opacity-80">Default branch: {defaultBranch}</div>
            )}
          </div>
        </li>

        {repoOwner && repoName && (
          <li className="flex items-start gap-2">
            <span className="mt-1 h-2 w-2 rounded-full bg-blue-500" />
            <div className="flex-1">
              <div className="text-sm font-medium">Target branch</div>
              <div className="text-xs opacity-80 font-mono">{expectedBranch}</div>
              {meetingId && status?.current_branch && (
                <div className="text-xs opacity-70 mt-1">Current local branch: {status.current_branch}</div>
              )}
            </div>
          </li>
        )}
      </ul>

      {showRepoPicker && (
        <div className="mt-3">
          <GitHubRepoPicker grouped descriptionMode="tooltip" />
        </div>
      )}
    </div>
  );
};
