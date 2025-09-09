import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { format, startOfDay, addHours } from "date-fns";

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

interface HourlyData {
  hour: number;
  activityTime: number; // in seconds
  activities: ActivityEntry[];
}

interface HourlyHeatmapProps {
  date: Date;
  onHourClick: (hour: number, activities: ActivityEntry[]) => void;
  refreshTrigger?: number;
}

export function HourlyHeatmap({
  date,
  onHourClick,
  refreshTrigger,
}: HourlyHeatmapProps) {
  const [hourlyData, setHourlyData] = useState<HourlyData[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadHourlyData();
  }, [date, refreshTrigger]);

  const loadHourlyData = async () => {
    setLoading(true);
    try {
      const dateString = format(date, "yyyy-MM-dd");
      const activities = await invoke<ActivityEntry[]>(
        "get_activities_by_date",
        {
          date: dateString,
        }
      );

      // Process activities into hourly buckets
      const hourlyBuckets: HourlyData[] = Array.from(
        { length: 24 },
        (_, i) => ({
          hour: i,
          activityTime: 0,
          activities: [],
        })
      );

      activities.forEach((activity) => {
        const startTime = new Date(activity.start_time);
        const endTime = activity.end_time
          ? new Date(activity.end_time)
          : new Date();

        // Calculate which hours this activity spans
        const startHour = startTime.getHours();
        const endHour = endTime.getHours();

        if (startHour === endHour) {
          // Activity is within a single hour
          const duration = (endTime.getTime() - startTime.getTime()) / 1000;
          hourlyBuckets[startHour].activityTime += duration;
          hourlyBuckets[startHour].activities.push(activity);
        } else {
          // Activity spans multiple hours
          for (let hour = startHour; hour <= endHour; hour++) {
            const hourStart =
              hour === startHour
                ? startTime
                : addHours(startOfDay(startTime), hour);
            const hourEnd =
              hour === endHour
                ? endTime
                : addHours(startOfDay(startTime), hour + 1);

            const duration = (hourEnd.getTime() - hourStart.getTime()) / 1000;
            hourlyBuckets[hour].activityTime += duration;
            hourlyBuckets[hour].activities.push(activity);
          }
        }
      });

      setHourlyData(hourlyBuckets);
    } catch (error) {
      console.error("Failed to load hourly data:", error);
    } finally {
      setLoading(false);
    }
  };

  const maxActivityTime = Math.max(...hourlyData.map((h) => h.activityTime));

  const getIntensityColor = (activityTime: number) => {
    if (activityTime === 0) return "bg-gray-100 dark:bg-gray-800";

    const intensity = activityTime / maxActivityTime;
    if (intensity > 0.8) return "bg-green-600";
    if (intensity > 0.6) return "bg-green-500";
    if (intensity > 0.4) return "bg-green-400";
    if (intensity > 0.2) return "bg-green-300";
    return "bg-green-200";
  };

  const formatDuration = (seconds: number) => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);

    if (hours > 0) {
      return `${hours}h ${minutes}m`;
    }
    return `${minutes}m`;
  };

  return (
    <Card>
      <CardHeader className="pb-3">
        <CardTitle className="flex items-center gap-2">
          Today's Heatmap
        </CardTitle>
        <CardDescription>
          Hourly activity for {format(date, "MMM d")}
        </CardDescription>
      </CardHeader>
      <CardContent className="pb-4">
        {loading ? (
          <div className="flex justify-center items-center h-[100px]">
            <p className="text-muted-foreground text-sm">Loading heatmap...</p>
          </div>
        ) : (
          <div className="h-[100px] flex flex-col justify-center space-y-3 overflow-x-auto">
            {/* Hour labels */}
            <div
              className="grid gap-1 text-xs text-muted-foreground min-w-[600px]"
              style={{ gridTemplateColumns: "repeat(24, minmax(0, 1fr))" }}
            >
              {Array.from({ length: 24 }, (_, i) => (
                <div key={i} className="text-center">
                  {i.toString().padStart(2, "0")}
                </div>
              ))}
            </div>

            {/* Heatmap blocks */}
            <div
              className="grid gap-1 min-w-[600px] flex-1"
              style={{ gridTemplateColumns: "repeat(24, minmax(0, 1fr))" }}
            >
              {hourlyData.map((hourData) => (
                <div
                  key={hourData.hour}
                  className={`
                    aspect-square rounded cursor-pointer transition-all duration-200
                    hover:scale-110 hover:z-10 relative
                    ${getIntensityColor(hourData.activityTime)}
                    ${
                      hourData.activityTime > 0
                        ? "hover:ring-2 hover:ring-blue-500"
                        : ""
                    }
                  `}
                  onClick={() =>
                    onHourClick(hourData.hour, hourData.activities)
                  }
                  title={`${hourData.hour}:00 - ${formatDuration(
                    hourData.activityTime
                  )} active`}
                />
              ))}
            </div>

            {/* Legend and Summary */}
            <div className="flex items-center justify-between text-xs text-muted-foreground mt-2">
              <span>Less</span>
              <div className="flex items-center space-x-1">
                <div className="w-2 h-2 bg-gray-100 dark:bg-gray-800 rounded-sm" />
                <div className="w-2 h-2 bg-green-200 rounded-sm" />
                <div className="w-2 h-2 bg-green-300 rounded-sm" />
                <div className="w-2 h-2 bg-green-400 rounded-sm" />
                <div className="w-2 h-2 bg-green-500 rounded-sm" />
                <div className="w-2 h-2 bg-green-600 rounded-sm" />
                <span className="ml-2">More</span>
              </div>
              <span>
                {hourlyData.filter((h) => h.activityTime > 0).length} active
                hours
              </span>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
