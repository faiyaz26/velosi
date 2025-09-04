import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useState, useEffect } from "react";
import { Play, Pause, Activity, Shield } from "lucide-react";

interface CurrentActivity {
  app_name: string;
  app_bundle_id?: string;
  window_title: string;
  url?: string;
  timestamp: string;
}

export function TrackingControls() {
  const [isTracking, setIsTracking] = useState(false);
  const [
    currentActivity,
    setCurrentActivity,
  ] = useState<CurrentActivity | null>(null);
  // Permission status is surfaced via alert for now; no need to store in state
  const [testingPermissions, setTestingPermissions] = useState(false);

  useEffect(() => {
    // Check initial tracking status
    invoke<boolean>("get_tracking_status").then(setIsTracking);

    // Listen for tracking status changes from tray or other sources
    const unlistenPromise = listen<boolean>(
      "tracking-status-changed",
      (event) => {
        setIsTracking(event.payload);
      }
    );

    // Set up interval to get current activity
    const interval = setInterval(async () => {
      if (isTracking) {
        try {
          const activity = await invoke<CurrentActivity | null>(
            "get_current_activity"
          );
          setCurrentActivity(activity);
        } catch (error) {
          console.error("Failed to get current activity:", error);
        }
      }
    }, 2000);

    return () => {
      clearInterval(interval);
      // Clean up event listener
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, [isTracking]);

  const handleStartTracking = async () => {
    try {
      await invoke("start_tracking");
      setIsTracking(true);
    } catch (error) {
      console.error("Failed to start tracking:", error);
    }
  };

  const handleStopTracking = async () => {
    try {
      await invoke("stop_tracking");
      setIsTracking(false);
      setCurrentActivity(null);
    } catch (error) {
      console.error("Failed to stop tracking:", error);
    }
  };

  const handleTestPermissions = async () => {
    setTestingPermissions(true);
    try {
      const result = await invoke<string>("test_permissions");
      alert(result); // Show result in alert for now
    } catch (error) {
      const errorMsg = `Failed to test permissions: ${error}`;
      alert(errorMsg);
    } finally {
      setTestingPermissions(false);
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Activity className="h-5 w-5" />
          Activity Tracking
        </CardTitle>
        <CardDescription>
          Monitor your application usage and productivity
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex gap-2">
          <Button
            onClick={handleStartTracking}
            disabled={isTracking}
            className="flex items-center gap-2"
          >
            <Play className="h-4 w-4" />
            Start Tracking
          </Button>
          <Button
            onClick={handleStopTracking}
            disabled={!isTracking}
            variant="outline"
            className="flex items-center gap-2"
          >
            <Pause className="h-4 w-4" />
            Stop Tracking
          </Button>
          <Button
            onClick={handleTestPermissions}
            disabled={testingPermissions}
            variant="secondary"
            size="sm"
            className="flex items-center gap-2"
          >
            <Shield className="h-4 w-4" />
            {testingPermissions ? "Testing..." : "Test Permissions"}
          </Button>
        </div>

        <div className="flex items-center gap-2">
          <div
            className={`h-3 w-3 rounded-full ${
              isTracking ? "bg-green-500" : "bg-gray-400"
            }`}
          />
          <span className="text-sm text-muted-foreground">
            {isTracking ? "Tracking active" : "Tracking stopped"}
          </span>
        </div>

        {currentActivity && (
          <div className="border rounded-lg p-4 bg-muted/50">
            <h4 className="font-medium mb-2">Current Activity</h4>
            <div className="space-y-1 text-sm">
              <p>
                <span className="font-medium">App:</span>{" "}
                {currentActivity.app_name}
              </p>
              <p>
                <span className="font-medium">Window:</span>{" "}
                {currentActivity.window_title}
              </p>
              {currentActivity.url && (
                <p>
                  <span className="font-medium">URL:</span>{" "}
                  {currentActivity.url}
                </p>
              )}
              <p className="text-muted-foreground">
                Started:{" "}
                {new Date(currentActivity.timestamp).toLocaleTimeString()}
              </p>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
