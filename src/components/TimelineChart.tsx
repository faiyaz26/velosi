import { useMemo, useState } from "react";
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  ResponsiveContainer,
  Cell,
  Tooltip,
} from "recharts";
import {
  parseISO,
  format,
  isValid,
  isSameDay,
  startOfDay,
  eachDayOfInterval,
} from "date-fns";
import { ErrorBoundary } from "@/components/ui/ErrorBoundary";
import { useCategoryService } from "../hooks/useCategoryService";

interface ActivityEntry {
  id: string;
  start_time: string; // ISO
  end_time?: string; // ISO
  app_name: string;
  app_bundle_id?: string;
  window_title: string;
  url?: string;
  category: any;
}

interface DateRange {
  startDate: Date;
  endDate: Date;
}

interface TimelineChartProps {
  activities: ActivityEntry[];
  onActivityClick?: (activity: ActivityEntry) => void;
  isMultiDay?: boolean;
  dateRange?: DateRange;
}

// Small helper to color categories consistently using category service
const getCategoryColor = (
  cat: any,
  isInitialized: boolean,
  categoryService: any
): string => {
  const key =
    typeof cat === "string" ? cat : Object.keys(cat || {})[0] || "Unknown";

  if (isInitialized && categoryService) {
    const categoryInfo = categoryService.getCategoryById(key.toLowerCase());
    if (categoryInfo) {
      return categoryInfo.color;
    }
  }

  // Fallback colors if category service hasn't loaded
  switch (key) {
    case "Productive":
      return "#22d3ee"; // cyan
    case "Development":
      return "#a78bfa"; // violet
    case "Communication":
      return "#60a5fa"; // blue
    case "Social":
      return "#f472b6"; // pink
    case "Entertainment":
      return "#34d399"; // emerald
    default:
      return "#94a3b8"; // slate
  }
};

