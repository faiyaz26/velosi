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
  CartesianGrid,
  Tooltip,
} from "recharts";
import { Clock, TrendingUp, Zap } from "lucide-react";
import { format } from "date-fns";

export interface ActivityCategory {
  Productive?: null;
  Social?: null;
  Entertainment?: null;
  Development?: null;
  Communication?: null;
  Unknown?: null;
}

export interface CategorySummary {
  category: ActivityCategory;
  duration_seconds: number;
  percentage: number;
}

export interface AppSummary {
  app_name: string;
  duration_seconds: number;
  percentage: number;
}

export interface ActivitySummary {
  date: string;
  total_active_time: number;
  categories: CategorySummary[];
  top_apps: AppSummary[];
}

interface ActivityDashboardProps {
  summary: ActivitySummary | null;
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

export function ActivityDashboard({ summary }: ActivityDashboardProps) {
  if (!summary) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Activity Dashboard</CardTitle>
          <CardDescription>No activity data available</CardDescription>
        </CardHeader>
        <CardContent>
          <p className="text-muted-foreground">
            Start tracking to see your activity data here.
          </p>
        </CardContent>
      </Card>
    );
  }

  const pieData = summary.categories.map((cat) => ({
    name: getCategoryName(cat.category),
    value: cat.duration_seconds,
    percentage: cat.percentage,
    color:
      CATEGORY_COLORS[
        getCategoryName(cat.category) as keyof typeof CATEGORY_COLORS
      ] || CATEGORY_COLORS.Unknown,
  }));

  const barData = summary.top_apps.slice(0, 5).map((app) => ({
    name:
      app.app_name.length > 15
        ? app.app_name.substring(0, 15) + "..."
        : app.app_name,
    duration: app.duration_seconds / 60, // Convert to minutes
    percentage: app.percentage,
  }));

  return (
    <div className="space-y-6">
      {/* Stats Overview */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <Card>
          <CardContent className="flex items-center justify-between p-6">
            <div>
              <p className="text-2xl font-bold">
                {formatDuration(summary.total_active_time)}
              </p>
              <p className="text-muted-foreground">Total Active Time</p>
            </div>
            <Clock className="h-8 w-8 text-blue-500" />
          </CardContent>
        </Card>

        <Card>
          <CardContent className="flex items-center justify-between p-6">
            <div>
              <p className="text-2xl font-bold">{summary.top_apps.length}</p>
              <p className="text-muted-foreground">Apps Used</p>
            </div>
            <Zap className="h-8 w-8 text-green-500" />
          </CardContent>
        </Card>

        <Card>
          <CardContent className="flex items-center justify-between p-6">
            <div>
              <p className="text-2xl font-bold">{summary.categories.length}</p>
              <p className="text-muted-foreground">Categories</p>
            </div>
            <TrendingUp className="h-8 w-8 text-purple-500" />
          </CardContent>
        </Card>
      </div>

      {/* Charts */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Category Breakdown */}
        <Card>
          <CardHeader>
            <CardTitle>Time by Category</CardTitle>
            <CardDescription>
              How you spent your time on{" "}
              {format(new Date(summary.date), "MMMM d, yyyy")}
            </CardDescription>
          </CardHeader>
          <CardContent>
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
          </CardContent>
        </Card>

        {/* Top Apps */}
        <Card>
          <CardHeader>
            <CardTitle>Top Applications</CardTitle>
            <CardDescription>Most used applications today</CardDescription>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={300}>
              <BarChart data={barData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis
                  dataKey="name"
                  tick={{ fontSize: 12 }}
                  angle={-45}
                  textAnchor="end"
                  height={80}
                />
                <YAxis tick={{ fontSize: 12 }} />
                <Tooltip
                  formatter={(value: number) => [
                    `${value.toFixed(1)} min`,
                    "Duration",
                  ]}
                />
                <Bar dataKey="duration" fill="#8884d8" />
              </BarChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>
      </div>

      {/* Detailed App List */}
      <Card>
        <CardHeader>
          <CardTitle>Detailed Application Usage</CardTitle>
          <CardDescription>
            Complete breakdown of your app usage
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-2">
            {summary.top_apps.map((app, index) => (
              <div
                key={index}
                className="flex items-center justify-between p-3 border rounded-lg"
              >
                <div className="flex-1">
                  <p className="font-medium">{app.app_name}</p>
                  <div className="w-full bg-gray-200 rounded-full h-2 mt-1">
                    <div
                      className="bg-blue-600 h-2 rounded-full"
                      style={{ width: `${app.percentage}%` }}
                    />
                  </div>
                </div>
                <div className="text-right ml-4">
                  <p className="font-medium">
                    {formatDuration(app.duration_seconds)}
                  </p>
                  <p className="text-sm text-muted-foreground">
                    {app.percentage.toFixed(1)}%
                  </p>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
