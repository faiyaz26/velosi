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
import { DateSelector } from "@/components/DateSelector";
import { TimelineChart } from "@/components/TimelineChart";
import { AppUsageTreemap } from "@/components/AppUsageTreemap";
import {
  Activity,
  Clock,
  Calendar,
  RefreshCw,
  ExternalLink,
} from "lucide-react";
import { format, subDays } from "date-fns";

interface ActivityEntry {
  id: string;
  start_time: string;
  end_time?: string;
  app_name: string;
  app_bundle_id?: string;
  window_title: string;
  url?: string;
  category: any;
}

export function ActivityLog() {
  const [selectedDate, setSelectedDate] = useState(new Date());
  const [activities, setActivities] = useState<ActivityEntry[]>([]);
  const [
    selectedActivity,
    setSelectedActivity,
  ] = useState<ActivityEntry | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadActivities(selectedDate);
  }, [selectedDate]);

  const loadActivities = async (date: Date) => {
    setLoading(true);
    try {
      const dateString = format(date, "yyyy-MM-dd");
      const result = await invoke<ActivityEntry[]>("get_activities_by_date", {
        date: dateString,
      });
      setActivities(result);
    } catch (error) {
      console.error("Failed to load activities:", error);
      setActivities([]);
    } finally {
      setLoading(false);
    }
  };

  const handleRefresh = () => {
    loadActivities(selectedDate);
  };

  const formatDuration = (startTime: string, endTime?: string): string => {
    const start = new Date(startTime);
    const end = endTime ? new Date(endTime) : new Date();
    const durationMs = end.getTime() - start.getTime();
    const minutes = Math.floor(durationMs / (1000 * 60));
    const hours = Math.floor(minutes / 60);
    const remainingMinutes = minutes % 60;

    if (hours > 0) {
      return `${hours}h ${remainingMinutes}m`;
    }
    return `${minutes}m`;
  };

  const getCategoryColor = (category: any): string => {
    const categoryName = Object.keys(category)[0] || "Unknown";
    const colors: { [key: string]: string } = {
      Development: "bg-blue-500",
      Productive: "bg-green-500",
      Communication: "bg-yellow-500",
      Social: "bg-red-500",
      Entertainment: "bg-purple-500",
      Unknown: "bg-gray-500",
    };
    return colors[categoryName] || colors.Unknown;
  };

  const getCategoryName = (category: any): string => {
    return Object.keys(category)[0] || "Unknown";
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Activity Log</h1>
        <p className="text-muted-foreground mt-2">
          View detailed logs of your tracked activities
        </p>
      </div>

      {/* Date Selector */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <DateSelector
            selectedDate={selectedDate}
            onDateChange={setSelectedDate}
          />
          <div className="flex gap-2">
            <Button
              size="sm"
              variant="outline"
              onClick={() => setSelectedDate(new Date())}
            >
              Today
            </Button>
            <Button
              size="sm"
              variant="outline"
              onClick={() => setSelectedDate(subDays(new Date(), 1))}
            >
              Yesterday
            </Button>
          </div>
        </div>
        <Button
          onClick={handleRefresh}
          disabled={loading}
          size="sm"
          variant="outline"
          className="flex items-center gap-2"
        >
          <RefreshCw className={`h-4 w-4 ${loading ? "animate-spin" : ""}`} />
          Refresh
        </Button>
      </div>

      {/* Timeline */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Calendar className="h-5 w-5" />
            Activity Timeline
          </CardTitle>
          <CardDescription>
            {format(selectedDate, "EEEE, MMMM d, yyyy")} - Click on activities
            to see details
          </CardDescription>
        </CardHeader>
        <CardContent>
          <TimelineChart
            activities={activities}
            onActivityClick={(activity: ActivityEntry) => {
              setSelectedActivity(activity);
            }}
          />
        </CardContent>
      </Card>

      {/* App Usage Treemap */}
      <AppUsageTreemap activities={activities} />

      {/* Selected Activity Details */}
      {selectedActivity && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Activity className="h-5 w-5" />
              Activity Details
            </CardTitle>
            <CardDescription>Selected activity information</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="grid gap-4">
              <div className="flex items-start gap-4">
                <div
                  className={`h-4 w-4 rounded-full mt-1 ${getCategoryColor(
                    selectedActivity.category
                  )}`}
                ></div>
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-2">
                    <h3 className="text-lg font-semibold">
                      {selectedActivity.app_name}
                    </h3>
                    <span className="px-2 py-1 text-xs rounded-full bg-muted">
                      {getCategoryName(selectedActivity.category)}
                    </span>
                  </div>
                  <p className="text-muted-foreground mb-3">
                    {selectedActivity.window_title}
                  </p>

                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
                    <div>
                      <span className="font-medium">Duration:</span>
                      <p className="text-muted-foreground">
                        {formatDuration(
                          selectedActivity.start_time,
                          selectedActivity.end_time
                        )}
                      </p>
                    </div>
                    <div>
                      <span className="font-medium">Time:</span>
                      <p className="text-muted-foreground">
                        <Clock className="inline h-3 w-3 mr-1" />
                        {format(new Date(selectedActivity.start_time), "HH:mm")}
                        {selectedActivity.end_time && (
                          <>
                            {" "}
                            -{" "}
                            {format(
                              new Date(selectedActivity.end_time),
                              "HH:mm"
                            )}
                          </>
                        )}
                      </p>
                    </div>
                    {selectedActivity.app_bundle_id && (
                      <div>
                        <span className="font-medium">Bundle ID:</span>
                        <p className="text-muted-foreground text-xs">
                          {selectedActivity.app_bundle_id}
                        </p>
                      </div>
                    )}
                    {selectedActivity.url && (
                      <div>
                        <span className="font-medium">URL:</span>
                        <div className="flex items-center gap-2">
                          <p className="text-muted-foreground text-xs truncate max-w-[200px]">
                            {selectedActivity.url}
                          </p>
                          <Button
                            size="sm"
                            variant="ghost"
                            className="h-6 w-6 p-0"
                            onClick={() =>
                              window.open(selectedActivity.url, "_blank")
                            }
                          >
                            <ExternalLink className="h-3 w-3" />
                          </Button>
                        </div>
                      </div>
                    )}
                  </div>
                </div>
              </div>

              <div className="flex justify-end">
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => setSelectedActivity(null)}
                >
                  Close Details
                </Button>
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Loading State */}
      {loading && (
        <div className="text-center p-8">
          <RefreshCw className="h-8 w-8 animate-spin mx-auto mb-4" />
          <p className="text-muted-foreground">Loading activities...</p>
        </div>
      )}

      {/* Empty State */}
      {!loading && activities.length === 0 && (
        <Card>
          <CardContent className="text-center p-8">
            <Activity className="h-12 w-12 mx-auto mb-4 text-muted-foreground" />
            <h3 className="text-lg font-semibold mb-2">No Activities Found</h3>
            <p className="text-muted-foreground">
              No activities were recorded for{" "}
              {format(selectedDate, "MMMM d, yyyy")}.
            </p>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
