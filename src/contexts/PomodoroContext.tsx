import React, {
  createContext,
  useContext,
  useState,
  useEffect,
  useRef,
  ReactNode,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import { sendNotification } from "@tauri-apps/plugin-notification";
import { PomodoroSession, PomodoroSettings } from "../components/PomodoroTimer";

export type TimerState = "stopped" | "running" | "paused";
export type SessionType = "work" | "break";

interface PomodoroContextType {
  // Timer state
  timeLeft: number;
  timerState: TimerState;
  sessionType: SessionType;
  currentSession: PomodoroSession | null;
  workDescription: string;
  settings: PomodoroSettings | null;
  completedSessionType: SessionType | null;

  // Actions
  setTimeLeft: (time: number) => void;
  setTimerState: (state: TimerState) => void;
  setSessionType: (type: SessionType) => void;
  setCurrentSession: (session: PomodoroSession | null) => void;
  setWorkDescription: (description: string) => void;
  setSettings: (settings: PomodoroSettings | null) => void;
  setCompletedSessionType: (type: SessionType | null) => void;

  // Timer controls
  startTimer: () => Promise<void>;
  pauseTimer: () => void;
  resumeTimer: () => void;
  stopTimer: () => Promise<void>;
  loadSettings: () => Promise<void>;

  // Callbacks
  onSessionComplete?: (session: PomodoroSession) => void;
  setOnSessionComplete: (callback: (session: PomodoroSession) => void) => void;
}

const PomodoroContext = createContext<PomodoroContextType | undefined>(
  undefined
);

interface PomodoroProviderProps {
  children: ReactNode;
}

export const PomodoroProvider: React.FC<PomodoroProviderProps> = ({
  children,
}) => {
  const [timeLeft, setTimeLeft] = useState(0);
  const [timerState, setTimerState] = useState<TimerState>("stopped");
  const [sessionType, setSessionType] = useState<SessionType>("work");
  const [currentSession, setCurrentSession] = useState<PomodoroSession | null>(
    null
  );
  const [workDescription, setWorkDescription] = useState("");
  const [settings, setSettings] = useState<PomodoroSettings | null>(null);
  const [
    completedSessionType,
    setCompletedSessionType,
  ] = useState<SessionType | null>(null);
  const [onSessionComplete, setOnSessionComplete] = useState<
    ((session: PomodoroSession) => void) | undefined
  >();

  const intervalRef = useRef<NodeJS.Timeout | null>(null);
  const startTimeRef = useRef<number>(0);
  const endTimeRef = useRef<number | null>(null);

  // Initialize timer when settings are loaded
  useEffect(() => {
    if (settings && timerState === "stopped") {
      setTimeLeft(
        sessionType === "work"
          ? settings.work_duration_minutes * 60
          : settings.break_duration_minutes * 60
      );
    }
  }, [settings, sessionType, timerState]);

  // Timer countdown effect: compute remaining time from endTimeRef so timer stays accurate in background
  useEffect(() => {
    if (timerState === "running") {
      // ensure we have an end time
      if (!endTimeRef.current) {
        // fallback: set end from current time + timeLeft
        endTimeRef.current = Date.now() + timeLeft * 1000;
      }

      intervalRef.current = setInterval(() => {
        if (!endTimeRef.current) return;
        const remainingMs = endTimeRef.current - Date.now();
        const remainingSec = Math.max(0, Math.ceil(remainingMs / 1000));
        setTimeLeft(remainingSec);
        if (remainingSec <= 0) {
          // clear before calling completion to avoid races
          if (intervalRef.current) {
            clearInterval(intervalRef.current);
            intervalRef.current = null;
          }
          endTimeRef.current = null;
          handleTimerComplete();
        }
      }, 500);
    } else {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
        intervalRef.current = null;
      }
    }

    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
        intervalRef.current = null;
      }
    };
  }, [timerState, timeLeft]);

  const loadSettings = async () => {
    try {
      const pomodoroSettings = await invoke<PomodoroSettings>(
        "get_pomodoro_settings"
      );
      // Normalize boolean-like fields (DB may return strings or numbers)
      const normalizeBool = (v: any): boolean => {
        if (v === true) return true;
        if (v === false) return false;
        if (typeof v === "string") {
          const s = v.trim().toLowerCase();
          return s === "true" || s === "1" || s === "t" || s === "yes";
        }
        if (typeof v === "number") return v === 1;
        return Boolean(v);
      };

      const normalized: PomodoroSettings = {
        ...pomodoroSettings,
        enable_focus_mode: normalizeBool(pomodoroSettings.enable_focus_mode),
        enable_app_tracking: normalizeBool(
          pomodoroSettings.enable_app_tracking
        ),
      };

      setSettings(normalized);
    } catch (error) {
      console.error("Failed to load pomodoro settings:", error);
      // Set default settings if loading fails
      const defaultSettings: PomodoroSettings = {
        id: "default",
        work_duration_minutes: 25,
        break_duration_minutes: 5,
        enable_focus_mode: false,
        enable_app_tracking: false,
        auto_start_breaks: true,
        auto_start_work: true,
        updated_at: new Date().toISOString(),
      };
      setSettings(defaultSettings);
    }

    // Proactively request notification permissions for better UX
    if ("Notification" in window && Notification.permission === "default") {
      try {
        await Notification.requestPermission();
      } catch (error) {
        console.warn("Could not request notification permission:", error);
      }
    }
  };

  const handleTimerComplete = async () => {
    setTimerState("stopped");

    if (currentSession) {
      try {
        const completedSession = await invoke<PomodoroSession>(
          "complete_pomodoro_session",
          {
            sessionId: currentSession.id,
            completed: true,
          }
        );

        onSessionComplete?.(completedSession);
        setCompletedSessionType(sessionType);

        // Auto-switch to next session type
        const nextType: SessionType = sessionType === "work" ? "break" : "work";
        setSessionType(nextType);

        if (settings) {
          const nextDuration =
            nextType === "work"
              ? settings.work_duration_minutes
              : settings.break_duration_minutes;
          setTimeLeft(nextDuration * 60);
        }
      } catch (error) {
        console.error("Failed to complete pomodoro session:", error);
      }
    }

    setCurrentSession(null);
    endTimeRef.current = null;

    // Play ding sound
    playDingSound();

    // Show system-wide notification
    showSystemNotification(sessionType);
  };

  const playDingSound = () => {
    try {
      // Create a simple ding sound using Web Audio API
      const audioContext = new (window.AudioContext ||
        (window as any).webkitAudioContext)();

      // Create oscillator for the ding sound
      const oscillator = audioContext.createOscillator();
      const gainNode = audioContext.createGain();

      // Connect nodes
      oscillator.connect(gainNode);
      gainNode.connect(audioContext.destination);

      // Configure the ding sound (pleasant bell-like tone)
      oscillator.frequency.setValueAtTime(800, audioContext.currentTime); // Start at 800Hz
      oscillator.frequency.exponentialRampToValueAtTime(
        600,
        audioContext.currentTime + 0.1
      ); // Drop to 600Hz
      oscillator.frequency.exponentialRampToValueAtTime(
        400,
        audioContext.currentTime + 0.3
      ); // Drop to 400Hz

      // Set volume envelope (fade in and fade out)
      gainNode.gain.setValueAtTime(0, audioContext.currentTime);
      gainNode.gain.linearRampToValueAtTime(
        0.3,
        audioContext.currentTime + 0.01
      ); // Quick attack
      gainNode.gain.exponentialRampToValueAtTime(
        0.01,
        audioContext.currentTime + 0.8
      ); // Slow decay

      // Play the sound for 0.8 seconds
      oscillator.start(audioContext.currentTime);
      oscillator.stop(audioContext.currentTime + 0.8);

      // Clean up
      setTimeout(() => {
        audioContext.close();
      }, 1000);
    } catch (error) {
      console.warn("Could not play ding sound:", error);
    }
  };

  const showSystemNotification = async (completedSessionType: SessionType) => {
    const title = `${
      completedSessionType === "work" ? "Work" : "Break"
    } session completed!`;
    const body =
      completedSessionType === "work"
        ? "Time for a break! 🎉"
        : "Break's over! Ready to get back to work? 💪";

    console.log("🔔 Attempting to show notification:", { title, body });

    try {
      // Try to use native Tauri notification first (system-wide and more reliable)
      console.log("📱 Trying native Tauri notification...");
      await sendNotification({
        title,
        body,
      });
      console.log("✅ Native notification sent successfully");
    } catch (error) {
      console.warn(
        "❌ Native notification failed, falling back to browser notification:",
        error
      );

      // Fallback to browser notification
      if ("Notification" in window) {
        let permission = Notification.permission;

        if (permission === "default") {
          permission = await Notification.requestPermission();
        }

        if (permission === "granted") {
          const notification = new Notification(title, {
            body,
            icon: "/Velosi.png",
            badge: "/Velosi.png",
            tag: "pomodoro-complete", // Prevents duplicate notifications
            requireInteraction: true, // Keeps notification visible until user interacts
            silent: false, // Allow system sound (in addition to our ding)
          });

          // Auto-close notification after 10 seconds if user doesn't interact
          setTimeout(() => {
            notification.close();
          }, 10000);

          // Optional: Handle notification click
          notification.onclick = () => {
            window.focus(); // Bring app to foreground
            notification.close();
          };
        } else {
          console.warn("Notification permission denied or not supported");
        }
      } else {
        console.warn("Notifications not supported in this browser");
      }
    }
  };

  const startTimer = async () => {
    if (!settings) return;

    try {
      const duration =
        sessionType === "work"
          ? settings.work_duration_minutes
          : settings.break_duration_minutes;

      const session = await invoke<PomodoroSession>("start_pomodoro_session", {
        sessionType,
        durationMinutes: duration,
        workDescription:
          sessionType === "work" ? workDescription || null : null,
        enableFocusMode: settings.enable_focus_mode,
        enableAppTracking: settings.enable_app_tracking,
      });

      setCurrentSession(session);
      setTimerState("running");
      startTimeRef.current = Date.now();
      endTimeRef.current = Date.now() + duration * 60 * 1000;
      setTimeLeft(duration * 60);

      // Request notification permission if not already granted
      if ("Notification" in window && Notification.permission === "default") {
        Notification.requestPermission();
      }
      // Ensure backend tracking state matches the Pomodoro setting (defensive)
      try {
        if (settings.enable_app_tracking) {
          await invoke("start_tracking");
          console.log("Started app tracking as requested by Pomodoro settings");
        } else {
          await invoke("stop_tracking");
          console.log("Stopped app tracking as requested by Pomodoro settings");
        }
      } catch (err) {
        console.warn(
          "Failed to sync tracking state after starting session:",
          err
        );
      }
    } catch (error) {
      console.error("Failed to start pomodoro session:", error);
    }
  };

  const pauseTimer = () => {
    // compute remaining seconds and clear end time so background doesn't update
    if (endTimeRef.current) {
      const remainingSec = Math.max(
        0,
        Math.ceil((endTimeRef.current - Date.now()) / 1000)
      );
      setTimeLeft(remainingSec);
      endTimeRef.current = null;
    }
    setTimerState("paused");
  };

  const resumeTimer = () => {
    // resume from timeLeft
    endTimeRef.current = Date.now() + timeLeft * 1000;
    setTimerState("running");
  };

  const stopTimer = async () => {
    setTimerState("stopped");
    endTimeRef.current = null;

    if (currentSession) {
      try {
        await invoke("complete_pomodoro_session", {
          sessionId: currentSession.id,
          completed: false,
        });
      } catch (error) {
        console.error("Failed to stop pomodoro session:", error);
      }
    }

    setCurrentSession(null);

    if (settings) {
      const duration =
        sessionType === "work"
          ? settings.work_duration_minutes
          : settings.break_duration_minutes;
      setTimeLeft(duration * 60);
    }
  };

  const value: PomodoroContextType = {
    // State
    timeLeft,
    timerState,
    sessionType,
    currentSession,
    workDescription,
    settings,
    completedSessionType,

    // Setters
    setTimeLeft,
    setTimerState,
    setSessionType,
    setCurrentSession,
    setWorkDescription,
    setSettings,
    setCompletedSessionType,

    // Actions
    startTimer,
    pauseTimer,
    resumeTimer,
    stopTimer,
    loadSettings,

    // Callbacks
    onSessionComplete,
    setOnSessionComplete: (callback: (session: PomodoroSession) => void) =>
      setOnSessionComplete(() => callback),
  };

  return (
    <PomodoroContext.Provider value={value}>
      {children}
    </PomodoroContext.Provider>
  );
};

export const usePomodoro = () => {
  const context = useContext(PomodoroContext);
  if (context === undefined) {
    throw new Error("usePomodoro must be used within a PomodoroProvider");
  }
  return context;
};
