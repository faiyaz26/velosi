import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { TrackingControls } from "@/components/TrackingControls";
import {
  ActivityDashboard,
  ActivitySummary,
} from "@/components/ActivityDashboard";
import { DateSelector } from "@/components/DateSelector";
import { TimelineChart } from "@/components/TimelineChart";
import { Button } from "@/components/ui/button";
import { format } from "date-fns";
import { RefreshCw } from "lucide-react";

function App() {
  const [selectedDate, setSelectedDate] = useState(new Date());
  const [
    activitySummary,
    setActivitySummary,
  ] = useState<ActivitySummary | null>(null);
  const [loading, setLoading] = useState(false);
  const [lastUpdated, setLastUpdated] = useState<Date | null>(null);

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
    <div className="min-h-screen bg-background p-8">
      <div className="max-w-7xl mx-auto space-y-8">
        {/* Header */}
        <div className="text-center">
          <h1 className="text-4xl font-bold">Velosi Tracker</h1>
          <p className="text-muted-foreground mt-2">
            Track your productivity and understand how you spend your time
          </p>
          {lastUpdated && (
            <p className="text-sm text-muted-foreground mt-1">
              Last updated: {format(lastUpdated, "HH:mm:ss")}
            </p>
          )}
        </div>

        {/* Controls */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          <TrackingControls />
          <DateSelector
            selectedDate={selectedDate}
            onDateChange={setSelectedDate}
          />
          <div className="flex items-center justify-center lg:justify-end">
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
              Refresh Data
            </Button>
          </div>
        </div>

        {/* Timeline Chart */}
        <TimelineChart minutes={30} />

        {/* Dashboard */}
        {loading ? (
          <div className="text-center p-8">
            <p className="text-muted-foreground">Loading activity data...</p>
          </div>
        ) : (
          <ActivityDashboard summary={activitySummary} />
        )}
      </div>
    </div>
  );
}

export default App;
