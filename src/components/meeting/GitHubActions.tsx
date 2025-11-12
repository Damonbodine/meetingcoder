import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  GitHubRepoStatus,
  PushResult,
  PRResult,
} from "../../lib/types";
import { useSettings } from "../../hooks/useSettings";
import { toast } from "sonner";

export const GitHubActions: React.FC<{
  meetingId: string;
  meetingName: string;
}> = ({ meetingId, meetingName }) => {
  const { getSetting } = useSettings();
  const githubEnabled = getSetting("github_enabled") ?? false;
  const [status, setStatus] = useState<GitHubRepoStatus | null>(null);
  const [isPushing, setIsPushing] = useState(false);
  const [isCreatingPR, setIsCreatingPR] = useState(false);
  const [isPostingComment, setIsPostingComment] = useState(false);

  useEffect(() => {
    if (!githubEnabled) return;
    loadStatus();
  }, [githubEnabled, meetingId]);

  const loadStatus = async () => {
    try {
      const result = await invoke<GitHubRepoStatus>(
        "get_github_repo_status",
        { meetingId }
      );
      setStatus(result);
    } catch (error) {
      console.error("Failed to load GitHub status:", error);
    }
  };

  const handlePush = async () => {
    setIsPushing(true);
    try {
      const result = await invoke<PushResult>("push_meeting_changes", {
        meetingId,
        commitMessage: null,
      });

      if (result.success) {
        toast.success("Pushed to GitHub", {
          description: `Branch: ${result.branch}`,
        });
        await loadStatus();
      } else {
        toast.error("Push failed", {
          description: result.error || "Unknown error",
        });
      }
    } catch (error) {
      console.error("Failed to push:", error);
      toast.error("Push failed", {
        description: String(error),
      });
    } finally {
      setIsPushing(false);
    }
  };

  const handleCreateOrUpdatePR = async () => {
    setIsCreatingPR(true);
    try {
      const result = await invoke<PRResult>("create_or_update_pr", {
        meetingId,
        title: null,
        body: null,
      });

      if (result.success && result.pr_url) {
        toast.success("PR created/updated", {
          description: `PR #${result.pr_number}`,
          action: {
            label: "Open",
            onClick: () => window.open(result.pr_url!, "_blank"),
          },
        });
        await loadStatus();
      } else {
        toast.error("PR creation failed", {
          description: result.error || "Unknown error",
        });
      }
    } catch (error) {
      console.error("Failed to create PR:", error);
      toast.error("PR creation failed", {
        description: String(error),
      });
    } finally {
      setIsCreatingPR(false);
    }
  };

  const handlePostComment = async () => {
    setIsPostingComment(true);
    try {
      await invoke("post_meeting_update_comment", {
        meetingId,
        comment: null,
      });

      toast.success("Comment posted to PR");
    } catch (error) {
      console.error("Failed to post comment:", error);
      toast.error("Failed to post comment", {
        description: String(error),
      });
    } finally {
      setIsPostingComment(false);
    }
  };

  if (!githubEnabled) {
    return (
      <div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
        <p className="text-sm text-gray-600">
          GitHub integration is disabled. Enable it in Settings to push changes
          and create PRs.
        </p>
      </div>
    );
  }

  if (!status) {
    return (
      <div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
        <p className="text-sm text-gray-600">Loading GitHub status...</p>
      </div>
    );
  }

  const isConfigured =
    status.has_token && status.repo_owner && status.repo_name;

  if (!isConfigured) {
    return (
      <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
        <p className="text-sm text-yellow-800">
          GitHub integration is not fully configured. Please set up your token
          and repository in Settings.
        </p>
      </div>
    );
  }

  return (
    <div className="bg-white border border-gray-200 rounded-lg p-4 space-y-4">
      <div className="flex items-center justify-between">
        <h4 className="font-medium text-sm">GitHub Integration</h4>
        <a
          href={`https://github.com/${status.repo_owner}/${status.repo_name}`}
          target="_blank"
          rel="noopener noreferrer"
          className="text-xs text-blue-600 hover:underline"
        >
          {status.repo_owner}/{status.repo_name}
        </a>
      </div>

      {/* Status Information */}
      <div className="space-y-2 text-sm">
        {status.current_branch && (
          <div className="flex items-center gap-2">
            <span className="text-gray-600">Branch:</span>
            <code className="bg-gray-100 px-2 py-0.5 rounded text-xs">
              {status.current_branch}
            </code>
          </div>
        )}

        {status.last_pr_url && (
          <div className="flex items-center gap-2">
            <span className="text-gray-600">PR:</span>
            <a
              href={status.last_pr_url}
              target="_blank"
              rel="noopener noreferrer"
              className="text-blue-600 hover:underline text-xs"
            >
              #{status.last_pr_number}
            </a>
          </div>
        )}

        {status.last_push_time && (
          <div className="flex items-center gap-2">
            <span className="text-gray-600">Last push:</span>
            <span className="text-xs text-gray-500">
              {new Date(status.last_push_time).toLocaleString()}
            </span>
          </div>
        )}
      </div>

      {/* Action Buttons */}
      <div className="flex gap-2 flex-wrap">
        <button
          onClick={handlePush}
          disabled={isPushing}
          className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed text-sm"
        >
          {isPushing ? "Pushing..." : "Push Changes"}
        </button>

        <button
          onClick={handleCreateOrUpdatePR}
          disabled={isCreatingPR}
          className="px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 disabled:bg-gray-300 disabled:cursor-not-allowed text-sm"
        >
          {isCreatingPR
            ? "Creating..."
            : status.last_pr_number
            ? "Update PR"
            : "Create PR"}
        </button>

        {status.last_pr_number && (
          <button
            onClick={handlePostComment}
            disabled={isPostingComment}
            className="px-4 py-2 bg-purple-600 text-white rounded-md hover:bg-purple-700 disabled:bg-gray-300 disabled:cursor-not-allowed text-sm"
          >
            {isPostingComment ? "Posting..." : "Post Update"}
          </button>
        )}
      </div>
    </div>
  );
};
