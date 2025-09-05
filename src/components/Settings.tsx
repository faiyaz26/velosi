import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  User,
  Bell,
  Database,
  Palette,
  Shield,
  CheckCircle,
  XCircle,
  RefreshCw,
} from "lucide-react";

export function Settings() {
  const [hasPermissions, setHasPermissions] = useState<boolean | null>(null);
  const [checkingPermissions, setCheckingPermissions] = useState(false);

  const checkPermissionStatus = async () => {
    setCheckingPermissions(true);
    try {
      const status = await invoke<boolean>("get_permission_status");
      setHasPermissions(status);
    } catch (error) {
      console.error("Failed to check permissions:", error);
      setHasPermissions(false);
    } finally {
      setCheckingPermissions(false);
    }
  };

  useEffect(() => {
    checkPermissionStatus();
  }, []);

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-semibold tracking-tight">Settings</h1>
        <p className="text-muted-foreground">
          Configure your Velosi Tracker preferences
        </p>
      </div>

      <div className="grid gap-6">
        {/* Permissions */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Shield className="h-5 w-5" />
              Permissions
            </CardTitle>
            <CardDescription>
              Manage application permissions and accessibility settings
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            {/* Permission Status Display */}
            <div className="flex items-center justify-between p-4 border rounded-lg">
              <div className="flex items-center gap-3">
                {checkingPermissions ? (
                  <RefreshCw className="h-5 w-5 animate-spin text-blue-500" />
                ) : hasPermissions === true ? (
                  <CheckCircle className="h-5 w-5 text-green-500" />
                ) : hasPermissions === false ? (
                  <XCircle className="h-5 w-5 text-red-500" />
                ) : (
                  <Shield className="h-5 w-5 text-gray-500" />
                )}
                <div>
                  <p className="font-medium">
                    {checkingPermissions
                      ? "Checking permissions..."
                      : hasPermissions === true
                      ? "Accessibility Permissions Granted"
                      : hasPermissions === false
                      ? "Accessibility Permissions Required"
                      : "Permission Status Unknown"}
                  </p>
                  <p className="text-sm text-muted-foreground">
                    {hasPermissions === true
                      ? "The app can track your activities"
                      : hasPermissions === false
                      ? "Please grant accessibility permissions to enable tracking"
                      : "Click refresh to check current status"}
                  </p>
                </div>
              </div>
              <Button
                onClick={checkPermissionStatus}
                disabled={checkingPermissions}
                variant="outline"
                size="sm"
                className="flex items-center gap-2"
              >
                <RefreshCw
                  className={`h-4 w-4 ${
                    checkingPermissions ? "animate-spin" : ""
                  }`}
                />
                Refresh
              </Button>
            </div>

            {/* Instructions for granting permissions */}
            {hasPermissions === false && (
              <div className="p-4 border rounded-lg bg-red-50 dark:bg-red-950">
                <p className="text-sm text-red-800 dark:text-red-200">
                  <strong>🚨 Action Required:</strong> This app requires
                  accessibility permissions to track application usage.
                </p>
                <div className="mt-3 text-sm text-red-700 dark:text-red-300">
                  <p className="font-medium mb-2">
                    📋 Step-by-step instructions:
                  </p>
                  <ol className="list-decimal list-inside space-y-2">
                    <li>
                      <strong>Open System Settings:</strong>
                      <br />
                      <span className="text-xs">
                        Click the Apple menu → System Settings (or System
                        Preferences on older macOS)
                      </span>
                    </li>
                    <li>
                      <strong>Navigate to Privacy & Security:</strong>
                      <br />
                      <span className="text-xs">
                        Click "Privacy & Security" in the sidebar
                      </span>
                    </li>
                    <li>
                      <strong>Find Accessibility section:</strong>
                      <br />
                      <span className="text-xs">
                        Scroll down and click "Accessibility" on the right
                      </span>
                    </li>
                    <li>
                      <strong>Add this application:</strong>
                      <br />
                      <span className="text-xs">
                        Click the "+" button and add:
                        <br />
                        • If running from Terminal: Add "Terminal"
                        <br />
                        • If running from VS Code: Add "Visual Studio Code"
                        <br />• If running the built app: Add "Velosi Tracker"
                      </span>
                    </li>
                    <li>
                      <strong>Enable the toggle:</strong>
                      <br />
                      <span className="text-xs">
                        Make sure the checkbox next to the app is checked
                      </span>
                    </li>
                    <li>
                      <strong>Restart the application:</strong>
                      <br />
                      <span className="text-xs">
                        Close and restart Velosi Tracker for changes to take
                        effect
                      </span>
                    </li>
                  </ol>
                  <div className="mt-3 p-2 bg-red-100 dark:bg-red-900 rounded text-xs">
                    <strong>💡 Tip:</strong> After granting permissions, click
                    the "Refresh" button above to verify the status.
                  </div>
                </div>
              </div>
            )}

            {hasPermissions === true && (
              <div className="p-4 border rounded-lg bg-green-50 dark:bg-green-950">
                <p className="text-sm text-green-800 dark:text-green-200">
                  <strong>✅ All set!</strong> The app has the necessary
                  permissions to track your activities.
                </p>
                <p className="text-xs text-green-700 dark:text-green-300 mt-1">
                  Accessibility permissions allow the app to monitor which
                  applications you're using and track your productivity.
                </p>
              </div>
            )}

            {/* General information about accessibility permissions */}
            <div className="p-4 border rounded-lg bg-blue-50 dark:bg-blue-950">
              <p className="text-sm text-blue-800 dark:text-blue-200">
                <strong>ℹ️ About Accessibility Permissions:</strong>
              </p>
              <p className="text-xs text-blue-700 dark:text-blue-300 mt-1">
                This app uses accessibility APIs to monitor active applications
                and track your productivity. Your data stays completely local on
                your device and is never sent anywhere. You can revoke these
                permissions at any time in System Settings.
              </p>
            </div>
          </CardContent>
        </Card>

        {/* General Settings */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <User className="h-5 w-5" />
              General
            </CardTitle>
            <CardDescription>
              Basic application settings and preferences
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <div className="text-sm font-medium">Username</div>
              <Input
                id="username"
                placeholder="Enter your username"
                defaultValue="User"
              />
            </div>
            <div className="space-y-2">
              <div className="text-sm font-medium">
                Tracking Interval (seconds)
              </div>
              <Input
                id="tracking-interval"
                type="number"
                placeholder="5"
                defaultValue="5"
              />
            </div>
          </CardContent>
        </Card>

        {/* Notifications */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Bell className="h-5 w-5" />
              Notifications
            </CardTitle>
            <CardDescription>
              Configure when and how you receive notifications
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="font-medium">Break Reminders</p>
                <p className="text-sm text-muted-foreground">
                  Get reminded to take breaks during long work sessions
                </p>
              </div>
              <Button variant="outline" size="sm">
                Configure
              </Button>
            </div>
            <div className="flex items-center justify-between">
              <div>
                <p className="font-medium">Daily Summary</p>
                <p className="text-sm text-muted-foreground">
                  Receive a daily summary of your activities
                </p>
              </div>
              <Button variant="outline" size="sm">
                Configure
              </Button>
            </div>
          </CardContent>
        </Card>

        {/* Data Management */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Database className="h-5 w-5" />
              Data Management
            </CardTitle>
            <CardDescription>
              Manage your activity data and privacy settings
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="font-medium">Export Data</p>
                <p className="text-sm text-muted-foreground">
                  Export your activity data to a file
                </p>
              </div>
              <Button variant="outline" size="sm">
                Export
              </Button>
            </div>
            <div className="flex items-center justify-between">
              <div>
                <p className="font-medium">Clear Data</p>
                <p className="text-sm text-muted-foreground">
                  Remove all stored activity data
                </p>
              </div>
              <Button variant="destructive" size="sm">
                Clear All
              </Button>
            </div>
          </CardContent>
        </Card>

        {/* Appearance */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Palette className="h-5 w-5" />
              Appearance
            </CardTitle>
            <CardDescription>
              Customize the look and feel of the application
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="font-medium">Theme</p>
                <p className="text-sm text-muted-foreground">
                  Choose between light and dark modes
                </p>
              </div>
              <Button variant="outline" size="sm">
                Dark Mode
              </Button>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
