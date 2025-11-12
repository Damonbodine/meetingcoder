import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { useSettings } from "../../hooks/useSettings";

type Props = {
  meetingId: string | null;
  meetingName: string;
};

export const MeetingChecklist: React.FC<Props> = ({ meetingId, meetingName }) => {
  const { getSetting } = useSettings();
  const githubEnabled = getSetting("github_enabled") ?? false;
  const repoOwner = getSetting("github_repo_owner") as string | undefined;
  const repoName = getSetting("github_repo_name") as string | undefined;

  const [loading, setLoading] = useState(false);

  const isGitHubFullyConfigured = githubEnabled && repoOwner && repoName;

  return (
    <div className="p-3 border border-mid-gray/30 rounded-md bg-black/10">
      <div className="flex items-center justify-between mb-3">
        <h3 className="text-sm font-semibold">Meeting Tools</h3>
        {meetingId && (
          <div className="flex items-center gap-2">
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
          </div>
        )}
      </div>

      <div className="flex items-start gap-2 p-2 bg-black/5 rounded border border-mid-gray/20">
        <span className={`mt-1 h-2 w-2 rounded-full ${isGitHubFullyConfigured ? "bg-green-500" : "bg-yellow-500"}`} />
        <div className="flex-1">
          <div className="text-sm font-medium">
            GitHub Integration: {isGitHubFullyConfigured ? `${repoOwner}/${repoName}` : "Not Configured"}
          </div>
          {!isGitHubFullyConfigured && (
            <div className="text-xs text-muted-foreground mt-1">
              To enable GitHub features, configure integration in the{" "}
              <span className="font-semibold">Integrations</span> tab
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
