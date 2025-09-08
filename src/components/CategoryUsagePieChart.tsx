import { useMemo } from "react";
import {
  PieChart,
  Pie,
  Cell,
  ResponsiveContainer,
  Tooltip,
  Legend,
} from "recharts";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Tags } from "lucide-react";
import { useCategoryService } from "@/hooks/useCategoryService";

// Fallback colors for when category service isn't available
const FALLBACK_COLORS = [
  "#3B82F6", // Blue
  "#EF4444", // Red
  "#10B981", // Green
  "#F59E0B", // Yellow
  "#8B5CF6", // Purple
  "#06B6D4", // Cyan
  "#F97316", // Orange
  "#84CC16", // Lime
  "#EC4899", // Pink
  "#6B7280", // Gray
];

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

interface ActivitySummary {
  categories: Array<{
    category: any;
    total_duration?: number; // Dashboard format
    duration_seconds?: number; // ActivityLog format
    count?: number;
  }>;
  total_duration?: number;
  activity_count?: number;
}

interface CategoryUsagePieChartProps {
  activities: ActivityEntry[];
  activitySummary?: ActivitySummary | null;
  getCategoryName?: (category: any) => string;
  getCategoryColor?: (category: any) => string;
}

function getDefaultCategoryName(
  category: any,
  categoryService: any,
  isInitialized: boolean
): string {
  let categoryKey = "unknown";

  // Handle both string and object category formats
  if (typeof category === "string") {
    categoryKey = category;
  } else if (typeof category === "object" && category) {
    const keys = Object.keys(category);
    categoryKey = keys.length > 0 ? keys[0] : "unknown";
  }

  if (isInitialized && categoryService) {
    const categoryInfo = categoryService.getCategoryById(
      categoryKey.toLowerCase()
    );
    if (categoryInfo) {
      return categoryInfo.name;
    }
  }

  // Fallback to the original enum name
  return categoryKey;
}

function getDefaultCategoryColor(
  category: any,
  categoryService: any,
  isInitialized: boolean,
  categoryIndex: number = 0
): string {
  let categoryKey = "unknown";

  // Handle both string and object category formats
  if (typeof category === "string") {
    categoryKey = category;
  } else if (typeof category === "object" && category) {
    const keys = Object.keys(category);
    categoryKey = keys.length > 0 ? keys[0] : "unknown";
  }

  if (isInitialized && categoryService) {
    const categoryInfo = categoryService.getCategoryById(
      categoryKey.toLowerCase()
    );
    if (categoryInfo) {
      return categoryInfo.color;
    }
  }

  // Use colorful fallback colors instead of gray
  return FALLBACK_COLORS[categoryIndex % FALLBACK_COLORS.length];
}

export const CategoryUsagePieChart = ({
  activities,
  activitySummary,
  getCategoryName,
  getCategoryColor,
}: CategoryUsagePieChartProps) => {
  const { isInitialized, categoryService } = useCategoryService();

  console.log("CategoryUsagePieChart render:", {
    activitiesCount: activities.length,
    activitySummary,
    isInitialized,
  });

  const pieData = useMemo(() => {
    console.log("Computing pieData...", {
      activitySummary,
      activitiesCount: activities.length,
    });

    // Use activity summary data if available (preferred method for accuracy)
    if (activitySummary?.categories) {
      console.log(
        "Using activity summary categories:",
        activitySummary.categories
      );
      const result = activitySummary.categories.map((cat, index) => {
        // Handle both Dashboard format (total_duration) and ActivityLog format (duration_seconds)
        const durationSeconds = cat.total_duration || cat.duration_seconds || 0;

        return {
          name: getCategoryName
            ? getCategoryName(cat.category)
            : getDefaultCategoryName(
                cat.category,
                categoryService,
                isInitialized
              ),
          value: Math.round(durationSeconds / 60), // Convert seconds to minutes
          fill: getCategoryColor
            ? getCategoryColor(cat.category)
            : getDefaultCategoryColor(
                cat.category,
                categoryService,
                isInitialized,
                index
              ),
        };
      });
      console.log("Activity summary result:", result);
      return result;
    }

    // Fallback to manual processing for backward compatibility
    console.log(
      "Using fallback manual processing for",
      activities.length,
      "activities"
    );
    const categoryMap = new Map<
      string,
      { duration: number; color: string; index: number }
    >();
    let categoryIndex = 0;

    activities.forEach((activity) => {
      if (activity.end_time) {
        const duration =
          new Date(activity.end_time).getTime() -
          new Date(activity.start_time).getTime();
        const minutes = Math.round(duration / (1000 * 60));

        const categoryName = getCategoryName
          ? getCategoryName(activity.category)
          : getDefaultCategoryName(
              activity.category,
              categoryService,
              isInitialized
            );

        if (categoryMap.has(categoryName)) {
          categoryMap.get(categoryName)!.duration += minutes;
        } else {
          const categoryColor = getCategoryColor
            ? getCategoryColor(activity.category)
            : getDefaultCategoryColor(
                activity.category,
                categoryService,
                isInitialized,
                categoryIndex
              );
          categoryMap.set(categoryName, {
            duration: minutes,
            color: categoryColor,
            index: categoryIndex,
          });
          categoryIndex++;
        }
      }
    });

    const result = Array.from(categoryMap.entries())
      .map(([name, { duration, color }]) => ({
        name,
        value: duration,
        fill: color,
      }))
      .sort((a, b) => b.value - a.value);
    console.log("Manual processing result:", result);
    return result;
  }, [
    activities,
    activitySummary,
    getCategoryName,
    getCategoryColor,
    categoryService,
    isInitialized,
  ]);

  const totalMinutes = useMemo(() => {
    return pieData.reduce((sum, item) => sum + item.value, 0);
  }, [pieData]);

  if (pieData.length === 0) {
    return (
      <Card>
        <CardHeader className="flex flex-row items-center space-y-0 pb-2">
          <CardTitle className="text-base font-medium">
            Category Usage
          </CardTitle>
          <Tags className="h-4 w-4 text-muted-foreground ml-auto" />
        </CardHeader>
        <CardContent>
          <div className="flex h-40 items-center justify-center text-muted-foreground">
            No activities recorded for this date
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader className="flex flex-row items-center space-y-0 pb-2">
        <CardTitle className="text-base font-medium">Category Usage</CardTitle>
        <Tags className="h-4 w-4 text-muted-foreground ml-auto" />
      </CardHeader>
      <CardContent>
        <CardDescription className="mb-4">
          Total time tracked: {Math.floor(totalMinutes / 60)}h{" "}
          {totalMinutes % 60}m across {pieData.length} categories
        </CardDescription>
        <ResponsiveContainer width="100%" height={300}>
          <PieChart>
            <Pie
              data={pieData}
              cx="50%"
              cy="50%"
              labelLine={false}
              label={({ name, value }) => {
                const percentage = (
                  ((value || 0) / totalMinutes) *
                  100
                ).toFixed(1);
                return `${name}: ${percentage}%`;
              }}
              outerRadius={80}
              fill="#8884d8"
              dataKey="value"
            >
              {pieData.map((entry, index) => (
                <Cell key={`cell-${index}`} fill={entry.fill} />
              ))}
            </Pie>
            <Tooltip
              formatter={(value: number) => [
                `${Math.floor(value / 60)}h ${value % 60}m`,
                "Time",
              ]}
            />
            <Legend />
          </PieChart>
        </ResponsiveContainer>
      </CardContent>
    </Card>
  );
};
