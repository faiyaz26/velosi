import { useMemo } from "react";
import { Treemap, ResponsiveContainer } from "recharts";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Tags } from "lucide-react";
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

interface CategoryUsageTreemapProps {
  activities: ActivityEntry[];
}

interface TreemapData {
  name: string;
  value: number;
  fill: string;
  hours: number;
  minutes: number;
}

const FALLBACK_COLORS = [
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

export function CategoryUsageTreemap({
  activities,
}: CategoryUsageTreemapProps) {
  const { isInitialized, categoryService } = useCategoryService();

  const treemapData = useMemo(() => {
    console.log("=== CategoryUsageTreemap Debug ===");
    console.log("Activities received:", activities.length);

    if (activities.length === 0) {
      console.log("No activities to process");
      return [];
    }

    console.log("Sample activity:", activities[0]);
    console.log("CategoryService initialized:", isInitialized);

    const categoryUsage = new Map<string, number>();

    activities.forEach((activity, index) => {
      try {
        const start = new Date(activity.start_time);
        const end = activity.end_time
          ? new Date(activity.end_time)
          : new Date();
        const duration = Math.max(0, end.getTime() - start.getTime());

        // Skip activities with 0 or negative duration
        if (duration <= 0) {
          console.log(
            `Skipping activity ${index + 1} with duration ${duration}ms`
          );
          return;
        }

        let categoryId = "unknown";

        // Handle different category formats
        if (typeof activity.category === "string") {
          // Remove quotes if present (JSON string format)
          categoryId = activity.category.replace(/"/g, "").toLowerCase();
        } else if (typeof activity.category === "object" && activity.category) {
          const keys = Object.keys(activity.category);
          categoryId = keys.length > 0 ? keys[0].toLowerCase() : "unknown";
        }

        // Log category extraction for debugging
        if (index < 3) {
          console.log(
            `Raw category: ${JSON.stringify(
              activity.category
            )}, Extracted: ${categoryId}`
          );
        }

        // Use category service if available
        if (isInitialized && categoryService) {
          try {
            const categoryInfo = categoryService.getCategoryByAppName(
              activity.app_name
            );
            if (categoryInfo && categoryInfo.id) {
              categoryId = categoryInfo.id;
            }
          } catch (e) {
            console.log("Error getting category from service:", e);
          }
        }

        const existing = categoryUsage.get(categoryId) || 0;
        categoryUsage.set(categoryId, existing + duration);

        if (index < 3) {
          // Log first 3 activities
          console.log(
            `Activity ${index + 1}: App="${
              activity.app_name
            }", Category="${categoryId}", Duration=${Math.floor(
              duration / 1000 / 60
            )}min`
          );
        }
      } catch (error) {
        console.error(`Error processing activity ${index + 1}:`, error);
      }
    });

    console.log(
      "Category usage map:",
      Array.from(categoryUsage.entries()).map(([cat, dur]) => [
        cat,
        Math.floor(dur / 1000 / 60),
      ])
    );

    const result = Array.from(categoryUsage.entries())
      .map(([categoryId, duration], index) => {
        const totalMinutes = Math.floor(duration / (1000 * 60));
        const hours = Math.floor(totalMinutes / 60);
        const minutes = totalMinutes % 60;

        let displayName =
          categoryId.charAt(0).toUpperCase() + categoryId.slice(1);
        let color = FALLBACK_COLORS[index % FALLBACK_COLORS.length];

        // Try to get better display name and color from category service
        if (isInitialized && categoryService) {
          try {
            const categoryInfo = categoryService.getCategoryById(categoryId);
            if (categoryInfo) {
              displayName = categoryInfo.name || displayName;
              color = categoryInfo.color || color;
            }
          } catch (e) {
            console.log("Error getting category info:", e);
          }
        }

        return {
          name: displayName,
          value: totalMinutes,
          fill: color,
          hours,
          minutes,
        };
      })
      .filter((item) => item.value > 0)
      .sort((a, b) => b.value - a.value);

    console.log("Final treemap data:", result);
    return result;
  }, [activities, isInitialized, categoryService]);

  if (treemapData.length === 0) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Tags className="h-5 w-5" />
            Category Usage
          </CardTitle>
          <CardDescription>
            Time spent across different activity categories
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="h-[400px] flex items-center justify-center text-muted-foreground">
            No category data available
          </div>
        </CardContent>
      </Card>
    );
  }

  const CustomContent = (props: any) => {
    const { depth, x, y, width, height, payload } = props;

    if (depth !== 1) return null;

    const data = payload as TreemapData;
    if (!data) return null;

    const fontSize = Math.min(width / 8, height / 6, 16);
    const shouldShowText = width > 60 && height > 40;

    return (
      <g>
        <rect
          x={x}
          y={y}
          width={width}
          height={height}
          style={{
            fill: data.fill,
            stroke: "#fff",
            strokeWidth: 2,
            strokeOpacity: 1,
          }}
        />
        {shouldShowText && (
          <>
            <text
              x={x + width / 2}
              y={y + height / 2 - 8}
              textAnchor="middle"
              fill="#fff"
              fontSize={fontSize}
              fontWeight="bold"
            >
              {data.name}
            </text>
            <text
              x={x + width / 2}
              y={y + height / 2 + 8}
              textAnchor="middle"
              fill="#fff"
              fontSize={Math.max(fontSize - 2, 10)}
              opacity={0.9}
            >
              {data.hours > 0
                ? `${data.hours}h ${data.minutes}m`
                : `${data.minutes}m`}
            </text>
          </>
        )}
      </g>
    );
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Tags className="h-5 w-5" />
          Category Usage
        </CardTitle>
        <CardDescription>
          Time spent across different activity categories
        </CardDescription>
      </CardHeader>
      <CardContent>
        <ResponsiveContainer width="100%" height={400}>
          <Treemap
            data={treemapData}
            dataKey="value"
            aspectRatio={4 / 3}
            stroke="#fff"
            fill="#8884d8"
            content={<CustomContent />}
          />
        </ResponsiveContainer>
      </CardContent>
    </Card>
  );
}
