import { format } from "date-fns";
import { Clock, Monitor, Edit } from "lucide-react";
import { useCategoryService } from "@/hooks/useCategoryService";
import { invoke } from "@tauri-apps/api/core";
import { useState, useEffect } from "react";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";

interface ActivityEntry {
  id: string;
  start_time: string;
  end_time: string | null;
  app_name: string;
  app_bundle_id?: string;
  window_title: string;
  url?: string;
  category: any;
}

interface HourActivitiesModalProps {
  isOpen: boolean;
  onClose: () => void;
  hour: number;
  activities: ActivityEntry[];
  onActivityUpdated?: () => void;
}

export function HourActivitiesModal({
  isOpen,
  onClose,
  hour,
  activities,
  onActivityUpdated,
}: HourActivitiesModalProps) {
  const { categoryService, isInitialized } = useCategoryService();
  const [editingActivity, setEditingActivity] = useState<string | null>(null);
  const [updating, setUpdating] = useState<string | null>(null);

  // Handle escape key to cancel editing
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setEditingActivity(null);
      }
    };

    if (editingActivity) {
      document.addEventListener("keydown", handleKeyDown);
      return () => document.removeEventListener("keydown", handleKeyDown);
    }
  }, [editingActivity]);

  const formatDuration = (start: string, end: string | null) => {
    const startTime = new Date(start);
    const endTime = end ? new Date(end) : new Date();
    const duration = (endTime.getTime() - startTime.getTime()) / 1000 / 60; // in minutes

    if (duration < 1) {
      return "< 1m";
    }

    const hours = Math.floor(duration / 60);
    const minutes = Math.floor(duration % 60);

    if (hours > 0) {
      return `${hours}h ${minutes}m`;
    }
    return `${minutes}m`;
  };

  const getCategoryInfo = (activity: ActivityEntry) => {
    if (!isInitialized || !categoryService) {
      return { name: "Unknown", color: "#6b7280" };
    }

    try {
      // Handle both string and object formats for category
      let categoryName = "Unknown";
      if (typeof activity.category === "string") {
        categoryName = activity.category;
      } else if (activity.category && typeof activity.category === "object") {
        const keys = Object.keys(activity.category);
        categoryName = keys[0] || "Unknown";
      }

      const categoryInfo = categoryService.getCategoryById(
        categoryName.toLowerCase()
      );
      return {
        name: categoryInfo?.name || categoryName,
        color: categoryInfo?.color || "#6b7280",
      };
    } catch (error) {
      console.error("Error getting category info:", error);
      return { name: "Unknown", color: "#6b7280" };
    }
  };

  const totalDuration = activities.reduce((total, activity) => {
    const startTime = new Date(activity.start_time);
    const endTime = activity.end_time
      ? new Date(activity.end_time)
      : new Date();
    return total + (endTime.getTime() - startTime.getTime()) / 1000 / 60;
  }, 0);

  const formatTotalDuration = (minutes: number) => {
    const hours = Math.floor(minutes / 60);
    const mins = Math.floor(minutes % 60);

    if (hours > 0) {
      return `${hours}h ${mins}m`;
    }
    return `${mins}m`;
  };

  const updateActivityCategory = async (
    activityId: string,
    categoryId: string
  ) => {
    setUpdating(activityId);
    try {
      await invoke("update_activity_category", {
        activityId: activityId,
        category: categoryId,
      });

      // Call the callback to refresh the parent data
      if (onActivityUpdated) {
        onActivityUpdated();
      }

      setEditingActivity(null);
    } catch (error) {
      console.error("Failed to update activity category:", error);
    } finally {
      setUpdating(null);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <Card className="w-full max-w-2xl max-h-[80vh] overflow-y-auto">
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle className="flex items-center gap-2">
              <Clock className="h-5 w-5" />
              Activities at {hour}:00 - {(hour + 1) % 24}:00
            </CardTitle>
            <Button
              variant="outline"
              size="sm"
              onClick={onClose}
              className="h-8 w-8 p-0"
            >
              ×
            </Button>
          </div>
          <CardDescription>
            {activities.length} activities • Total time:{" "}
            {formatTotalDuration(totalDuration)}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-3">
            {activities.length === 0 ? (
              <div className="text-center py-8 text-muted-foreground">
                No activities recorded for this hour
              </div>
            ) : (
              activities
                .sort(
                  (a, b) =>
                    new Date(a.start_time).getTime() -
                    new Date(b.start_time).getTime()
                )
                .map((activity) => {
                  const categoryInfo = getCategoryInfo(activity);

                  return (
                    <div
                      key={activity.id}
                      className="border rounded-lg p-4 space-y-2 hover:bg-muted/50 transition-colors"
                    >
                      <div className="flex items-start justify-between">
                        <div className="flex items-center gap-2 flex-1 min-w-0">
                          <Monitor className="h-4 w-4 text-muted-foreground flex-shrink-0" />
                          <div className="min-w-0 flex-1">
                            <h4 className="font-medium truncate">
                              {activity.app_name}
                            </h4>
                            <p className="text-sm text-muted-foreground truncate">
                              {activity.window_title}
                            </p>
                          </div>
                        </div>
                        <div className="flex items-center gap-2 flex-shrink-0">
                          {editingActivity === activity.id ? (
                            <div className="flex items-center gap-2">
                              <select
                                className="px-2 py-1 text-xs border rounded disabled:opacity-50"
                                defaultValue={categoryInfo.name.toLowerCase()}
                                onChange={(e) =>
                                  updateActivityCategory(
                                    activity.id,
                                    e.target.value
                                  )
                                }
                                disabled={updating === activity.id}
                              >
                                {categoryService?.getCategories().map((cat) => (
                                  <option key={cat.id} value={cat.id}>
                                    {cat.name}
                                  </option>
                                ))}
                              </select>
                              {updating === activity.id && (
                                <span className="text-xs text-muted-foreground">
                                  Updating...
                                </span>
                              )}
                              <Button
                                variant="outline"
                                size="sm"
                                className="h-6 w-6 p-0"
                                onClick={() => setEditingActivity(null)}
                                title="Cancel"
                              >
                                ×
                              </Button>
                            </div>
                          ) : (
                            <div className="flex items-center gap-1">
                              <span
                                className="px-2 py-1 text-xs rounded-full font-medium cursor-pointer hover:opacity-80"
                                style={{
                                  backgroundColor: `${categoryInfo.color}20`,
                                  color: categoryInfo.color,
                                }}
                                onClick={() => setEditingActivity(activity.id)}
                                title="Click to edit category"
                              >
                                {categoryInfo.name}
                              </span>
                              <Button
                                variant="outline"
                                size="sm"
                                className="h-6 w-6 p-0"
                                onClick={() => setEditingActivity(activity.id)}
                                title="Edit category"
                              >
                                <Edit className="h-3 w-3" />
                              </Button>
                            </div>
                          )}
                          <span className="text-sm text-muted-foreground">
                            {formatDuration(
                              activity.start_time,
                              activity.end_time
                            )}
                          </span>
                        </div>
                      </div>

                      <div className="flex items-center gap-4 text-xs text-muted-foreground">
                        <span>
                          {format(new Date(activity.start_time), "HH:mm:ss")}
                          {activity.end_time &&
                            ` - ${format(
                              new Date(activity.end_time),
                              "HH:mm:ss"
                            )}`}
                        </span>
                        {activity.url && (
                          <span className="truncate">URL: {activity.url}</span>
                        )}
                      </div>
                    </div>
                  );
                })
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
