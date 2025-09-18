import React, { useState, useEffect } from "react";
import { Button } from "./ui/button";
import { Card, CardContent, CardHeader } from "./ui/card";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "./ui/dialog";
import { Input } from "./ui/input";
import { Label } from "./ui/label";
import { Textarea } from "./ui/textarea";
import { Badge } from "./ui/badge";
import { Switch } from "./ui/switch";
import { Play, Pause, Square, Settings } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { usePomodoro } from "@/contexts/PomodoroContext";

export interface PomodoroSession {
  id: string;
  session_type: "work" | "break";
  start_time: string;
  end_time?: string;
  duration_minutes: number;
  actual_duration_seconds?: number;
  work_description?: string;
  completed: boolean;
  focus_mode_enabled: boolean;
  app_tracking_enabled: boolean;
}

export interface PomodoroSettings {
  id: string;
  work_duration_minutes: number;
  break_duration_minutes: number;
  enable_focus_mode: boolean;
  enable_app_tracking: boolean;
  auto_start_breaks: boolean;
  auto_start_work: boolean;
  updated_at: string;
}

interface PomodoroTimerProps {
  onSessionComplete?: (session: PomodoroSession) => void;
}

export const PomodoroTimer: React.FC<PomodoroTimerProps> = ({
  onSessionComplete,
}) => {
  const {
    timeLeft,
    timerState,
    sessionType,
    currentSession,
    workDescription,
    settings,
    completedSessionType,
    setCompletedSessionType,
    setTimeLeft,
    setTimerState,
    setCurrentSession,
    setSessionType,
    setWorkDescription,
    setSettings,
    startTimer,
    pauseTimer,
    resumeTimer,
    stopTimer,
    loadSettings,
    setOnSessionComplete,
  } = usePomodoro();

  const [showSettingsDialog, setShowSettingsDialog] = useState(false);
  const [draftSettings, setDraftSettings] = useState<PomodoroSettings | null>(
    null
  );
  const [showConfirmDialog, setShowConfirmDialog] = useState(false);
  const [confirmAction, setConfirmAction] = useState<
    "reset" | "restart" | null
  >(null);
  const [showCompletionDialog, setShowCompletionDialog] = useState(false);

  // Set up the session completion callback
  useEffect(() => {
    if (onSessionComplete) {
      setOnSessionComplete(onSessionComplete);
    }
  }, [onSessionComplete, setOnSessionComplete]);

  // Load settings on component mount
  useEffect(() => {
    loadSettings();
  }, [loadSettings]);

  // Initialize/clear draft settings only when the dialog opens or closes
  useEffect(() => {
    if (showSettingsDialog) {
      setDraftSettings(settings ?? null);
    } else {
      setDraftSettings(null);
    }
  }, [showSettingsDialog]);

  // Helper to get a non-null draft object (falls back to settings)
  const currentDraft = (): PomodoroSettings => {
    return draftSettings ?? settings!; // settings is guaranteed to be non-null here
  };

  // Show completion dialog when a session completes
  const prevCompletedRef = React.useRef<"work" | "break" | null>(null);

  useEffect(() => {
    const prev = prevCompletedRef.current;
    // only open when completedSessionType changed from null to a value
    if (
      completedSessionType !== null &&
      prev !== completedSessionType &&
      timerState === "stopped" &&
      currentSession === null
    ) {
      setShowCompletionDialog(true);
    }
    prevCompletedRef.current = completedSessionType;
  }, [completedSessionType, timerState, currentSession]);

  const formatTime = (seconds: number): string => {
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;
    return `${minutes
      .toString()
      .padStart(2, "0")}:${remainingSeconds.toString().padStart(2, "0")}`;
  };

  const handleSessionTypeToggle = () => {
    if (timerState === "stopped") {
      setSessionType(sessionType === "work" ? "break" : "work");
    }
  };

  const saveSettings = async (newSettings: PomodoroSettings) => {
    try {
      await invoke("update_pomodoro_settings", { settings: newSettings });
      setSettings(newSettings);
      setShowSettingsDialog(false);
      setDraftSettings(null);
    } catch (error) {
      console.error("Failed to save settings:", error);
    }
  };

  const getSessionTypeColor = (type: "work" | "break") => {
    return type === "work" ? "bg-blue-500" : "bg-green-500";
  };

  const getSessionTypeLabel = (type: "work" | "break") => {
    return type === "work" ? "Work" : "Break";
  };

  if (!settings) {
    return (
      <Card className="w-full max-w-md mx-auto">
        <CardContent className="p-6">
          <div className="text-center">Loading...</div>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="space-y-4">
      <Card className="w-full max-w-md mx-auto">
        <CardHeader className="text-center">
          <div className="flex items-center justify-center gap-2">
            <Badge
              className={`${getSessionTypeColor(sessionType)} text-white`}
              onClick={handleSessionTypeToggle}
              style={{
                cursor: timerState === "stopped" ? "pointer" : "default",
              }}
            >
              {getSessionTypeLabel(sessionType)}
            </Badge>
            <div className="flex-1" />
            <Button
              variant="outline"
              size="sm"
              onClick={() => setShowSettingsDialog(true)}
              className="ml-auto"
            >
              <Settings className="h-4 w-4" />
            </Button>
          </div>
        </CardHeader>
        <CardContent className="text-center space-y-4">
          <div className="text-6xl font-mono font-bold">
            {formatTime(timeLeft)}
          </div>

          {sessionType === "work" && (
            <div className="space-y-2">
              <Label htmlFor="work-description">
                Work Description (optional)
              </Label>
              <Textarea
                id="work-description"
                placeholder="What are you working on?"
                value={workDescription}
                onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) =>
                  setWorkDescription(e.target.value)
                }
                disabled={timerState === "running"}
                className="resize-none"
                rows={2}
              />
            </div>
          )}

          <div className="flex justify-center gap-2">
            {timerState === "stopped" && (
              <Button onClick={startTimer} className="flex items-center gap-2">
                <Play className="h-4 w-4" />
                Start
              </Button>
            )}
            {timerState === "running" && (
              <Button
                onClick={pauseTimer}
                variant="outline"
                className="flex items-center gap-2"
              >
                <Pause className="h-4 w-4" />
                Pause
              </Button>
            )}
            {timerState === "paused" && (
              <Button onClick={resumeTimer} className="flex items-center gap-2">
                <Play className="h-4 w-4" />
                Resume
              </Button>
            )}
            {(timerState === "running" || timerState === "paused") && (
              <Button
                onClick={stopTimer}
                variant="destructive"
                className="flex items-center gap-2"
              >
                <Square className="h-4 w-4" />
                Stop
              </Button>
            )}
            {/* Reset current session timer to configured duration */}
            <Button
              variant="ghost"
              onClick={() => {
                setConfirmAction("reset");
                setShowConfirmDialog(true);
              }}
            >
              Reset
            </Button>

            {/* Restart as a new work session */}
            <Button
              variant="outline"
              onClick={() => {
                setConfirmAction("restart");
                setShowConfirmDialog(true);
              }}
            >
              Restart Work
            </Button>
          </div>

          {currentSession && (
            <div className="text-sm text-muted-foreground">
              Session started at{" "}
              {new Date(currentSession.start_time).toLocaleTimeString()}
            </div>
          )}
        </CardContent>
      </Card>

      <Dialog open={showSettingsDialog} onOpenChange={setShowSettingsDialog}>
        <DialogContent className="sm:max-w-[425px]">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <Settings className="h-5 w-5" />
              Pomodoro Settings
            </DialogTitle>
          </DialogHeader>
          <div className="space-y-6 py-4 px-4">
            <div className="space-y-4">
              <div>
                <Label htmlFor="work-duration" className="text-sm font-medium">
                  Work Duration (minutes)
                </Label>
                <div className="flex items-center gap-2 mt-1">
                  <Input
                    id="work-duration"
                    type="number"
                    value={
                      draftSettings?.work_duration_minutes ??
                      settings.work_duration_minutes
                    }
                    onChange={(e) => {
                      setDraftSettings({
                        ...currentDraft(),
                        work_duration_minutes: parseInt(e.target.value) || 25,
                      });
                    }}
                    min="1"
                    max="120"
                  />
                </div>
              </div>
              <div>
                <Label htmlFor="break-duration" className="text-sm font-medium">
                  Break Duration (minutes)
                </Label>
                <div className="flex items-center gap-2 mt-1">
                  <Input
                    id="break-duration"
                    type="number"
                    value={
                      draftSettings?.break_duration_minutes ??
                      settings.break_duration_minutes
                    }
                    onChange={(e) => {
                      setDraftSettings({
                        ...currentDraft(),
                        break_duration_minutes: parseInt(e.target.value) || 5,
                      });
                    }}
                    min="1"
                    max="60"
                  />
                </div>
              </div>
            </div>

            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label htmlFor="focus-mode" className="text-sm font-medium">
                    Focus Mode
                  </Label>
                  <p className="text-xs text-muted-foreground">
                    Block distracting websites during work sessions
                  </p>
                </div>
                <Switch
                  id="focus-mode"
                  checked={Boolean(draftSettings?.enable_focus_mode)}
                  onCheckedChange={(checked) => {
                    const value = checked === true;
                    const updatedDraft = {
                      ...currentDraft(),
                      enable_focus_mode: value,
                    };
                    setDraftSettings(updatedDraft);
                  }}
                />
              </div>
              <div className="flex items-center justify-between">
                <div className="space-y-0.5">
                  <Label htmlFor="app-tracking" className="text-sm font-medium">
                    App Tracking
                  </Label>
                  <p className="text-xs text-muted-foreground">
                    Monitor app usage during focus sessions
                  </p>
                </div>
                <Switch
                  id="app-tracking"
                  checked={Boolean(
                    draftSettings?.enable_app_tracking ??
                      settings.enable_app_tracking
                  )}
                  onCheckedChange={(checked) => {
                    const value = checked === true;
                    setDraftSettings({
                      ...currentDraft(),
                      enable_app_tracking: value,
                    });
                  }}
                />
              </div>
            </div>
          </div>
          <DialogFooter className="flex justify-end gap-2">
            <Button
              variant="ghost"
              onClick={() => setShowSettingsDialog(false)}
            >
              Cancel
            </Button>

            {/* <Button
              variant="outline"
              onClick={async () => {
                try {
                  await sendNotification({
                    title: "Test Notification",
                    body: "This is a test notification from Velosi",
                  });
                  console.log("Test notification sent");
                } catch (error) {
                  console.error("Test notification failed:", error);
                }
              }}
            >
              Test Notification
            </Button> */}

            <Button
              onClick={() => {
                if (!draftSettings) return;
                console.log("Saving settings to DB:", draftSettings);
                saveSettings(draftSettings);
              }}
            >
              Save Settings
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Confirmation dialog for reset/restart actions */}
      <Dialog open={showConfirmDialog} onOpenChange={setShowConfirmDialog}>
        <DialogContent className="sm:max-w-[400px]">
          <DialogHeader>
            <DialogTitle>
              {confirmAction === "reset" ? "Reset Timer" : "Restart Work"}
            </DialogTitle>
          </DialogHeader>
          <div className="py-2 px-4">
            <p>
              {confirmAction === "reset"
                ? "Are you sure you want to reset the current timer to its configured duration?"
                : "Are you sure you want to restart from a Work session? This will stop the current session."}
            </p>
          </div>
          <DialogFooter className="flex justify-end gap-2">
            <Button
              variant="ghost"
              onClick={() => {
                setShowConfirmDialog(false);
                setConfirmAction(null);
              }}
            >
              Cancel
            </Button>
            <Button
              onClick={() => {
                // perform the confirmed action
                if (confirmAction === "reset") {
                  if (!settings) return;
                  const duration =
                    sessionType === "work"
                      ? settings.work_duration_minutes
                      : settings.break_duration_minutes;
                  console.log("Confirmed reset to", duration, "minutes");
                  setTimeLeft(duration * 60);
                } else if (confirmAction === "restart") {
                  if (!settings) return;
                  const duration = settings.work_duration_minutes;
                  console.log(
                    "Confirmed restart as work session with",
                    duration
                  );
                  setSessionType("work");
                  setTimeLeft(duration * 60);
                  setTimerState("stopped");
                  setCurrentSession(null);
                }

                setShowConfirmDialog(false);
                setConfirmAction(null);
              }}
            >
              Confirm
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog
        open={showCompletionDialog}
        onOpenChange={setShowCompletionDialog}
      >
        <DialogContent showCloseButton={false}>
          <DialogHeader>
            <DialogTitle>
              {completedSessionType
                ? getSessionTypeLabel(completedSessionType)
                : "Session"}{" "}
              Session Complete!
            </DialogTitle>
          </DialogHeader>
          <div className="text-left px-6 py-4">
            <p>
              {completedSessionType === "work"
                ? "Great work! Time for a break."
                : "Break time's over. Ready to get back to work?"}
            </p>
          </div>
          <DialogFooter>
            <Button
              onClick={() => {
                setShowCompletionDialog(false);
                setCompletedSessionType(null);
              }}
            >
              OK
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
};
