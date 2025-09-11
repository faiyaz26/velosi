import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  Play,
  Pause,
  Clock,
  ChevronDown,
  Shield,
  ShieldCheck,
  Settings,
} from "lucide-react";
import { RingChart } from "@/components/RingChart";
import { format } from "date-fns";
import { useCategoryService } from "@/hooks/useCategoryService";
import { getCategoryColor, getCategoryName } from "@/lib/utils";
import { HourlyHeatmap } from "@/components/HourlyHeatmap";
import { HourActivitiesModal } from "@/components/HourActivitiesModal";

interface ActivityCategory {
  Productive?: null;
  Social?: null;
  Entertainment?: null;
  Development?: null;
  Communication?: null;
  Unknown?: null;
}

interface CategorySummary {
  category: ActivityCategory;
  duration_seconds: number;
  percentage: number;
}

interface AppSummary {
  app_name: string;
  duration_seconds: number;
  percentage: number;
}

interface ActivitySummary {
  date: string;
  total_active_time: number;
  categories: CategorySummary[];
  top_apps: AppSummary[];
}

function formatDuration(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);

  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }
  return `${minutes}m`;
}

export function Dashboard({
  onNavigate,
}: {
  onNavigate?: (view: string) => void;
}) {
  const { isInitialized, categoryService } = useCategoryService();
  const [
    activitySummary,
    setActivitySummary,
  ] = useState<ActivitySummary | null>(null);
  const [loading, setLoading] = useState(false);
  const [isTracking, setIsTracking] = useState(false);
  const [selectedHour, setSelectedHour] = useState<number | null>(null);
  const [selectedActivities, setSelectedActivities] = useState<any[]>([]);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [heatmapRefresh, setHeatmapRefresh] = useState(0);
  const [pauseStatus, setPauseStatus] = useState<{
    is_paused: boolean;
    remaining_seconds: number;
    is_indefinite?: boolean;
  }>({ is_paused: false, remaining_seconds: 0, is_indefinite: false });
  const [focusModeEnabled, setFocusModeEnabled] = useState(false);
  const [focusModeCategories, setFocusModeCategories] = useState<string[]>([]);

  useEffect(() => {
    loadTodaysActivitySummary();
    loadTrackingStatus();
    loadPauseStatus();
    loadFocusModeStatus();

    // Listen for tracking status changes from tray or other sources
    const unlistenPromise = listen<boolean>(
      "tracking-status-changed",
      (event) => {
        setIsTracking(event.payload);
        if (event.payload) {
          setPauseStatus({ is_paused: false, remaining_seconds: 0 });
        } else {
          loadPauseStatus();
        }
      }
    );

    // Listen for focus mode changes
    const unlistenFocusPromise = listen<boolean>(
      "focus-mode-changed",
      (event) => {
        setFocusModeEnabled(event.payload);
      }
    );

    // Auto-refresh every 90 seconds
    const interval = setInterval(() => {
      loadTodaysActivitySummary();
      loadTrackingStatus();
      loadPauseStatus();
    }, 90000);

    return () => {
      clearInterval(interval);
      // Clean up event listeners
      unlistenPromise.then((unlisten) => unlisten());
      unlistenFocusPromise.then((unlisten) => unlisten());
    };
  }, []);

  const loadTodaysActivitySummary = async () => {
    setLoading(true);
    try {
      const dateString = format(new Date(), "yyyy-MM-dd");
      const summary = await invoke<ActivitySummary>("get_activity_summary", {
        date: dateString,
      });
      setActivitySummary(summary);
    } catch (error) {
      console.error("Failed to load activity summary:", error);
      setActivitySummary(null);
    } finally {
      setLoading(false);
    }
  };

  const loadTrackingStatus = async () => {
    try {
      const status = await invoke<boolean>("get_tracking_status");
      setIsTracking(status);
    } catch (error) {
      console.error("Failed to load tracking status:", error);
    }
  };

  const loadFocusModeStatus = async () => {
    try {
      const [enabled, categories] = await Promise.all([
        invoke<boolean>("get_focus_mode_status"),
        invoke<string[]>("get_focus_mode_categories"),
      ]);
      setFocusModeEnabled(enabled);
      setFocusModeCategories(categories);
    } catch (error) {
      console.error("Failed to load focus mode status:", error);
    }
  };

  const toggleTracking = async () => {
    try {
      if (isTracking) {
        await invoke("stop_tracking");
        setIsTracking(false);
      } else {
        await invoke("start_tracking");
        setIsTracking(true);
      }
    } catch (error) {
      console.error("Failed to toggle tracking:", error);
    }
  };

  const pauseForDuration = async (minutes: number) => {
    try {
      await invoke("pause_tracking_for_duration", {
        duration_seconds: minutes * 60,
      });
      setIsTracking(false);
      loadPauseStatus();
    } catch (error) {
      console.error("Failed to pause tracking:", error);
    }
  };

  const pauseUntilTomorrow = async () => {
    try {
      await invoke("pause_tracking_until_tomorrow");
      setIsTracking(false);
      loadPauseStatus();
    } catch (error) {
      console.error("Failed to pause tracking until tomorrow:", error);
    }
  };

  const pauseIndefinitely = async () => {
    try {
      await invoke("pause_tracking_indefinitely");
      setIsTracking(false);
      loadPauseStatus();
    } catch (error) {
      console.error("Failed to pause tracking indefinitely:", error);
    }
  };

  const resumeNow = async () => {
    try {
      await invoke("resume_tracking_now");
      setIsTracking(true);
      setPauseStatus({ is_paused: false, remaining_seconds: 0 });
    } catch (error) {
      console.error("Failed to resume tracking:", error);
    }
  };

  const toggleFocusMode = async () => {
    try {
      if (focusModeEnabled) {
        await invoke("disable_focus_mode");
        setFocusModeEnabled(false);
      } else {
        await invoke("enable_focus_mode");
        setFocusModeEnabled(true);
      }
    } catch (error) {
      console.error("Failed to toggle focus mode:", error);
    }
  };

  const loadPauseStatus = async () => {
    try {
      const status = await invoke<{
        is_paused: boolean;
        remaining_seconds: number;
        is_indefinite?: boolean;
      }>("get_pause_status");
      setPauseStatus(status);
    } catch (error) {
      console.error("Failed to load pause status:", error);
    }
  };

  const handleHourClick = (hour: number, activities: any[]) => {
    setSelectedHour(hour);
    setSelectedActivities(activities);
    setIsModalOpen(true);
  };

  const handleCloseModal = () => {
    setIsModalOpen(false);
    setSelectedHour(null);
    setSelectedActivities([]);
  };

  const handleActivityUpdated = () => {
    // Refresh the heatmap data when an activity is updated
    loadTodaysActivitySummary();
    // Trigger heatmap refresh
    setHeatmapRefresh((prev) => prev + 1);
  };

  const pieData =
    activitySummary?.categories.map((cat) => {
      const categoryName = getCategoryName(
        cat.category,
        categoryService,
        isInitialized
      );
      const categoryColor = getCategoryColor(
        cat.category,
        categoryService,
        isInitialized
      );

      return {
        name: categoryName,
        value: cat.duration_seconds,
        percentage: cat.percentage,
        color: categoryColor,
      };
    }) || [];

  return (
    <div className="space-y-4">
      {/* Top Row - Tracking and Focus Mode Status */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {/* Tracking Status Card */}
        <Card>
          <CardHeader className="pb-3">
            <h3 className="text-lg font-semibold">Tracking Status</h3>
          </CardHeader>
          <CardContent className="space-y-4">
            {/* Tracking Status */}
            <div
              className={`flex items-center justify-between p-3 rounded-lg border ${
                isTracking
                  ? "bg-green-50 dark:bg-green-950/30 border-green-200 dark:border-green-800"
                  : pauseStatus.is_paused
                  ? "bg-orange-50 dark:bg-orange-950/30 border-orange-200 dark:border-orange-800"
                  : "bg-gray-50 dark:bg-gray-950/30 border-gray-200 dark:border-gray-800"
              }`}
            >
              <div className="flex items-center space-x-3">
                {isTracking ? (
                  <Pause className="h-5 w-5 text-green-600 dark:text-green-400" />
                ) : pauseStatus.is_paused ? (
                  <Clock className="h-5 w-5 text-orange-600 dark:text-orange-400" />
                ) : (
                  <Play className="h-5 w-5 text-gray-500 dark:text-gray-400" />
                )}
                <div>
                  <p
                    className={`text-sm font-medium ${
                      isTracking
                        ? "text-green-900 dark:text-green-100"
                        : pauseStatus.is_paused
                        ? "text-orange-900 dark:text-orange-100"
                        : "text-gray-900 dark:text-gray-100"
                    }`}
                  >
                    {isTracking
                      ? "Tracking Active"
                      : pauseStatus.is_paused
                      ? "Tracking Paused"
                      : "Tracking Stopped"}
                  </p>
                  <p
                    className={`text-xs ${
                      isTracking
                        ? "text-green-700 dark:text-green-300"
                        : pauseStatus.is_paused
                        ? "text-orange-700 dark:text-orange-300"
                        : "text-gray-600 dark:text-gray-400"
                    }`}
                  >
                    {isTracking
                      ? "Activity tracking is running"
                      : pauseStatus.is_paused
                      ? pauseStatus.is_indefinite
                        ? "Paused indefinitely"
                        : `Resumes in ${Math.ceil(
                            pauseStatus.remaining_seconds / 60
                          )} minutes`
                      : "Activity tracking is stopped"}
                  </p>
                </div>
              </div>
              {isTracking ? (
                <Pause className="h-6 w-6 text-green-600 dark:text-green-400" />
              ) : pauseStatus.is_paused ? (
                <Clock className="h-6 w-6 text-orange-600 dark:text-orange-400" />
              ) : (
                <Play className="h-6 w-6 text-gray-500 dark:text-gray-400" />
              )}
            </div>

            {/* Control Buttons */}
            <div className="flex gap-2">
              {isTracking ? (
                <>
                  <DropdownMenu>
                    <DropdownMenuTrigger asChild>
                      <Button variant="outline" size="sm">
                        <Pause className="h-4 w-4 mr-2" />
                        Pause tracking
                        <ChevronDown className="h-4 w-4 ml-2" />
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="start">
                      <DropdownMenuItem onClick={() => pauseForDuration(1)}>
                        Pause tracking for 1 minute
                      </DropdownMenuItem>
                      <DropdownMenuItem onClick={() => pauseForDuration(5)}>
                        Pause tracking for 5 minutes
                      </DropdownMenuItem>
                      <DropdownMenuItem onClick={() => pauseForDuration(30)}>
                        Pause tracking for 30 minutes
                      </DropdownMenuItem>
                      <DropdownMenuItem onClick={() => pauseForDuration(60)}>
                        Pause tracking for 1 hour
                      </DropdownMenuItem>
                      <DropdownMenuItem onClick={pauseUntilTomorrow}>
                        Pause tracking until tomorrow
                      </DropdownMenuItem>
                      <DropdownMenuItem onClick={pauseIndefinitely}>
                        Pause tracking indefinitely
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </>
              ) : pauseStatus.is_paused ? (
                <Button variant="default" size="sm" onClick={resumeNow}>
                  Resume Now
                </Button>
              ) : (
                <Button variant="default" size="sm" onClick={toggleTracking}>
                  Start Tracking
                </Button>
              )}
            </div>
          </CardContent>
        </Card>

        {/* Focus Mode Status Card */}
        <Card>
          <CardHeader className="pb-3">
            <h3 className="text-lg font-semibold">Focus Mode</h3>
          </CardHeader>
          <CardContent className="space-y-4">
            {/* Focus Mode Status */}
            <div
              className={`flex items-center justify-between p-3 rounded-lg border ${
                focusModeEnabled
                  ? "bg-blue-50 dark:bg-blue-950/30 border-blue-200 dark:border-blue-800"
                  : "bg-gray-50 dark:bg-gray-950/30 border-gray-200 dark:border-gray-800"
              }`}
            >
              <div className="flex items-center space-x-3">
                {focusModeEnabled ? (
                  <ShieldCheck className="h-5 w-5 text-blue-600 dark:text-blue-400" />
                ) : (
                  <Shield className="h-5 w-5 text-gray-500 dark:text-gray-400" />
                )}
                <div>
                  <p
                    className={`text-sm font-medium ${
                      focusModeEnabled
                        ? "text-blue-900 dark:text-blue-100"
                        : "text-gray-900 dark:text-gray-100"
                    }`}
                  >
                    Focus Mode {focusModeEnabled ? "Active" : "Inactive"}
                  </p>
                  <p
                    className={`text-xs ${
                      focusModeEnabled
                        ? "text-blue-700 dark:text-blue-300"
                        : "text-gray-600 dark:text-gray-400"
                    }`}
                  >
                    {focusModeEnabled
                      ? focusModeCategories.length > 0
                        ? `Allowing ${focusModeCategories.length} category${
                            focusModeCategories.length === 1 ? "" : "ies"
                          }`
                        : "Blocking all apps"
                      : "All apps allowed"}
                  </p>
                </div>
              </div>
              {focusModeEnabled ? (
                <ShieldCheck className="h-6 w-6 text-blue-600 dark:text-blue-400" />
              ) : (
                <Shield className="h-6 w-6 text-gray-500 dark:text-gray-400" />
              )}
            </div>

            {/* Focus Mode Control Buttons */}
            <div className="flex gap-2">
              {focusModeEnabled ? (
                <Button variant="outline" size="sm" onClick={toggleFocusMode}>
                  <Pause className="h-4 w-4 mr-2" />
                  Pause focus mode
                </Button>
              ) : (
                <Button
                  variant="default"
                  size="sm"
                  onClick={toggleFocusMode}
                  className="bg-blue-600 hover:bg-blue-700"
                >
                  <Play className="h-4 w-4 mr-2" />
                  Resume focus mode
                </Button>
              )}
              <Button
                variant="ghost"
                size="sm"
                onClick={() => onNavigate?.("focus-mode")}
                className="text-blue-600 hover:text-blue-700 hover:bg-blue-50 dark:text-blue-400 dark:hover:bg-blue-950/30"
              >
                <Settings className="h-4 w-4 mr-1" />
                Configure
              </Button>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Ring Chart Row - Full Width */}
      <RingChart
        data={pieData}
        title="Activity Overview"
        description="Time spent across different categories"
        centerText={
          activitySummary
            ? formatDuration(activitySummary.total_active_time)
            : "0m"
        }
        centerSubText="Total Active Time"
        emptyStateText="No activity data yet"
        emptyStateSubText="Start using your apps to see tracking"
      />

      {/* Hourly Heatmap Row - Full Width */}
      <HourlyHeatmap
        date={new Date()}
        onHourClick={handleHourClick}
        refreshTrigger={heatmapRefresh}
      />

      {loading && (
        <div className="text-center p-8">
          <p className="text-muted-foreground">Loading activity data...</p>
        </div>
      )}

      {/* Hour Activities Modal */}
      {selectedHour !== null && (
        <HourActivitiesModal
          isOpen={isModalOpen}
          onClose={handleCloseModal}
          hour={selectedHour}
          activities={selectedActivities}
          onActivityUpdated={handleActivityUpdated}
        />
      )}
    </div>
  );
}
