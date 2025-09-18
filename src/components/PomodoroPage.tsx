import React, { useState, useEffect } from "react";
import { PomodoroTimer, PomodoroSession } from "./PomodoroTimer";
import { Card, CardContent, CardHeader, CardTitle } from "./ui/card";
import { Button } from "./ui/button";
import { Badge } from "./ui/badge";
import { Calendar, Clock, Target, TrendingUp } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";

interface PomodoroSummary {
  total_sessions: number;
  completed_sessions: number;
  total_work_time_seconds: number;
  total_break_time_seconds: number;
  average_session_duration: number;
  sessions_by_date: Array<{
    date: string;
    work_sessions: number;
    break_sessions: number;
    total_work_time_seconds: number;
    total_break_time_seconds: number;
  }>;
}

export const PomodoroPage: React.FC = () => {
  const [recentSessions, setRecentSessions] = useState<PomodoroSession[]>([]);
  const [summary, setSummary] = useState<PomodoroSummary | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadPomodoroData();
  }, []);

  const loadPomodoroData = async () => {
    try {
      setLoading(true);

      // Get recent sessions (last 7 days)
      const endDate = new Date().toISOString().split("T")[0];
      const startDate = new Date(Date.now() - 7 * 24 * 60 * 60 * 1000)
        .toISOString()
        .split("T")[0];

      const [sessions, summaryData] = await Promise.all([
        invoke<PomodoroSession[]>("get_pomodoro_sessions", {
          startDate,
          endDate,
          sessionType: null,
          limit: 10,
        }),
        invoke<PomodoroSummary>("get_pomodoro_summary", {
          startDate,
          endDate,
        }),
      ]);

      setRecentSessions(sessions.slice(0, 10)); // Show only last 10 sessions
      setSummary(summaryData);
    } catch (error) {
      console.error("Failed to load pomodoro data:", error);
    } finally {
      setLoading(false);
    }
  };

  const handleSessionComplete = (_session: PomodoroSession) => {
    // Refresh data when a session is completed
    loadPomodoroData();
  };

  const formatDuration = (seconds: number): string => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);

    if (hours > 0) {
      return `${hours}h ${minutes}m`;
    }
    return `${minutes}m`;
  };

  const formatDate = (dateString: string): string => {
    return new Date(dateString).toLocaleDateString(undefined, {
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  };

  if (loading) {
    return (
      <div className="container mx-auto p-6">
        <div className="flex items-center justify-center h-64">
          <div className="text-lg">Loading...</div>
        </div>
      </div>
    );
  }

  return (
    <div className="container mx-auto p-6 max-w-6xl">
      <div className="mb-8">
        <h1 className="text-3xl font-bold mb-2">Pomodoro Timer</h1>
        <p className="text-gray-600">
          Boost your productivity with focused work sessions and regular breaks.
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        {/* Timer Section */}
        <div className="lg:col-span-1">
          <PomodoroTimer onSessionComplete={handleSessionComplete} />
        </div>

        {/* Statistics and Recent Sessions */}
        <div className="lg:col-span-2 space-y-6">
          {/* Statistics Cards */}
          {summary && (
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              <Card>
                <CardContent className="p-4 text-center">
                  <Target className="h-8 w-8 mx-auto mb-2 text-blue-500" />
                  <div className="text-2xl font-bold">
                    {summary.total_sessions}
                  </div>
                  <div className="text-sm text-gray-600">Total Sessions</div>
                </CardContent>
              </Card>

              <Card>
                <CardContent className="p-4 text-center">
                  <TrendingUp className="h-8 w-8 mx-auto mb-2 text-green-500" />
                  <div className="text-2xl font-bold">
                    {summary.completed_sessions}
                  </div>
                  <div className="text-sm text-gray-600">Completed</div>
                </CardContent>
              </Card>

              <Card>
                <CardContent className="p-4 text-center">
                  <Clock className="h-8 w-8 mx-auto mb-2 text-purple-500" />
                  <div className="text-2xl font-bold">
                    {formatDuration(summary.total_work_time_seconds)}
                  </div>
                  <div className="text-sm text-gray-600">Work Time</div>
                </CardContent>
              </Card>

              <Card>
                <CardContent className="p-4 text-center">
                  <Calendar className="h-8 w-8 mx-auto mb-2 text-orange-500" />
                  <div className="text-2xl font-bold">
                    {Math.round(summary.average_session_duration / 60)}m
                  </div>
                  <div className="text-sm text-gray-600">Avg Session</div>
                </CardContent>
              </Card>
            </div>
          )}

          {/* Daily Summary */}
          {summary && summary.sessions_by_date.length > 0 && (
            <Card>
              <CardHeader>
                <CardTitle>Daily Summary (Last 7 Days)</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  {summary.sessions_by_date.map((day) => (
                    <div
                      key={day.date}
                      className="flex items-center justify-between p-2 border rounded"
                    >
                      <div className="font-medium">
                        {new Date(day.date).toLocaleDateString(undefined, {
                          weekday: "short",
                          month: "short",
                          day: "numeric",
                        })}
                      </div>
                      <div className="flex items-center space-x-4 text-sm">
                        <span>{day.work_sessions} work sessions</span>
                        <span>
                          {formatDuration(day.total_work_time_seconds)} work
                          time
                        </span>
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          )}

          {/* Recent Sessions */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center justify-between">
                <span>Recent Sessions</span>
                <Button variant="outline" size="sm" onClick={loadPomodoroData}>
                  Refresh
                </Button>
              </CardTitle>
            </CardHeader>
            <CardContent>
              {recentSessions.length === 0 ? (
                <div className="text-center py-8 text-gray-500">
                  <Clock className="h-12 w-12 mx-auto mb-4 opacity-50" />
                  <p>No sessions yet. Start your first pomodoro!</p>
                </div>
              ) : (
                <div className="space-y-3">
                  {recentSessions.map((session) => (
                    <div
                      key={session.id}
                      className="flex items-center justify-between p-3 border rounded-lg"
                    >
                      <div className="flex items-center space-x-3">
                        <Badge
                          variant={
                            session.session_type === "work"
                              ? "default"
                              : "secondary"
                          }
                        >
                          {session.session_type === "work" ? "Work" : "Break"}
                        </Badge>
                        <div>
                          <div className="font-medium">
                            {session.duration_minutes} minutes
                          </div>
                          <div className="text-sm text-gray-600">
                            {formatDate(session.start_time)}
                          </div>
                        </div>
                      </div>

                      <div className="text-right">
                        <Badge
                          variant={session.completed ? "default" : "outline"}
                        >
                          {session.completed ? "Completed" : "Interrupted"}
                        </Badge>
                        {session.work_description && (
                          <div className="text-sm text-gray-600 mt-1 max-w-48 truncate">
                            {session.work_description}
                          </div>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
};
