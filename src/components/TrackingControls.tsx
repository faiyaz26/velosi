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
import { AppleEventsPermissionDialog } from "./AppleEventsPermissionDialog";

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
  const [hasPermissions, setHasPermissions] = useState<boolean | null>(null);
  const [showPermissionDialog, setShowPermissionDialog] = useState(false);

  // Check permissions on component mount
  useEffect(() => {
    const checkPermissions = async () => {
      try {
        const status = await invoke<boolean>("check_apple_events_permissions");
        setHasPermissions(status);
      } catch (error) {
        console.error("Failed to check permissions:", error);
        setHasPermissions(false);
      }
    };

    checkPermissions();
  }, []);

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
    // Check permissions first
    try {
      const status = await invoke<boolean>("check_apple_events_permissions");
      if (!status) {
        setShowPermissionDialog(true);
        return;
      }

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

  const handlePermissionGranted = () => {
    setHasPermissions(true);
    setShowPermissionDialog(false);
    // Automatically start tracking once permissions are granted
    handleStartTracking();
  };

  const testChromeAccess = async () => {
    try {
      const result = await invoke<string>("test_chrome_access");
      alert(`Chrome Access Test Result:\n${result}`);
    } catch (error) {
      console.error("Chrome test failed:", error);
      alert(`Chrome test failed: ${error}`);
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
        {hasPermissions === false && !isTracking && (
          <div className="flex items-center gap-2 p-3 rounded-lg bg-orange-50 dark:bg-orange-950 border border-orange-200 dark:border-orange-800 text-orange-800 dark:text-orange-200">
            <Shield className="h-4 w-4 text-orange-600" />
            <span className="text-sm">
              System permission required to track activities
            </span>
          </div>
        )}

        <div className="flex gap-2">
          <Button
            onClick={handleStartTracking}
            disabled={isTracking}
            className="flex items-center gap-2"
          >
            <Play className="h-4 w-4" />
            {hasPermissions === false
              ? "Grant Permission & Start"
              : "Start Tracking"}
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

        {/* Debug test button for Chrome access */}
        <div className="pt-2 border-t">
          <Button
            onClick={testChromeAccess}
            variant="outline"
            size="sm"
            className="w-full text-xs"
          >
            🧪 Test Chrome Access (Debug)
          </Button>
        </div>
      </CardContent>

      <AppleEventsPermissionDialog
        isOpen={showPermissionDialog}
        onClose={() => setShowPermissionDialog(false)}
        onPermissionGranted={handlePermissionGranted}
      />
    </Card>
  );
}
