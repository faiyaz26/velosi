import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { TrackingControls } from "@/components/TrackingControls";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  PieChart,
  Pie,
  Cell,
  ResponsiveContainer,
  BarChart,
  Bar,
  XAxis,
  YAxis,
  Tooltip,
} from "recharts";
import { Clock, TrendingUp, Zap } from "lucide-react";
import { format } from "date-fns";

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

const CATEGORY_COLORS = {
  Development: "#8884d8",
  Productive: "#82ca9d",
  Communication: "#ffc658",
  Social: "#ff7c7c",
  Entertainment: "#8dd1e1",
  Unknown: "#d084d0",
};

function formatDuration(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);

  if (hours > 0) {
    return `${hours}h ${minutes}m`;
  }
  return `${minutes}m`;
}

function getCategoryName(category: ActivityCategory): string {
  const keys = Object.keys(category);
  return keys.length > 0 ? keys[0] : "Unknown";
}

export function Dashboard() {
  const [
    activitySummary,
    setActivitySummary,
  ] = useState<ActivitySummary | null>(null);
  const [loading, setLoading] = useState(false);
  const [lastUpdated, setLastUpdated] = useState<Date | null>(null);

  useEffect(() => {
    loadTodaysActivitySummary();

    // Auto-refresh every 30 seconds for today's data
    const interval = setInterval(() => {
      loadTodaysActivitySummary();
    }, 30000);

    return () => clearInterval(interval);
  }, []);

  const loadTodaysActivitySummary = async () => {
    setLoading(true);
    try {
      const dateString = format(new Date(), "yyyy-MM-dd");
      const summary = await invoke<ActivitySummary>("get_activity_summary", {
        date: dateString,
      });
      setActivitySummary(summary);
      setLastUpdated(new Date());
    } catch (error) {
      console.error("Failed to load activity summary:", error);
      setActivitySummary(null);
    } finally {
      setLoading(false);
    }
  };

  const pieData =
    activitySummary?.categories.map((cat) => ({
      name: getCategoryName(cat.category),
      value: cat.duration_seconds,
      percentage: cat.percentage,
      color:
        CATEGORY_COLORS[
          getCategoryName(cat.category) as keyof typeof CATEGORY_COLORS
        ] || CATEGORY_COLORS.Unknown,
    })) || [];

  const barData =
    activitySummary?.top_apps.slice(0, 5).map((app) => ({
      name:
        app.app_name.length > 15
          ? app.app_name.substring(0, 15) + "..."
          : app.app_name,
      duration: app.duration_seconds / 60, // Convert to minutes
      percentage: app.percentage,
    })) || [];

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-semibold tracking-tight">Dashboard</h1>
        <p className="text-muted-foreground">
          {format(new Date(), "EEEE, MMMM d, yyyy")}
        </p>
        {lastUpdated && (
          <p className="text-xs text-muted-foreground mt-1">
            Last updated {format(lastUpdated, "HH:mm:ss")}
          </p>
        )}
      </div>

      {/* Current Tracking Status */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <TrackingControls />

        {/* Today's Active Time */}
        <Card>
          <CardContent className="flex items-center justify-between p-6">
            <div>
              <p className="text-3xl font-bold">
                {activitySummary
                  ? formatDuration(activitySummary.total_active_time)
                  : "0m"}
              </p>
              <p className="text-muted-foreground">Total Active Time Today</p>
            </div>
            <Clock className="h-12 w-12 text-blue-500" />
          </CardContent>
        </Card>
      </div>

      {/* Stats Overview */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <Card>
          <CardContent className="flex items-center justify-between p-6">
            <div>
              <p className="text-2xl font-bold">
                {activitySummary?.top_apps.length || 0}
              </p>
              <p className="text-muted-foreground">Apps Used Today</p>
            </div>
            <Zap className="h-8 w-8 text-green-500" />
          </CardContent>
        </Card>

        <Card>
          <CardContent className="flex items-center justify-between p-6">
            <div>
              <p className="text-2xl font-bold">
                {activitySummary?.categories.length || 0}
              </p>
              <p className="text-muted-foreground">Activity Categories</p>
            </div>
            <TrendingUp className="h-8 w-8 text-purple-500" />
          </CardContent>
        </Card>
      </div>

      {/* Charts */}
      {activitySummary && (
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {/* Category Breakdown */}
          <Card>
            <CardHeader>
              <CardTitle>Time by Category</CardTitle>
              <CardDescription>How you spent your time today</CardDescription>
            </CardHeader>
            <CardContent>
              {pieData.length > 0 ? (
                <ResponsiveContainer width="100%" height={300}>
                  <PieChart>
                    <Pie
                      data={pieData}
                      cx="50%"
                      cy="50%"
                      labelLine={false}
                      label={({ name, percentage }) =>
                        `${name} (${percentage.toFixed(1)}%)`
                      }
                      outerRadius={80}
                      fill="#8884d8"
                      dataKey="value"
                    >
                      {pieData.map((entry, index) => (
                        <Cell key={`cell-${index}`} fill={entry.color} />
                      ))}
                    </Pie>
                    <Tooltip
                      formatter={(value: number) => [
                        formatDuration(value),
                        "Duration",
                      ]}
                    />
                  </PieChart>
                </ResponsiveContainer>
              ) : (
                <div className="h-[300px] flex items-center justify-center text-muted-foreground">
                  No activity data yet
                </div>
              )}
            </CardContent>
          </Card>

          {/* Top Apps */}
          <Card>
            <CardHeader>
              <CardTitle>Top Applications</CardTitle>
              <CardDescription>Most used applications today</CardDescription>
            </CardHeader>
            <CardContent>
              {barData.length > 0 ? (
                <ResponsiveContainer width="100%" height={300}>
                  <BarChart data={barData}>
                    <XAxis dataKey="name" />
                    <YAxis />
                    <Tooltip
                      formatter={(value: number) => [
                        `${value.toFixed(1)} min`,
                        "Duration",
                      ]}
                    />
                    <Bar dataKey="duration" fill="#8884d8" />
                  </BarChart>
                </ResponsiveContainer>
              ) : (
                <div className="h-[300px] flex items-center justify-center text-muted-foreground">
                  No app usage data yet
                </div>
              )}
            </CardContent>
          </Card>
        </div>
      )}

      {loading && (
        <div className="text-center p-8">
          <p className="text-muted-foreground">Loading activity data...</p>
        </div>
      )}
    </div>
  );
}
