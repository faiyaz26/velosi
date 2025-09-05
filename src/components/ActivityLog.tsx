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
import { Input } from "@/components/ui/input";
import { DateSelector } from "@/components/DateSelector";
import { TimelineChart } from "@/components/TimelineChart";
import { Activity, Clock, Calendar, RefreshCw } from "lucide-react";
import { format, subDays } from "date-fns";

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

export function ActivityLog() {
  const [selectedDate, setSelectedDate] = useState(new Date());
  const [activities, setActivities] = useState<ActivityEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [timelineMinutes, setTimelineMinutes] = useState(30);

  useEffect(() => {
    loadActivities(selectedDate);
  }, [selectedDate]);

  const loadActivities = async (date: Date) => {
    setLoading(true);
    try {
      const dateString = format(date, "yyyy-MM-dd");
      const result = await invoke<ActivityEntry[]>("get_activities_by_date", {
        date: dateString,
      });
      setActivities(result);
    } catch (error) {
      console.error("Failed to load activities:", error);
      setActivities([]);
    } finally {
      setLoading(false);
    }
  };

  const handleRefresh = () => {
    loadActivities(selectedDate);
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

  const getCategoryColor = (category: any): string => {
    const categoryName = Object.keys(category)[0] || "Unknown";
    const colors: { [key: string]: string } = {
      Development: "bg-blue-500",
      Productive: "bg-green-500",
      Communication: "bg-yellow-500",
      Social: "bg-red-500",
      Entertainment: "bg-purple-500",
      Unknown: "bg-gray-500",
    };
    return colors[categoryName] || colors.Unknown;
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">
            Activity Log
          </h1>
          <p className="text-muted-foreground">
            View detailed logs of your tracked activities
          </p>
        </div>
        <div className="flex items-center gap-2">
          <DateSelector
            selectedDate={selectedDate}
            onDateChange={setSelectedDate}
          />
          <Button
            onClick={handleRefresh}
            disabled={loading}
            size="sm"
            variant="outline"
            className="flex items-center gap-2"
          >
            <RefreshCw className={`h-4 w-4 ${loading ? "animate-spin" : ""}`} />
            Refresh
          </Button>
        </div>
      </div>

      {/* Timeline Controls */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Calendar className="h-5 w-5" />
            Activity Timeline
          </CardTitle>
          <CardDescription>
            Visual representation of your activities for{" "}
            {format(selectedDate, "MMMM d, yyyy")}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-4 mb-4">
            <div className="flex items-center gap-2">
              <label htmlFor="timeline-minutes" className="text-sm font-medium">
                Timeline Duration (minutes):
              </label>
              <Input
                id="timeline-minutes"
                type="number"
                value={timelineMinutes}
                onChange={(e) => setTimelineMinutes(Number(e.target.value))}
                className="w-20"
                min="10"
                max="120"
              />
            </div>
            <Button
              size="sm"
              variant="outline"
              onClick={() => setTimelineMinutes(30)}
            >
              30m
            </Button>
            <Button
              size="sm"
              variant="outline"
              onClick={() => setTimelineMinutes(60)}
            >
              1h
            </Button>
            <Button
              size="sm"
              variant="outline"
              onClick={() => setTimelineMinutes(120)}
            >
              2h
            </Button>
          </div>
          <TimelineChart minutes={timelineMinutes} />
        </CardContent>
      </Card>

      {/* Activity Entries */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Activity className="h-5 w-5" />
            Activity Entries
          </CardTitle>
          <CardDescription>
            Detailed list of activities for{" "}
            {format(selectedDate, "MMMM d, yyyy")}
          </CardDescription>
        </CardHeader>
        <CardContent>
          {loading ? (
            <div className="text-center p-8">
              <p className="text-muted-foreground">Loading activities...</p>
            </div>
          ) : activities.length > 0 ? (
            <div className="space-y-4">
              {activities.map((activity) => (
                <div
                  key={activity.id}
                  className="flex items-center gap-3 p-3 border rounded-lg hover:bg-muted/50 transition-colors"
                >
                  <div
                    className={`h-3 w-3 rounded-full ${getCategoryColor(
                      activity.category
                    )}`}
                  ></div>
                  <div className="flex-1">
                    <p className="font-medium">{activity.app_name}</p>
                    <p className="text-sm text-muted-foreground">
                      {activity.window_title}
                    </p>
                    {activity.url && (
                      <p className="text-xs text-blue-600 truncate">
                        {activity.url}
                      </p>
                    )}
                  </div>
                  <div className="text-right">
                    <p className="text-sm font-medium">
                      {formatDuration(activity.start_time, activity.end_time)}
                    </p>
                    <p className="text-xs text-muted-foreground">
                      <Clock className="inline h-3 w-3 mr-1" />
                      {format(new Date(activity.start_time), "HH:mm")}
                      {activity.end_time && (
                        <> - {format(new Date(activity.end_time), "HH:mm")}</>
                      )}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <div className="text-center p-8">
              <p className="text-muted-foreground">
                No activities found for this date.
              </p>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Quick Date Navigation */}
      <Card>
        <CardHeader>
          <CardTitle>Quick Navigation</CardTitle>
          <CardDescription>Jump to recent dates</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex gap-2 flex-wrap">
            <Button
              size="sm"
              variant="outline"
              onClick={() => setSelectedDate(new Date())}
            >
              Today
            </Button>
            <Button
              size="sm"
              variant="outline"
              onClick={() => setSelectedDate(subDays(new Date(), 1))}
            >
              Yesterday
            </Button>
            <Button
              size="sm"
              variant="outline"
              onClick={() => setSelectedDate(subDays(new Date(), 7))}
            >
              Last Week
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
