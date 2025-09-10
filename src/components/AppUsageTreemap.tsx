import { useMemo } from "react";
import { Treemap, ResponsiveContainer } from "recharts";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Monitor } from "lucide-react";
import { ErrorBoundary } from "@/components/ui/ErrorBoundary";
import { useCategoryService } from "@/hooks/useCategoryService";

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

interface AppUsageTreemapProps {
  activities: ActivityEntry[];
}

const COLORS = [
  "#3b82f6", // blue-500
  "#10b981", // emerald-500
  "#f59e0b", // amber-500
  "#ef4444", // red-500
  "#8b5cf6", // violet-500
  "#06b6d4", // cyan-500
  "#84cc16", // lime-500
  "#f97316", // orange-500
  "#ec4899", // pink-500
  "#6366f1", // indigo-500
];

export function AppUsageTreemap({ activities }: AppUsageTreemapProps) {
  const { isInitialized, categoryService } = useCategoryService();

  const treemapData = useMemo(() => {
    const appUsage = new Map<string, { duration: number; category: string }>();

    activities.forEach((activity) => {
      const start = new Date(activity.start_time);
      const end = activity.end_time ? new Date(activity.end_time) : new Date();
      const duration = end.getTime() - start.getTime();

      const existing = appUsage.get(activity.app_name);

      // Handle both string and object category formats
      let categoryName = "Unknown";
      if (typeof activity.category === "string") {
        categoryName = activity.category;
      } else if (typeof activity.category === "object" && activity.category) {
        categoryName = Object.keys(activity.category)[0] || "Unknown";
      }

      if (existing) {
        existing.duration += duration;
      } else {
        appUsage.set(activity.app_name, {
          duration,
          category: categoryName,
        });
      }
    });

    return Array.from(appUsage.entries())
      .map(([appName, data], index) => {
        const totalMinutes = Math.floor(data.duration / (1000 * 60));
        const hours = Math.floor(totalMinutes / 60);
        const minutes = totalMinutes % 60;

        // Get color from category service
        const categoryInfo = isInitialized
          ? categoryService.getCategoryById(data.category.toLowerCase())
          : null;
        const color = categoryInfo?.color || COLORS[index % COLORS.length];

        return {
          name: appName,
          value: totalMinutes,
          fill: color,
          category: data.category,
          hours,
          minutes,
        };
      })
      .filter((item) => item.value > 0)
      .sort((a, b) => b.value - a.value);
  }, [activities, isInitialized, categoryService]);

  // treemapData prepared for rendering

  if (treemapData.length === 0) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            App Usage Treemap
          </CardTitle>
          <CardDescription>
            Visual breakdown of time spent in different applications
          </CardDescription>
        </CardHeader>
        <CardContent className="text-center p-8">
          <Monitor className="h-12 w-12 mx-auto mb-4 text-muted-foreground" />
          <p className="text-muted-foreground">No app usage data available</p>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          App Usage Treemap
        </CardTitle>
        <CardDescription>
          Visual breakdown of time spent in different applications
        </CardDescription>
      </CardHeader>
      <CardContent>
        <ErrorBoundary
          fallback={
            <div className="p-4 text-center text-muted-foreground">
              Unable to render treemap
            </div>
          }
        >
          <div className="h-80 w-full">
            <ResponsiveContainer width="100%" height="100%">
              <Treemap
                data={treemapData}
                dataKey="value"
                aspectRatio={4 / 3}
                stroke="hsl(var(--border))"
                fill="#8884d8"
              />
            </ResponsiveContainer>
          </div>
        </ErrorBoundary>

        {/* Legend */}
        <div className="mt-4 flex flex-wrap gap-2">
          {treemapData.slice(0, 10).map((item) => (
            <div key={item.name} className="flex items-center gap-2 text-sm">
              <div
                className="w-3 h-3 rounded"
                style={{ backgroundColor: item.fill }}
              />
              <span className="truncate max-w-[120px]">{item.name}</span>
              <span className="text-muted-foreground">
                (
                {item.hours > 0
                  ? `${item.hours}h ${item.minutes}m`
                  : `${item.minutes}m`}
                )
              </span>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  );
}
