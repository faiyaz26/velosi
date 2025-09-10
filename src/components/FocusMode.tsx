import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import {
  Focus,
  Shield,
  ShieldCheck,
  Settings,
  Clock,
  AlertCircle,
  CheckCircle2,
} from "lucide-react";
import { cn } from "@/lib/utils";

interface Category {
  id: string;
  name: string;
  description: string;
  color: string;
}

interface BlockedApp {
  app_name: string;
  reason: string;
  timestamp: string;
}

export function FocusMode() {
  const [focusModeEnabled, setFocusModeEnabled] = useState(false);
  const [allowedCategories, setAllowedCategories] = useState<string[]>([]);
  const [availableCategories, setAvailableCategories] = useState<Category[]>(
    []
  );
  const [blockedApps, setBlockedApps] = useState<BlockedApp[]>([]);
  const [allowedApps, setAllowedApps] = useState<string[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    loadFocusModeStatus();
    loadCategories();
    loadAllowedApps();
    setupEventListeners();
  }, []);

  const setupEventListeners = async () => {
    // Listen for focus mode changes
    await listen("focus-mode-changed", (event) => {
      setFocusModeEnabled(event.payload as boolean);
    });

    // Listen for blocked apps
    await listen("app-blocked", (event) => {
      const blockedApp = event.payload as BlockedApp;
      setBlockedApps((prev) => [blockedApp, ...prev.slice(0, 9)]); // Keep last 10
    });

    // Listen for temporarily allowed apps
    await listen("app-temporarily-allowed", () => {
      // Refresh the allowed apps list when an app is temporarily allowed
      loadAllowedApps();
    });
  };

  const loadFocusModeStatus = async () => {
    try {
      const [enabled, categories] = await Promise.all([
        invoke<boolean>("get_focus_mode_status"),
        invoke<string[]>("get_focus_mode_categories"),
      ]);
      setFocusModeEnabled(enabled);
      setAllowedCategories(categories);
    } catch (error) {
      console.error("Failed to load focus mode status:", error);
    }
  };

  const loadCategories = async () => {
    try {
      const categories = await invoke<Category[]>("get_categories");
      setAvailableCategories(categories);
    } catch (error) {
      console.error("Failed to load categories:", error);
    } finally {
      setIsLoading(false);
    }
  };

  const loadAllowedApps = async () => {
    try {
      const apps = await invoke<string[]>("get_focus_mode_allowed_apps");
      setAllowedApps(apps);
    } catch (error) {
      console.error("Failed to load allowed apps:", error);
    }
  };

  const removeAllowedApp = async (appName: string) => {
    try {
      await invoke("remove_focus_mode_allowed_app", { appName });
      await loadAllowedApps(); // Refresh the list
    } catch (error) {
      console.error("Failed to remove allowed app:", error);
    }
  };

  const toggleFocusMode = async () => {
    try {
      if (focusModeEnabled) {
        await invoke("disable_focus_mode");
      } else {
        if (allowedCategories.length === 0) {
          alert(
            "Please select at least one category before enabling focus mode."
          );
          return;
        }
        await invoke("enable_focus_mode");
      }
    } catch (error) {
      console.error("Failed to toggle focus mode:", error);
    }
  };

  const toggleCategory = async (categoryId: string) => {
    const newCategories = allowedCategories.includes(categoryId)
      ? allowedCategories.filter((id) => id !== categoryId)
      : [...allowedCategories, categoryId];

    try {
      await invoke("set_focus_mode_categories", { categories: newCategories });
      setAllowedCategories(newCategories);
    } catch (error) {
      console.error("Failed to update categories:", error);
    }
  };

  if (isLoading) {
    return (
      <div className="p-6">
        <div className="animate-pulse">
          <div className="h-8 bg-muted rounded w-1/3 mb-4"></div>
          <div className="space-y-3">
            <div className="h-4 bg-muted rounded"></div>
            <div className="h-4 bg-muted rounded w-2/3"></div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="p-6 space-y-6">
      {/* Header */}
      <div className="flex items-center space-x-3">
        <Focus className="h-8 w-8 text-primary" />
        <div>
          <h1 className="text-2xl font-bold text-foreground">Focus Mode</h1>
          <p className="text-muted-foreground">
            Block distracting apps and stay focused on what matters
          </p>
        </div>
      </div>

      {/* Status Card */}
      <Card className="p-6">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-3">
            {focusModeEnabled ? (
              <ShieldCheck className="h-6 w-6 text-green-500" />
            ) : (
              <Shield className="h-6 w-6 text-muted-foreground" />
            )}
            <div>
              <h3 className="font-semibold text-foreground">
                Focus Mode {focusModeEnabled ? "Active" : "Inactive"}
              </h3>
              <p className="text-sm text-muted-foreground">
                {focusModeEnabled
                  ? "Apps outside allowed categories are being blocked"
                  : "All apps are currently allowed"}
              </p>
            </div>
          </div>
          <Button
            onClick={toggleFocusMode}
            variant={focusModeEnabled ? "destructive" : "default"}
            size="lg"
          >
            {focusModeEnabled ? "Disable" : "Enable"}
          </Button>
        </div>
      </Card>

      {/* Category Selection */}
      <Card className="p-6">
        <div className="flex items-center space-x-2 mb-4">
          <Settings className="h-5 w-5 text-primary" />
          <h3 className="font-semibold text-foreground">Allowed Categories</h3>
        </div>

        <p className="text-sm text-muted-foreground mb-4">
          Select which app categories should be allowed during focus mode. Apps
          in unselected categories will be blocked.
        </p>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
          {availableCategories.map((category) => {
            const isSelected = allowedCategories.includes(category.id);
            return (
              <div
                key={category.id}
                className={cn(
                  "flex items-center space-x-3 p-3 rounded-lg border-2 cursor-pointer transition-all",
                  isSelected
                    ? "border-primary bg-primary/5"
                    : "border-border hover:border-muted-foreground"
                )}
                onClick={() => toggleCategory(category.id)}
              >
                <div
                  className="w-4 h-4 rounded-full flex-shrink-0"
                  style={{ backgroundColor: category.color }}
                />
                <div className="flex-1">
                  <div className="flex items-center justify-between">
                    <span className="font-medium text-foreground">
                      {category.name}
                    </span>
                    {isSelected && (
                      <CheckCircle2 className="h-4 w-4 text-primary" />
                    )}
                  </div>
                  <p className="text-xs text-muted-foreground">
                    {category.description}
                  </p>
                </div>
              </div>
            );
          })}
        </div>

        {allowedCategories.length > 0 && (
          <div className="mt-4 p-3 bg-muted rounded-lg">
            <div className="flex items-center space-x-2 mb-2">
              <CheckCircle2 className="h-4 w-4 text-green-500" />
              <span className="text-sm font-medium text-foreground">
                {allowedCategories.length} categories selected
              </span>
            </div>
            <div className="flex flex-wrap gap-2">
              {allowedCategories.map((categoryId) => {
                const category = availableCategories.find(
                  (c) => c.id === categoryId
                );
                return category ? (
                  <span
                    key={categoryId}
                    className="px-2 py-1 bg-primary/10 text-primary text-xs rounded-full"
                  >
                    {category.name}
                  </span>
                ) : null;
              })}
            </div>
          </div>
        )}
      </Card>

      {/* Manually Allowed Apps */}
      {allowedApps.length > 0 && (
        <Card className="p-6">
          <div className="flex items-center space-x-2 mb-4">
            <ShieldCheck className="h-5 w-5 text-green-500" />
            <h3 className="font-semibold text-foreground">
              Manually Allowed Apps
            </h3>
          </div>

          <p className="text-sm text-muted-foreground mb-4">
            These apps are temporarily allowed and will bypass focus mode
            restrictions.
          </p>

          <div className="space-y-2">
            {allowedApps.map((appName, index) => (
              <div
                key={index}
                className="flex items-center justify-between p-3 bg-green-50 dark:bg-green-950/30 border border-green-200 dark:border-green-800 rounded-lg"
              >
                <div className="flex items-center space-x-3">
                  <CheckCircle2 className="h-4 w-4 text-green-500" />
                  <span className="font-medium text-foreground">{appName}</span>
                </div>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => removeAllowedApp(appName)}
                  className="text-red-600 hover:text-red-700 hover:bg-red-50"
                >
                  Remove
                </Button>
              </div>
            ))}
          </div>
        </Card>
      )}

      {/* Recent Blocked Apps */}
      {focusModeEnabled && blockedApps.length > 0 && (
        <Card className="p-6">
          <div className="flex items-center space-x-2 mb-4">
            <AlertCircle className="h-5 w-5 text-orange-500" />
            <h3 className="font-semibold text-foreground">
              Recent Blocked Apps
            </h3>
          </div>

          <div className="space-y-2">
            {blockedApps.map((blockedApp, index) => (
              <div
                key={index}
                className="flex items-center justify-between p-3 bg-muted/50 rounded-lg"
              >
                <div>
                  <span className="font-medium text-foreground">
                    {blockedApp.app_name}
                  </span>
                  <p className="text-xs text-muted-foreground">
                    {blockedApp.reason}
                  </p>
                </div>
                <div className="flex items-center space-x-1 text-xs text-muted-foreground">
                  <Clock className="h-3 w-3" />
                  <span>
                    {new Date(blockedApp.timestamp).toLocaleTimeString()}
                  </span>
                </div>
              </div>
            ))}
          </div>
        </Card>
      )}

      {/* Tips */}
      <Card className="p-6 bg-muted/30">
        <h4 className="font-semibold text-foreground mb-2">
          Tips for Effective Focus Sessions
        </h4>
        <ul className="text-sm text-muted-foreground space-y-1">
          <li>• Select only essential categories for your current task</li>
          <li>• Blocked apps will show a popup asking what you'd like to do</li>
          <li>
            • You can temporarily allow apps for 30 minutes from the popup
          </li>
          <li>
            • You can disable focus mode anytime if you need access to blocked
            apps
          </li>
        </ul>
      </Card>
    </div>
  );
}
