import { useState, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { DateRangeSelector, DateRange } from "@/components/DateRangeSelector";
import { TimelineChart } from "@/components/TimelineChart";
import { AppUsageTreemap } from "@/components/AppUsageTreemap";
import { RingChart } from "@/components/RingChart";
import {
  Activity,
  Clock,
  Calendar,
  RefreshCw,
  ExternalLink,
  Edit3,
  Check,
  X,
} from "lucide-react";
import { format, startOfDay } from "date-fns";
import { useCategoryService } from "@/hooks/useCategoryService";
import {
  getCategoryColor,
  getCategoryName,
  getCategoryColorClass,
} from "@/lib/utils";

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

export function ActivityLog() {
  const today = startOfDay(new Date());
  const [selectedRange, setSelectedRange] = useState<DateRange>({
    startDate: today,
    endDate: today,
  });
  const [activities, setActivities] = useState<ActivityEntry[]>([]);
  const [
    activitySummary,
    setActivitySummary,
  ] = useState<ActivitySummary | null>(null);
  const [
    selectedActivity,
    setSelectedActivity,
  ] = useState<ActivityEntry | null>(null);
  const [loading, setLoading] = useState(false);
  const [editingCategory, setEditingCategory] = useState(false);
  const [selectedCategoryId, setSelectedCategoryId] = useState("");
  const { isInitialized, categoryService } = useCategoryService();

  useEffect(() => {
    loadActivities(selectedRange);
    loadActivitySummary();
  }, [selectedRange]);

  const loadActivities = async (range: DateRange) => {
    setLoading(true);
    try {
      const startDateString = format(range.startDate, "yyyy-MM-dd");
      const endDateString = format(range.endDate, "yyyy-MM-dd");
      const result = await invoke<ActivityEntry[]>(
        "get_activities_by_date_range",
        {
          startDate: startDateString,
          endDate: endDateString,
        }
      );
      setActivities(result);
    } catch (error) {
      console.error("Failed to load activities:", error);
      setActivities([]);
    } finally {
      setLoading(false);
    }
  };

  const loadActivitySummary = async () => {
    // Only load activity summary for single day (used for other potential features)
    if (selectedRange.startDate.getTime() === selectedRange.endDate.getTime()) {
      try {
        const result = await invoke<ActivitySummary>("get_activity_summary", {
          date: format(selectedRange.startDate, "yyyy-MM-dd"),
        });
        setActivitySummary(result);
      } catch (error) {
        console.error("Failed to load activity summary:", error);
        setActivitySummary(null);
      }
    } else {
      // For multi-day ranges, we calculate everything from activities directly
      setActivitySummary(null);
    }
  };

  const handleRefresh = () => {
    loadActivities(selectedRange);
    loadActivitySummary();
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

  const updateActivityCategory = async (
    activityId: string,
    categoryId: string
  ) => {
    try {
      console.log(
        "Updating category for activity:",
        activityId,
        "to:",
        categoryId
      );
      await invoke("update_activity_category", {
        activityId: activityId,
        category: categoryId,
      });
      console.log("Category update successful");

      // Refresh the activities to show the updated category
      await loadActivities(selectedRange);
      console.log("Activities reloaded after category update");

      // Update the selected activity if it's the one we just changed
      if (selectedActivity && selectedActivity.id === activityId) {
        console.log("Refreshing selected activity data...");
        const result = await invoke<ActivityEntry[]>(
          "get_activities_by_date_range",
          {
            startDate: format(selectedRange.startDate, "yyyy-MM-dd"),
            endDate: format(selectedRange.endDate, "yyyy-MM-dd"),
          }
        );
        const updatedActivity = result.find(
          (a: ActivityEntry) => a.id === activityId
        );
        if (updatedActivity) {
          console.log("Updated activity found:", updatedActivity);
          console.log("Updated activity category:", updatedActivity.category);
          setSelectedActivity(updatedActivity);
        } else {
          console.log("Updated activity not found in results");
        }
      }

      setEditingCategory(false);
      console.log("Category editing completed successfully");
    } catch (error) {
      console.error("Failed to update category:", error);
      alert("Failed to update category: " + error);
    }
  };

  // Calculate category data based on activities (works for both single and multi-day)
  const { ringChartData, totalActiveTime } = useMemo(() => {
    if (activities.length === 0) {
      return { ringChartData: [], totalActiveTime: 0 };
    }

    // Calculate duration for each activity
    const categoryDurations: Record<string, number> = {};
    let totalDuration = 0;

    activities.forEach((activity) => {
      const startTime = new Date(activity.start_time);
      const endTime = activity.end_time
        ? new Date(activity.end_time)
        : new Date();
      const duration = (endTime.getTime() - startTime.getTime()) / 1000; // seconds

      // Get category key
      const categoryKey =
        typeof activity.category === "string"
          ? activity.category
          : Object.keys(activity.category || {})[0] || "Unknown";

      categoryDurations[categoryKey] =
        (categoryDurations[categoryKey] || 0) + duration;
      totalDuration += duration;
    });

    // Convert to chart data
    const chartData = Object.entries(categoryDurations).map(
      ([category, duration]) => ({
        name: getCategoryName(
          { [category]: null },
          categoryService,
          isInitialized
        ),
        value: duration,
        percentage: totalDuration > 0 ? (duration / totalDuration) * 100 : 0,
        color: getCategoryColor(
          { [category]: null },
          categoryService,
          isInitialized
        ),
      })
    );

    return {
      ringChartData: chartData,
      totalActiveTime: totalDuration,
    };
  }, [activities, categoryService, isInitialized]);

  // Keep activitySummary for potential future use
  void activitySummary;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold tracking-tight">Activity Log</h1>
        <p className="text-muted-foreground mt-2">
          View detailed logs of your tracked activities
        </p>
      </div>

      {/* Date Range Selector - Full Row */}
      <DateRangeSelector
        selectedRange={selectedRange}
        onRangeChange={setSelectedRange}
        onRefresh={handleRefresh}
        loading={loading}
      />

      {/* Timeline */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            Activity Timeline
          </CardTitle>
          <CardDescription>
            {selectedRange.startDate.getTime() ===
            selectedRange.endDate.getTime()
              ? format(selectedRange.startDate, "EEEE, MMMM d, yyyy")
              : `${format(selectedRange.startDate, "MMM d")} - ${format(
                  selectedRange.endDate,
                  "MMM d, yyyy"
                )}`}{" "}
            - Click on activities to see details
          </CardDescription>
        </CardHeader>
        <CardContent>
          <TimelineChart
            activities={activities}
            onActivityClick={(activity: ActivityEntry) => {
              setSelectedActivity(activity);
            }}
            isMultiDay={
              selectedRange.startDate.getTime() !==
              selectedRange.endDate.getTime()
            }
            dateRange={selectedRange}
          />
        </CardContent>
      </Card>

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
                  className={`h-4 w-4 rounded-full mt-1 ${getCategoryColorClass(
                    selectedActivity.category,
                    categoryService,
                    isInitialized
                  )}`}
                ></div>
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-2">
                    <h3 className="text-lg font-semibold">
                      {selectedActivity.app_name}
                    </h3>
                    <div className="flex items-center gap-2">
                      {!editingCategory ? (
                        <>
                          <span className="px-2 py-1 text-xs rounded-full bg-muted">
                            {getCategoryName(
                              selectedActivity.category,
                              categoryService,
                              isInitialized
                            )}
                          </span>
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => {
                              setEditingCategory(true);

                              // Handle both string and object category formats
                              let currentCategoryKey: string;
                              if (
                                typeof selectedActivity.category === "string"
                              ) {
                                currentCategoryKey = selectedActivity.category;
                              } else if (
                                typeof selectedActivity.category === "object"
                              ) {
                                currentCategoryKey =
                                  Object.keys(selectedActivity.category)[0] ||
                                  "unknown";
                              } else {
                                currentCategoryKey = "unknown";
                              }

                              setSelectedCategoryId(
                                currentCategoryKey.toLowerCase()
                              );
                            }}
                          >
                            <Edit3 className="h-3 w-3" />
                          </Button>
                        </>
                      ) : (
                        <div className="flex items-center gap-2">
                          <select
                            value={selectedCategoryId}
                            onChange={(e) =>
                              setSelectedCategoryId(e.target.value)
                            }
                            className="px-2 py-1 text-xs border rounded"
                          >
                            {isInitialized && categoryService
                              ? categoryService
                                  .getCategories()
                                  .map((category: any) => (
                                    <option
                                      key={category.id}
                                      value={category.id}
                                    >
                                      {category.name}
                                    </option>
                                  ))
                              : // Fallback options if category service isn't ready
                                [
                                  { id: "development", name: "Development" },
                                  { id: "productive", name: "Productive" },
                                  {
                                    id: "communication",
                                    name: "Communication",
                                  },
                                  { id: "social", name: "Social" },
                                  {
                                    id: "entertainment",
                                    name: "Entertainment",
                                  },
                                  { id: "unknown", name: "Unknown" },
                                ].map((category) => (
                                  <option key={category.id} value={category.id}>
                                    {category.name}
                                  </option>
                                ))}
                          </select>
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() =>
                              updateActivityCategory(
                                selectedActivity.id,
                                selectedCategoryId
                              )
                            }
                          >
                            <Check className="h-3 w-3" />
                          </Button>
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => setEditingCategory(false)}
                          >
                            <X className="h-3 w-3" />
                          </Button>
                        </div>
                      )}
                    </div>
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
              {selectedRange.startDate.getTime() ===
              selectedRange.endDate.getTime()
                ? format(selectedRange.startDate, "MMMM d, yyyy")
                : `${format(selectedRange.startDate, "MMM d")} - ${format(
                    selectedRange.endDate,
                    "MMM d, yyyy"
                  )}`}
              .
            </p>
          </CardContent>
        </Card>
      )}

      {/* Usage Charts - Side by side at the end */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <RingChart
          data={ringChartData}
          title="Category Usage"
          description="Time spent across different categories"
          centerText={
            totalActiveTime > 0
              ? `${Math.floor(totalActiveTime / 3600)}h ${Math.floor(
                  (totalActiveTime % 3600) / 60
                )}m`
              : "0m"
          }
          centerSubText="Total Active Time"
          emptyStateText="No activities recorded"
          emptyStateSubText="Activities will appear here once tracked"
        />
        <AppUsageTreemap activities={activities} />
      </div>
    </div>
  );
}
