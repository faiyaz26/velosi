import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import {
  AlertTriangle,
  Shield,
  CheckCircle,
  Settings,
  RefreshCw,
} from "lucide-react";

interface AppleEventsPermissionDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onPermissionGranted?: () => void;
}

export function AppleEventsPermissionDialog({
  isOpen,
  onClose,
  onPermissionGranted,
}: AppleEventsPermissionDialogProps) {
  const [isRequesting, setIsRequesting] = useState(false);
  const [hasRequested, setHasRequested] = useState(false);
  const [permissionStatus, setPermissionStatus] = useState<boolean | null>(
    null
  );
  const [retryAttempts, setRetryAttempts] = useState(0);
  const [isChecking, setIsChecking] = useState(false);

  const checkPermissionStatus = useCallback(async () => {
    setIsChecking(true);
    try {
      const status = await invoke<boolean>("check_apple_events_permissions");
      setPermissionStatus(status);
      if (status && onPermissionGranted) {
        onPermissionGranted();
        // Auto-close dialog when permissions are granted
        setTimeout(() => onClose(), 1000);
      }
    } catch (error) {
      console.error("Failed to check permission status:", error);
      setPermissionStatus(false);
    } finally {
      setIsChecking(false);
    }
  }, [onPermissionGranted, onClose]);

  const requestPermissions = async () => {
    setIsRequesting(true);
    setRetryAttempts((prev) => prev + 1);

    try {
      // Use the dedicated permission request command
      const granted = await invoke<boolean>(
        "trigger_apple_events_permission_request"
      );

      setPermissionStatus(granted);
      setHasRequested(true);

      if (granted && onPermissionGranted) {
        onPermissionGranted();
        // Auto-close dialog when permissions are granted
        setTimeout(() => onClose(), 1000);
      }
    } catch (error) {
      console.error("Failed to request permissions:", error);
      setHasRequested(true);
      // Even if it fails, check the status
      await checkPermissionStatus();
    } finally {
      setIsRequesting(false);
    }
  };

  const retryPermissionCheck = async () => {
    setHasRequested(false);
    setRetryAttempts(0);
    await checkPermissionStatus();
  };

  const openSystemSettings = async () => {
    try {
      await invoke("open_automation_settings");
    } catch (error) {
      console.error("Failed to open system settings:", error);
      // Fallback: try to open manually
      alert(
        "Please open System Settings > Privacy & Security > Automation manually"
      );
    }
  };

  const resetPermissions = async () => {
    try {
      await invoke("reset_apple_events_permissions");
      alert(
        "Permissions have been reset. Please try requesting permissions again."
      );
      setPermissionStatus(null);
      setHasRequested(false);
    } catch (error) {
      console.error("Failed to reset permissions:", error);
      alert(
        "Failed to reset permissions. You may need to do this manually in System Settings."
      );
    }
  };

  useEffect(() => {
    if (isOpen) {
      checkPermissionStatus();

      // Set up periodic checking for permissions while dialog is open
      const interval = setInterval(() => {
        if (permissionStatus !== true) {
          checkPermissionStatus();
        }
      }, 3000); // Check every 3 seconds

      return () => clearInterval(interval);
    }
  }, [isOpen, checkPermissionStatus, permissionStatus]);

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5 text-blue-500" />
            {permissionStatus === true
              ? "Permissions Granted!"
              : "Permission Required"}
          </DialogTitle>
          <DialogDescription>
            {permissionStatus === true
              ? "Velosi Tracker now has access to track your activities."
              : "Velosi Tracker needs permission to track browser activity and provide insights."}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          {/* Permission Status */}
          <div className="flex items-center gap-2 p-3 rounded-lg bg-muted">
            {isChecking ? (
              <>
                <RefreshCw className="h-4 w-4 text-blue-500 animate-spin" />
                <span className="text-sm">Checking permission status...</span>
              </>
            ) : permissionStatus === true ? (
              <>
                <CheckCircle className="h-4 w-4 text-green-500" />
                <span className="text-sm text-green-700">
                  All permissions granted! ✨
                </span>
              </>
            ) : (
              <>
                <AlertTriangle className="h-4 w-4 text-orange-500" />
                <span className="text-sm text-orange-700">
                  System permission needed
                </span>
              </>
            )}
          </div>

          {/* Explanation - only show if permissions not granted */}
          {permissionStatus !== true && (
            <div className="text-sm text-muted-foreground space-y-2">
              <p className="font-medium text-foreground">
                What this permission does:
              </p>
              <ul className="list-disc list-inside space-y-1 ml-2">
                <li>Tracks active browser tabs and applications</li>
                <li>Provides detailed productivity insights</li>
                <li>Enables focus mode and website blocking</li>
              </ul>
              <p className="text-xs mt-2 p-2 bg-blue-50 dark:bg-blue-950 rounded border-l-2 border-blue-200 dark:border-blue-800">
                <strong>Privacy:</strong> We only read application names and
                window titles. No browsing history or personal data is accessed.
              </p>
            </div>
          )}

          {/* Action Buttons */}
          <div className="flex flex-col gap-2">
            {permissionStatus === false && !hasRequested && (
              <Button
                onClick={requestPermissions}
                disabled={isRequesting}
                className="w-full"
                size="lg"
              >
                {isRequesting
                  ? "Opening Permission Dialog..."
                  : "Grant Permission"}
              </Button>
            )}

            {hasRequested && permissionStatus === false && (
              <div className="space-y-3">
                <div className="text-center space-y-2">
                  <p className="text-sm text-muted-foreground">
                    Permission not granted yet
                  </p>
                  <p className="text-xs text-muted-foreground">
                    If you missed the system dialog or denied it by mistake:
                  </p>
                </div>

                <div className="flex gap-2">
                  <Button
                    onClick={retryPermissionCheck}
                    variant="outline"
                    className="flex-1"
                    disabled={isChecking}
                  >
                    <RefreshCw
                      className={`h-4 w-4 mr-2 ${
                        isChecking ? "animate-spin" : ""
                      }`}
                    />
                    {retryAttempts > 0 ? "Try Again" : "Check Again"}
                  </Button>
                  <Button
                    onClick={requestPermissions}
                    disabled={isRequesting}
                    className="flex-1"
                  >
                    {isRequesting ? "Requesting..." : "Request Again"}
                  </Button>
                </div>

                <Button
                  onClick={openSystemSettings}
                  variant="outline"
                  size="sm"
                  className="w-full"
                >
                  <Settings className="h-4 w-4 mr-2" />
                  Open System Settings Manually
                </Button>

                {retryAttempts > 1 && (
                  <Button
                    onClick={resetPermissions}
                    variant="ghost"
                    size="sm"
                    className="w-full text-xs"
                  >
                    Reset Permissions (Advanced)
                  </Button>
                )}
              </div>
            )}

            {permissionStatus === true && (
              <Button onClick={onClose} className="w-full" size="lg">
                Continue to App
              </Button>
            )}
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
