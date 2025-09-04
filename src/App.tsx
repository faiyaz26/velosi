import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { TrackingControls } from "@/components/TrackingControls";
import {
  ActivityDashboard,
  ActivitySummary,
} from "@/components/ActivityDashboard";
import { DateSelector } from "@/components/DateSelector";
import { TimelineChart } from "@/components/TimelineChart";
import { ThemeToggle } from "@/components/ThemeToggle";
import { Button } from "@/components/ui/button";
import { format } from "date-fns";
import { RefreshCw, Minimize2, X } from "lucide-react";

function App() {
  const [selectedDate, setSelectedDate] = useState(new Date());
  const [
    activitySummary,
    setActivitySummary,
  ] = useState<ActivitySummary | null>(null);
  const [loading, setLoading] = useState(false);
  const [lastUpdated, setLastUpdated] = useState<Date | null>(null);

  const handleMinimize = async () => {
    try {
      await invoke("hide_window");
    } catch (error) {
      console.error("Failed to minimize window:", error);
    }
  };

  const handleClose = async () => {
    try {
      await invoke("hide_window");
    } catch (error) {
      console.error("Failed to close app:", error);
    }
  };

  useEffect(() => {
    loadActivitySummary(selectedDate);
  }, [selectedDate]);

  // Auto-refresh effect for current date
  useEffect(() => {
    const isToday =
      format(selectedDate, "yyyy-MM-dd") === format(new Date(), "yyyy-MM-dd");

    if (isToday) {
      // Refresh every 30 seconds for today's data
      const interval = setInterval(() => {
        loadActivitySummary(selectedDate);
      }, 30000);

      return () => clearInterval(interval);
    }
  }, [selectedDate]);

  const loadActivitySummary = async (date: Date) => {
    setLoading(true);
    try {
      const dateString = format(date, "yyyy-MM-dd");
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

  const handleRefresh = () => {
    loadActivitySummary(selectedDate);
  };

  return (
    <div className="min-h-screen bg-background p-6">
      <div className="max-w-7xl mx-auto space-y-6">
        {/* Top bar */}
        <div className="flex items-center justify-between">
          <div>
            <p className="text-sm text-muted-foreground">
              {format(new Date(), "EEEE, MMMM d, yyyy")}
            </p>
            <h1 className="text-2xl font-semibold tracking-tight">
              Velosi Tracker
            </h1>
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
              <RefreshCw
                className={`h-4 w-4 ${loading ? "animate-spin" : ""}`}
              />
              Refresh
            </Button>
            <ThemeToggle />
            <Button
              onClick={handleMinimize}
              size="sm"
              variant="outline"
              className="flex items-center gap-1"
            >
              <Minimize2 className="h-4 w-4" />
            </Button>
            <Button
              onClick={handleClose}
              size="sm"
              variant="outline"
              className="flex items-center gap-1"
            >
              <X className="h-4 w-4" />
            </Button>
          </div>
        </div>

        {lastUpdated && (
          <p className="text-xs text-muted-foreground">
            Last updated {format(lastUpdated, "HH:mm:ss")}
          </p>
        )}

        {/* Timeline across top */}
        <TimelineChart minutes={30} />

        {/* Main grid below like the screenshot: left controls/activity, right analytics */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          <div className="space-y-6 lg:col-span-1">
            <TrackingControls />
          </div>
          <div className="lg:col-span-2">
            {loading ? (
              <div className="text-center p-8">
                <p className="text-muted-foreground">
                  Loading activity data...
                </p>
              </div>
            ) : (
              <ActivityDashboard summary={activitySummary} />
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