export function TimelineChart({
  activities,
  onActivityClick,
  isMultiDay = false,
  dateRange,
}: TimelineChartProps) {
  const [
    selectedActivity,
    setSelectedActivity,
  ] = useState<ActivityEntry | null>(null);
  const { isInitialized, categoryService } = useCategoryService();

  const safeActivities = (activities || []).filter((a) => {
    const s = parseISO(a.start_time);
    const e = a.end_time ? parseISO(a.end_time) : new Date();
    return isValid(s) && isValid(e) && s.getTime() <= e.getTime();
  });

  // Group activities by hour for single day or by day for multi-day
  const timelineData = useMemo(() => {
    if (isMultiDay && dateRange) {
      // Multi-day view: group by days
      const days = eachDayOfInterval({
        start: startOfDay(dateRange.startDate),
        end: startOfDay(dateRange.endDate),
      });

      const dailyData = days.map((day) => {
        const dayActivities = safeActivities.filter((activity) =>
          isSameDay(parseISO(activity.start_time), day)
        );

        const totalDuration = dayActivities.reduce((total, activity) => {
          const startTime = parseISO(activity.start_time);
          const endTime = activity.end_time
            ? parseISO(activity.end_time)
            : new Date();
          return total + (endTime.getTime() - startTime.getTime()) / 1000 / 60; // minutes
        }, 0);

        const dominantColor =
          dayActivities.length > 0
            ? getCategoryColor(
                dayActivities[0].category,
                isInitialized,
                categoryService
              )
            : "#374151";

        return {
          day: format(day, "MMM d"),
          date: day,
          activities: dayActivities,
          totalDuration,
          color: dominantColor,
        };
      });

      return dailyData;
    } else {
      // Single day view: group by hours
      const hourlyData: Record<
        number,
        {
          hour: number;
          activities: ActivityEntry[];
          totalDuration: number;
          color: string;
        }
      > = {};

      for (let hour = 0; hour < 24; hour++) {
        hourlyData[hour] = {
          hour,
          activities: [],
          totalDuration: 0,
          color: "#374151", // default gray
        };
      }

      safeActivities.forEach((activity) => {
        const startTime = parseISO(activity.start_time);
        const endTime = activity.end_time
          ? parseISO(activity.end_time)
          : new Date();
        const hour = startTime.getHours();
        const duration = (endTime.getTime() - startTime.getTime()) / 1000 / 60; // minutes

        if (hourlyData[hour]) {
          hourlyData[hour].activities.push(activity);
          hourlyData[hour].totalDuration += duration;
          hourlyData[hour].color = getCategoryColor(
            activity.category,
            isInitialized,
            categoryService
          );
        }
      });

      return Object.values(hourlyData);
    }
  }, [safeActivities, isMultiDay, dateRange, isInitialized, categoryService]);

  const handleBarClick = (data: any) => {
    if (data.activities && data.activities.length > 0) {
      const activity = data.activities[0]; // Take first activity for that hour
      setSelectedActivity(activity);
      if (onActivityClick) {
        onActivityClick(activity);
      }
    }
  };

  if (safeActivities.length === 0) {
    return (
      <div className="h-[190px] w-full rounded-md border border-dashed border-slate-700/60 text-slate-400 text-sm flex items-center justify-center">
        No activity for this date
      </div>
    );
  }

  return (
    <ErrorBoundary>
      <div className="w-full space-y-4">
        {/* Main Timeline Chart */}
        <div className="h-32 w-full">
          <ResponsiveContainer width="100%" height="100%">
            <BarChart
              data={timelineData}
              margin={{ top: 10, right: 10, left: 10, bottom: 10 }}
            >
              <XAxis
                dataKey={isMultiDay ? "day" : "hour"}
                axisLine={false}
                tickLine={false}
                tick={{ fontSize: 12, fill: "#94a3b8" }}
                interval={0}
              />
              <YAxis hide />
              <Tooltip
                content={({ active, payload }) => {
                  if (active && payload && payload[0]) {
                    const data = payload[0].payload;
                    const activityCount = data.activities.length;
                    const duration = Math.round(data.totalDuration);
                    const hours = Math.floor(duration / 60);
                    const minutes = duration % 60;

                    const durationText =
                      hours > 0 ? `${hours}h ${minutes}m` : `${minutes}m`;

                    return (
                      <div className="bg-slate-800 border border-slate-600 rounded-md p-2 text-sm text-slate-200">
                        <div className="font-medium">
                          {isMultiDay ? data.day : `${data.hour}:00`}
                        </div>
                        <div>{activityCount} activities</div>
                        <div>{durationText}</div>
                      </div>
                    );
                  }
                  return null;
                }}
              />
              <Bar
                dataKey="totalDuration"
                onClick={handleBarClick}
                cursor="pointer"
              >
                {timelineData.map((entry, index) => (
                  <Cell
                    key={`cell-${index}`}
                    fill={entry.totalDuration > 0 ? entry.color : "#374151"}
                    opacity={entry.totalDuration > 0 ? 0.8 : 0.3}
                  />
                ))}
              </Bar>
            </BarChart>
          </ResponsiveContainer>
        </div>

        {/* Category Band */}
        <div className="h-6 w-full flex rounded-sm overflow-hidden">
          {safeActivities.map((activity) => {
            const startTime = parseISO(activity.start_time);
            const endTime = activity.end_time
              ? parseISO(activity.end_time)
              : new Date();
            const duration =
              (endTime.getTime() - startTime.getTime()) / 1000 / 60; // minutes
            const totalDuration = safeActivities.reduce((total, a) => {
              const s = parseISO(a.start_time);
              const e = a.end_time ? parseISO(a.end_time) : new Date();
              return total + (e.getTime() - s.getTime()) / 1000 / 60;
            }, 0);

            const widthPercentage = (duration / totalDuration) * 100;

            return (
              <div
                key={activity.id}
                className="h-full cursor-pointer hover:opacity-80 transition-opacity"
                style={{
                  width: `${widthPercentage}%`,
                  backgroundColor: getCategoryColor(
                    activity.category,
                    isInitialized,
                    categoryService
                  ),
                }}
                onClick={() => handleBarClick({ activities: [activity] })}
                title={`${activity.app_name} - ${Math.round(duration)}m`}
              />
            );
          })}
        </div>
      </div>
    </ErrorBoundary>
  );
}
