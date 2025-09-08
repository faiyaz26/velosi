import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Card, CardContent } from "@/components/ui/card";
import { Play, Pause, Monitor, Tag } from "lucide-react";
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

interface CurrentActivity {
  app_name: string;
  app_bundle_id?: string;
  window_title: string;
  url?: string;
  timestamp: string;
}

function formatDuration(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);

  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }
  return `${minutes}m`;
}

export function Dashboard() {
  const { isInitialized, categoryService } = useCategoryService();
  const [
    activitySummary,
    setActivitySummary,
  ] = useState<ActivitySummary | null>(null);
  const [loading, setLoading] = useState(false);
  const [isTracking, setIsTracking] = useState(false);
  const [
    currentActivity,
    setCurrentActivity,
  ] = useState<CurrentActivity | null>(null);
  const [selectedHour, setSelectedHour] = useState<number | null>(null);
  const [selectedActivities, setSelectedActivities] = useState<any[]>([]);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [heatmapRefresh, setHeatmapRefresh] = useState(0);

  useEffect(() => {
    loadTodaysActivitySummary();
    loadTrackingStatus();
    loadCurrentActivity();

    // Listen for tracking status changes from tray or other sources
    const unlistenPromise = listen<boolean>(
      "tracking-status-changed",
      (event) => {
        setIsTracking(event.payload);
        if (!event.payload) {
          setCurrentActivity(null);
        }
      }
    );

    // Auto-refresh every 10 seconds
    const interval = setInterval(() => {
      loadTodaysActivitySummary();
      loadTrackingStatus();
      loadCurrentActivity();
    }, 10000);

    return () => {
      clearInterval(interval);
      // Clean up event listener
      unlistenPromise.then((unlisten) => unlisten());
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

  const loadCurrentActivity = async () => {
    try {
      const activity = await invoke<CurrentActivity | null>(
        "get_current_activity"
      );
      setCurrentActivity(activity);
    } catch (error) {
      console.error("Failed to load current activity:", error);
    }
  };

  const getCurrentActivityCategory = () => {
    if (!currentActivity || !isInitialized || !categoryService) {
      return "Unknown";
    }

    try {
      const categoryInfo = categoryService.getCategoryByAppName(
        currentActivity.app_name
      );
      return categoryInfo?.name || "Unknown";
    } catch (error) {
      return "Unknown";
    }
  };

  const toggleTracking = async () => {
    try {
      if (isTracking) {
        await invoke("stop_tracking");
        setIsTracking(false);
        setCurrentActivity(null);
      } else {
        await invoke("start_tracking");
        setIsTracking(true);
      }
    } catch (error) {
      console.error("Failed to toggle tracking:", error);
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
      {/* Top Row - Tracking Status, Current App, Current Category */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
        {/* Tracking Status */}
        <Card
          className="cursor-pointer hover:bg-muted/50 transition-colors"
          onClick={toggleTracking}
        >
          <CardContent className="flex items-center justify-between p-4">
            <div>
              <p className="text-base font-semibold">
                {isTracking ? "Tracking Active" : "Tracking Paused"}
              </p>
              <p className="text-muted-foreground text-sm">
                {isTracking
                  ? "Click to pause tracking"
                  : "Click to start tracking"}
              </p>
            </div>
            {isTracking ? (
              <Play className="h-6 w-6 text-green-500" />
            ) : (
              <Pause className="h-6 w-6 text-red-500" />
            )}
          </CardContent>
        </Card>

        {/* Current App */}
        <Card>
          <CardContent className="flex items-center justify-between p-4">
            <div className="min-w-0 flex-1">
              <p className="text-base font-semibold truncate">
                {currentActivity?.app_name || "No Activity"}
              </p>
              <p className="text-muted-foreground text-sm truncate">
                {currentActivity?.window_title || "Waiting for activity..."}
              </p>
            </div>
            <Monitor className="h-6 w-6 text-blue-500 flex-shrink-0 ml-2" />
          </CardContent>
        </Card>

        {/* Current Category */}
        <Card>
          <CardContent className="flex items-center justify-between p-4">
            <div className="min-w-0 flex-1">
              <p className="text-base font-semibold truncate">
                {getCurrentActivityCategory()}
              </p>
              <p className="text-muted-foreground text-sm">Current category</p>
            </div>
            <Tag className="h-6 w-6 text-purple-500 flex-shrink-0 ml-2" />
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
