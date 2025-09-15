import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import {
  Focus,
  Shield,
  ShieldCheck,
  Settings,
  Clock,
  AlertCircle,
  CheckCircle2,
  RefreshCw,
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

interface AllowedAppInfo {
  app_name: string;
  expires_at: number | null;
  is_indefinite: boolean;
  expires_in_minutes: number | null;
}

interface WebsiteBlockerStatus {
  running: boolean;
  system_proxy_enabled: boolean;
  method: string;
  platform: string;
  proxy_address?: string;
  proxy_port?: number;
}

interface BlockedWebsite {
  url: string;
  reason: string;
  timestamp: string;
}

interface ProxyLog {
  message: string;
  timestamp: string;
}

export function FocusMode() {
  const [focusModeEnabled, setFocusModeEnabled] = useState(false);
  const [allowedCategories, setAllowedCategories] = useState<string[]>([]);
  const [availableCategories, setAvailableCategories] = useState<Category[]>(
    []
  );
  const [blockedApps, setBlockedApps] = useState<BlockedApp[]>([]);
  const [allowedApps, setAllowedApps] = useState<AllowedAppInfo[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  // Website blocking state
  const [
    websiteBlockerStatus,
    setWebsiteBlockerStatus,
  ] = useState<WebsiteBlockerStatus | null>(null);
  const [blockedWebsites, setBlockedWebsites] = useState<BlockedWebsite[]>([]);
  const [proxyLogs, setProxyLogs] = useState<ProxyLog[]>([]);

  // Blocking preferences state
  const [appBlockingEnabled, setAppBlockingEnabled] = useState(true);
  const [websiteBlockingPreference, setWebsiteBlockingPreference] = useState(
    true
  );

  useEffect(() => {
    loadFocusModeStatus();
    loadCategories();
    loadAllowedApps();
    loadBlockingPreferences();
    initializeProxyServer();
    loadWebsiteBlockerStatus();

    let cleanup: (() => void) | undefined;
    setupEventListeners().then((cleanupFn) => {
      cleanup = cleanupFn;
    });

    return () => {
      if (cleanup) {
        cleanup();
      }
    };
  }, []);

  const setupEventListeners = async () => {
    // Listen for focus mode changes
    const unlistenFocus = await listen("focus-mode-changed", (event) => {
      setFocusModeEnabled(event.payload as boolean);
    });

    // Listen for blocked apps
    const unlistenBlocked = await listen("app-blocked", (event) => {
      const blockedApp = event.payload as BlockedApp;
      setBlockedApps((prev) => [blockedApp, ...prev.slice(0, 9)]); // Keep last 10
    });

    // Listen for temporarily allowed apps
    const unlistenAllowed = await listen("app-temporarily-allowed", () => {
      // Refresh the allowed apps list when an app is temporarily allowed
      loadAllowedApps();
    });

    // Listen for blocked websites
    const unlistenWebsites = await listen("website-blocked", (event) => {
      const blockedWebsite = event.payload as BlockedWebsite;
      setBlockedWebsites((prev) => [blockedWebsite, ...prev.slice(0, 9)]); // Keep last 10
    });

    // Listen for system proxy changes
    const unlistenSystemProxy = await listen(
      "system-proxy-changed",
      async (event) => {
        console.log("🔄 System proxy status changed:", event.payload);
        // Refresh the website blocker status when system proxy changes
        await loadWebsiteBlockerStatus();
      }
    );

    // Listen for proxy logs
    const unlistenProxy = await listen("proxy-log", (event) => {
      const proxyLog = event.payload as ProxyLog;
      setProxyLogs((prev) => [proxyLog, ...prev.slice(0, 49)]); // Keep last 50 logs
    });

    // Auto-refresh website blocker status every 30 seconds (keep for proxy server status)
    const interval = setInterval(() => {
      loadWebsiteBlockerStatus();
    }, 30000);

    // Return cleanup function
    return () => {
      unlistenFocus();
      unlistenBlocked();
      unlistenAllowed();
      unlistenWebsites();
      unlistenProxy();
      unlistenSystemProxy();
      clearInterval(interval);
    };
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
      const apps = await invoke<AllowedAppInfo[]>(
        "get_focus_mode_allowed_apps_detailed"
      );
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

  const loadBlockingPreferences = async () => {
    try {
      const [appBlocking, websiteBlocking] = await Promise.all([
        invoke<boolean>("get_app_blocking_enabled"),
        invoke<boolean>("get_website_blocking_enabled"),
      ]);
      setAppBlockingEnabled(appBlocking);
      setWebsiteBlockingPreference(websiteBlocking);
    } catch (error) {
      console.error("Failed to load blocking preferences:", error);
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

  const toggleAppBlocking = async () => {
    try {
      const newValue = !appBlockingEnabled;
      await invoke("set_app_blocking_enabled", { enabled: newValue });
      setAppBlockingEnabled(newValue);
    } catch (error) {
      console.error("Failed to toggle app blocking:", error);
    }
  };

  const toggleWebsiteBlocking = async () => {
    try {
      const newValue = !websiteBlockingPreference;
      await invoke("set_website_blocking_enabled", { enabled: newValue });
      setWebsiteBlockingPreference(newValue);
    } catch (error) {
      console.error("Failed to toggle website blocking:", error);
    }
  };

  const loadWebsiteBlockerStatus = async () => {
    try {
      const status = await invoke<WebsiteBlockerStatus>(
        "get_website_blocker_status"
      );
      setWebsiteBlockerStatus(status);
    } catch (error) {
      console.error("Failed to load website blocker status:", error);
    }
  };

  const initializeProxyServer = async () => {
    try {
      const result = await invoke<{
        success: boolean;
        message: string;
        proxy_address?: string;
        proxy_port?: number;
      }>("initialize_proxy_server");

      if (result.success) {
        console.log("✅ Proxy server initialized:", result.message);
        if (result.proxy_address && result.proxy_port) {
          console.log(
            `📡 Proxy running on ${result.proxy_address}:${result.proxy_port}`
          );
        }
      }
    } catch (error) {
      console.error("Failed to initialize proxy server:", error);
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

      {/* Blocking Preferences */}
      <Card className="p-6">
        <div className="flex items-center space-x-2 mb-4">
          <Shield className="h-5 w-5 text-orange-500" />
          <h3 className="font-semibold text-foreground">
            Blocking Preferences
          </h3>
        </div>

        <p className="text-sm text-muted-foreground mb-4">
          Choose which types of blocking to enable during focus mode.
        </p>

        <div className="space-y-4">
          {/* App Blocking Toggle */}
          <div className="flex items-center justify-between p-3 border rounded-lg">
            <div className="flex items-center space-x-3">
              <div
                className={cn(
                  "w-3 h-3 rounded-full",
                  appBlockingEnabled ? "bg-green-500" : "bg-gray-400"
                )}
              />
              <div>
                <p className="font-medium text-foreground">App Blocking</p>
                <p className="text-xs text-muted-foreground">
                  Block apps outside allowed categories
                </p>
              </div>
            </div>
            <Switch
              checked={appBlockingEnabled}
              onCheckedChange={toggleAppBlocking}
            />
          </div>

          {/* Website Blocking Toggle */}
          <div className="flex items-center justify-between p-3 border rounded-lg">
            <div className="flex items-center space-x-3">
              <div
                className={cn(
                  "w-3 h-3 rounded-full",
                  websiteBlockingPreference ? "bg-green-500" : "bg-gray-400"
                )}
              />
              <div>
                <p className="font-medium text-foreground">Website Blocking</p>
                <p className="text-xs text-muted-foreground">
                  Block distracting websites via proxy
                </p>
              </div>
            </div>
            <Switch
              checked={websiteBlockingPreference}
              onCheckedChange={toggleWebsiteBlocking}
            />
          </div>
        </div>
      </Card>

      {/* Website Blocking */}
      <Card className="p-6">
        <div className="flex items-center space-x-2 mb-4">
          <Shield className="h-5 w-5 text-blue-500" />
          <h3 className="font-semibold text-foreground">Website Blocking</h3>
        </div>

        <p className="text-sm text-muted-foreground mb-4">
          Website blocking status and activity. Configure blocking preferences
          above to enable/disable.
        </p>

        <div className="p-3 bg-muted rounded-lg mb-4">
          <div className="flex items-center space-x-3">
            <div
              className={cn(
                "w-3 h-3 rounded-full",
                websiteBlockerStatus?.running ? "bg-green-500" : "bg-orange-500"
              )}
            />
            <div>
              <p className="font-medium text-foreground">
                Proxy Server:{" "}
                {websiteBlockerStatus?.running ? "Running" : "Not Running"}
              </p>
              <p className="text-xs text-muted-foreground">
                {websiteBlockerStatus?.running
                  ? "Local proxy server is active on port 62828"
                  : "Proxy server not initialized"}
              </p>
            </div>
          </div>

          <div className="flex items-center space-x-3">
            <div
              className={cn(
                "w-3 h-3 rounded-full",
                websiteBlockingPreference
                  ? websiteBlockerStatus?.system_proxy_enabled
                    ? "bg-green-500"
                    : "bg-red-500"
                  : "bg-gray-400"
              )}
            />
            <div>
              <p className="font-medium text-foreground">
                System Proxy:{" "}
                {websiteBlockingPreference
                  ? websiteBlockerStatus?.system_proxy_enabled
                    ? "Enabled"
                    : "Disabled"
                  : "Disabled"}
              </p>
              <p className="text-xs text-muted-foreground">
                {websiteBlockingPreference
                  ? websiteBlockerStatus?.system_proxy_enabled
                    ? "System proxy configured - blocking social & entertainment sites"
                    : "System proxy not configured - websites not blocked"
                  : "Website blocking disabled in preferences"}
              </p>
            </div>
          </div>
        </div>

        {/* Show recent blocked websites */}
        {blockedWebsites.length > 0 && (
          <div className="mt-4">
            <h4 className="text-sm font-medium text-foreground mb-2">
              Recently Blocked Websites
            </h4>
            <div className="space-y-2 max-h-32 overflow-y-auto">
              {blockedWebsites.map((site, index) => (
                <div
                  key={index}
                  className="flex items-center justify-between p-2 bg-red-50 dark:bg-red-950/20 rounded border border-red-200 dark:border-red-800"
                >
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium text-red-700 dark:text-red-300 truncate">
                      {site.url}
                    </p>
                    <p className="text-xs text-red-600 dark:text-red-400">
                      {site.reason}
                    </p>
                  </div>
                  <div className="text-xs text-red-500 ml-2">
                    {new Date(site.timestamp).toLocaleTimeString()}
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
      </Card>

      {/* Proxy Server Logs */}
      <Card className="p-6">
        <div className="flex items-center space-x-2 mb-4">
          <RefreshCw className="h-5 w-5 text-blue-500" />
          <h3 className="font-semibold text-foreground">Proxy Server Logs</h3>
        </div>

        <p className="text-sm text-muted-foreground mb-4">
          Real-time activity from the proxy server showing blocked and allowed
          requests.
        </p>

        {proxyLogs.length > 0 ? (
          <div className="space-y-2 max-h-64 overflow-y-auto bg-muted/30 rounded-lg p-3">
            {proxyLogs.map((log, index) => (
              <div
                key={index}
                className="flex items-start space-x-2 text-xs font-mono"
              >
                <span className="text-muted-foreground flex-shrink-0">
                  {new Date(log.timestamp).toLocaleTimeString()}
                </span>
                <span
                  className={cn(
                    "flex-1",
                    log.message.includes("BLOCKED")
                      ? "text-red-600 dark:text-red-400"
                      : log.message.includes("ALLOWED")
                      ? "text-green-600 dark:text-green-400"
                      : "text-foreground"
                  )}
                >
                  {log.message}
                </span>
              </div>
            ))}
          </div>
        ) : (
          <div className="flex items-center justify-center p-8 text-center bg-muted/30 rounded-lg">
            <div>
              <div className="flex justify-center mb-3">
                <RefreshCw className="h-8 w-8 text-muted-foreground/40" />
              </div>
              <h4 className="font-medium text-foreground mb-2">
                No proxy activity yet
              </h4>
              <p className="text-sm text-muted-foreground max-w-md">
                Proxy server logs will appear here when websites are accessed
                during focus mode. Enable focus mode to start seeing activity.
              </p>
            </div>
          </div>
        )}

        {proxyLogs.length > 0 && (
          <div className="mt-3 flex items-center justify-between text-xs text-muted-foreground">
            <span>Showing last {proxyLogs.length} entries</span>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setProxyLogs([])}
              className="h-6 px-2"
            >
              Clear Logs
            </Button>
          </div>
        )}
      </Card>

      {/* Manually Allowed Apps */}
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

        {allowedApps.length > 0 ? (
          <div className="space-y-2">
            {allowedApps.map((appInfo, index) => (
              <div
                key={index}
                className="flex items-center justify-between p-3 bg-green-50 dark:bg-green-950/30 border border-green-200 dark:border-green-800 rounded-lg"
              >
                <div className="flex items-center space-x-3">
                  <CheckCircle2 className="h-4 w-4 text-green-500" />
                  <div>
                    <span className="font-medium text-foreground">
                      {appInfo.app_name}
                    </span>
                    <div className="text-xs text-muted-foreground">
                      {appInfo.is_indefinite ? (
                        <span className="text-blue-600 font-medium">
                          Allowed indefinitely
                        </span>
                      ) : appInfo.expires_in_minutes !== null ? (
                        appInfo.expires_in_minutes > 0 ? (
                          <span className="text-orange-600">
                            Expires in {appInfo.expires_in_minutes} minutes
                          </span>
                        ) : (
                          <span className="text-red-600 font-medium">
                            Expired
                          </span>
                        )
                      ) : null}
                    </div>
                  </div>
                </div>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => removeAllowedApp(appInfo.app_name)}
                  className="text-red-600 hover:text-red-700 hover:bg-red-50"
                >
                  Remove
                </Button>
              </div>
            ))}
          </div>
        ) : (
          <div className="flex items-center justify-center p-8 text-center">
            <div>
              <div className="flex justify-center mb-3">
                <ShieldCheck className="h-12 w-12 text-muted-foreground/40" />
              </div>
              <h4 className="font-medium text-foreground mb-2">
                No manually allowed apps
              </h4>
              <p className="text-sm text-muted-foreground max-w-md">
                When an app is blocked by focus mode, you can temporarily allow
                it from the blocking overlay. Those apps will appear here with
                their expiration times.
              </p>
            </div>
          </div>
        )}
      </Card>

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
