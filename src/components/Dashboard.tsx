import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Play, Pause, Clock, ChevronDown } from "lucide-react";
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

export function Dashboard() {
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

  useEffect(() => {
    loadTodaysActivitySummary();
    loadTrackingStatus();
    loadPauseStatus();

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

    // Auto-refresh every 90 seconds
    const interval = setInterval(() => {
      loadTodaysActivitySummary();
      loadTrackingStatus();
      loadPauseStatus();
    }, 90000);

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
      await invoke("pause_tracking_for_duration", { minutes });
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
      {/* Top Row - Tracking Status */}
      <div className="grid grid-cols-1 gap-4">
        {/* Tracking Status */}
        <Card>
          <CardHeader className="pb-3"></CardHeader>
          <CardContent className="space-y-4">
            {/* Current Status */}
            <div className="flex items-center justify-between">
              <div>
                <p className="text-base font-semibold">
                  {isTracking
                    ? "Tracking Active"
                    : pauseStatus.is_paused
                    ? "Tracking temporarily Paused"
                    : "Tracking Paused"}
                </p>
                <p className="text-muted-foreground text-sm">
                  {isTracking
                    ? "Activity tracking is running"
                    : pauseStatus.is_paused
                    ? pauseStatus.is_indefinite
                      ? "Paused indefinitely - click Resume to continue"
                      : `Resumes in ${Math.ceil(
                          pauseStatus.remaining_seconds / 60
                        )} minutes`
                    : "Activity tracking is stopped"}
                </p>
              </div>
              {isTracking ? (
                <Pause className="h-6 w-6 text-green-500" />
              ) : pauseStatus.is_paused ? (
                <Clock className="h-6 w-6 text-orange-500" />
              ) : (
                <Play className="h-6 w-6 text-red-500" />
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
                        Pause
                        <ChevronDown className="h-4 w-4 ml-2" />
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="start">
                      <DropdownMenuItem onClick={() => pauseForDuration(1)}>
                        Pause for 1 minute
                      </DropdownMenuItem>
                      <DropdownMenuItem onClick={() => pauseForDuration(5)}>
                        Pause for 5 minutes
                      </DropdownMenuItem>
                      <DropdownMenuItem onClick={() => pauseForDuration(30)}>
                        Pause for 30 minutes
                      </DropdownMenuItem>
                      <DropdownMenuItem onClick={() => pauseForDuration(60)}>
                        Pause for 1 hour
                      </DropdownMenuItem>
                      <DropdownMenuItem onClick={pauseUntilTomorrow}>
                        Pause until tomorrow
                      </DropdownMenuItem>
                      <DropdownMenuItem onClick={pauseIndefinitely}>
                        Pause indefinitely
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
