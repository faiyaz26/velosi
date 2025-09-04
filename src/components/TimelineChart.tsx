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
import { Clock, ChevronDown, ChevronRight, RefreshCw } from "lucide-react";
import { format } from "date-fns";

interface TimelineSegment {
  id: string;
  start_time: string;
  end_time: string | null;
  duration_seconds: number;
  segment_type: string;
  title: string;
  url?: string;
  file_path?: string;
  metadata?: string;
}

interface TimelineActivity {
  id: string;
  start_time: string;
  end_time: string | null;
  duration_seconds: number;
  app_name: string;
  app_bundle_id?: string;
  window_title: string;
  url?: string;
  category: string;
  segments: TimelineSegment[];
}

interface TimelineData {
  start_time: string;
  end_time: string;
  activities: TimelineActivity[];
}

interface TimelineChartProps {
  minutes?: number;
}

export function TimelineChart({ minutes = 30 }: TimelineChartProps) {
  const [timelineData, setTimelineData] = useState<TimelineData | null>(null);
  const [loading, setLoading] = useState(false);
  const [expandedActivities, setExpandedActivities] = useState<Set<string>>(
    new Set()
  );
  const [selectedMinutes, setSelectedMinutes] = useState(minutes);

  const loadTimelineData = async () => {
    setLoading(true);
    try {
      const data = await invoke<TimelineData>("get_timeline_data", {
        minutes: selectedMinutes,
      });
      setTimelineData(data);
    } catch (error) {
      console.error("Failed to load timeline data:", error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadTimelineData();
    // Auto-refresh every 30 seconds
    const interval = setInterval(loadTimelineData, 30000);
    return () => clearInterval(interval);
  }, [selectedMinutes]);

  const toggleActivity = (activityId: string) => {
    const newExpanded = new Set(expandedActivities);
    if (newExpanded.has(activityId)) {
      newExpanded.delete(activityId);
    } else {
      newExpanded.add(activityId);
    }
    setExpandedActivities(newExpanded);
  };

  const getCategoryColor = (category: string) => {
    const colors = {
      Development: "bg-blue-500",
      Productive: "bg-green-500",
      Communication: "bg-yellow-500",
      Social: "bg-purple-500",
      Entertainment: "bg-red-500",
      Unknown: "bg-gray-500",
    };
    return colors[category as keyof typeof colors] || colors.Unknown;
  };

  const formatDuration = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}:${secs.toString().padStart(2, "0")}`;
  };

  const getTimelinePosition = (startTime: string, endTime: string | null) => {
    if (!timelineData) return { left: 0, width: 0 };

    const timelineStart = new Date(timelineData.start_time).getTime();
    const timelineEnd = new Date(timelineData.end_time).getTime();
    const timelineDuration = timelineEnd - timelineStart;

    const activityStart = new Date(startTime).getTime();
    const activityEnd = endTime ? new Date(endTime).getTime() : timelineEnd;

    const left = ((activityStart - timelineStart) / timelineDuration) * 100;
    const width = ((activityEnd - activityStart) / timelineDuration) * 100;

    return { left: Math.max(0, left), width: Math.max(0.5, width) };
  };

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle className="flex items-center gap-2">
              <Clock className="h-5 w-5" />
              Activity Timeline
            </CardTitle>
            <CardDescription>
              Last {selectedMinutes} minutes of activity with granular details
            </CardDescription>
          </div>
          <div className="flex items-center gap-2">
            <select
              value={selectedMinutes}
              onChange={(e) => setSelectedMinutes(Number(e.target.value))}
              className="px-3 py-1 border rounded-md text-sm"
            >
              <option value={15}>15 min</option>
              <option value={30}>30 min</option>
              <option value={60}>1 hour</option>
              <option value={120}>2 hours</option>
            </select>
            <Button
              onClick={loadTimelineData}
              disabled={loading}
              size="sm"
              variant="outline"
            >
              <RefreshCw
                className={`h-4 w-4 ${loading ? "animate-spin" : ""}`}
              />
            </Button>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        {loading && !timelineData ? (
          <div className="text-center py-8">
            <p className="text-muted-foreground">Loading timeline...</p>
          </div>
        ) : timelineData && timelineData.activities.length > 0 ? (
          <div className="space-y-4">
            {/* Timeline Header */}
            <div className="flex justify-between text-xs text-muted-foreground mb-2">
              <span>{format(new Date(timelineData.start_time), "HH:mm")}</span>
              <span>{format(new Date(timelineData.end_time), "HH:mm")}</span>
            </div>

            {/* Timeline Bar */}
            <div className="relative h-8 bg-gray-100 rounded-lg mb-4">
              {timelineData.activities.map((activity) => {
                const position = getTimelinePosition(
                  activity.start_time,
                  activity.end_time
                );
                return (
                  <div
                    key={activity.id}
                    className={`absolute h-full rounded ${getCategoryColor(
                      activity.category
                    )} opacity-80`}
                    style={{
                      left: `${position.left}%`,
                      width: `${position.width}%`,
                    }}
                    title={`${activity.app_name} - ${formatDuration(
                      activity.duration_seconds
                    )}`}
                  />
                );
              })}
            </div>

            {/* Activity List */}
            <div className="space-y-2">
              {timelineData.activities.map((activity) => (
                <div key={activity.id} className="border rounded-lg">
                  <div
                    className="p-3 cursor-pointer hover:bg-gray-50 flex items-center justify-between"
                    onClick={() => toggleActivity(activity.id)}
                  >
                    <div className="flex items-center gap-3">
                      <div
                        className={`w-3 h-3 rounded-full ${getCategoryColor(
                          activity.category
                        )}`}
                      />
                      <div>
                        <div className="font-medium">{activity.app_name}</div>
                        <div className="text-sm text-muted-foreground truncate max-w-md">
                          {activity.window_title}
                        </div>
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <span className="text-sm text-muted-foreground">
                        {formatDuration(activity.duration_seconds)}
                      </span>
                      {expandedActivities.has(activity.id) ? (
                        <ChevronDown className="h-4 w-4" />
                      ) : (
                        <ChevronRight className="h-4 w-4" />
                      )}
                    </div>
                  </div>

                  {expandedActivities.has(activity.id) && (
                    <div className="px-3 pb-3 space-y-2">
                      <div className="text-xs text-muted-foreground">
                        {format(new Date(activity.start_time), "HH:mm:ss")} -{" "}
                        {activity.end_time
                          ? format(new Date(activity.end_time), "HH:mm:ss")
                          : "ongoing"}
                      </div>
                      {activity.url && (
                        <div className="text-sm">
                          <span className="font-medium">URL: </span>
                          <span className="text-blue-600 truncate">
                            {activity.url}
                          </span>
                        </div>
                      )}
                      {activity.segments && activity.segments.length > 0 && (
                        <div className="mt-2">
                          <div className="text-sm font-medium mb-1">
                            Segments:
                          </div>
                          <div className="space-y-1 ml-4">
                            {activity.segments.map((segment) => (
                              <div key={segment.id} className="text-sm">
                                <div className="flex items-center justify-between">
                                  <span className="truncate">
                                    {segment.title}
                                  </span>
                                  <span className="text-xs text-muted-foreground">
                                    {formatDuration(segment.duration_seconds)}
                                  </span>
                                </div>
                                {segment.url && (
                                  <div className="text-xs text-blue-600 truncate ml-2">
                                    {segment.url}
                                  </div>
                                )}
                              </div>
                            ))}
                          </div>
                        </div>
                      )}
                    </div>
                  )}
                </div>
              ))}
            </div>
          </div>
        ) : (
          <div className="text-center py-8">
            <p className="text-muted-foreground">
              No activities found in the selected timeframe.
            </p>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
