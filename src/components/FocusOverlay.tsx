import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface BlockedAppInfo {
  app_name: string;
  reason: string;
  timestamp: string;
}

export const FocusOverlay: React.FC = () => {
  const [blockedApp, setBlockedApp] = useState<BlockedAppInfo | null>(null);
  const [countdown, setCountdown] = useState(8);
  const [theme, setTheme] = useState<"light" | "dark">("dark");

  useEffect(() => {
    // Get theme from localStorage
    const storedTheme = localStorage.getItem("theme");
    if (storedTheme === "light" || storedTheme === "dark") {
      setTheme(storedTheme);
    } else {
      // Default to dark if not set
      setTheme("dark");
    }
  }, []);
  useEffect(() => {
    // Get the blocked app info from the URL or window data
    const urlParams = new URLSearchParams(window.location.search);
    const appName = urlParams.get("app_name");
    const reason = urlParams.get("reason");

    console.log("FocusOverlay URL:", window.location.href);
    console.log("URL params:", { appName, reason });

    if (appName && reason) {
      setBlockedApp({
        app_name: appName,
        reason: reason,
        timestamp: new Date().toISOString(),
      });
    } else {
      console.error("Missing app_name or reason in URL params");
      // Set fallback data for testing
      setBlockedApp({
        app_name: appName || "Unknown App",
        reason: reason || "App blocked by focus mode",
        timestamp: new Date().toISOString(),
      });
    }

    // Auto-close countdown
    const timer = setInterval(() => {
      setCountdown((prev) => {
        if (prev <= 1) {
          handleStayFocused();
          return 0;
        }
        return prev - 1;
      });
    }, 1000);

    return () => clearInterval(timer);
  }, []);

  useEffect(() => {
    // Handle ESC key to close overlay
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        handleStayFocused();
      }
    };

    document.addEventListener("keydown", handleKeyDown);

    return () => {
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, []);

  const handleStayFocused = async () => {
    try {
      await invoke("hide_focus_overlay");
    } catch (error) {
      console.error("Failed to hide overlay:", error);
    }
  };

  const handleDisableFocusMode = async () => {
    try {
      await invoke("disable_focus_mode");
      await invoke("hide_focus_overlay");
    } catch (error) {
      console.error("Failed to disable focus mode:", error);
    }
  };

  const handleAllowApp = async (durationMinutes?: number) => {
    console.log("🚀 handleAllowApp function called!");
    console.log("📋 Function parameters:", { durationMinutes });
    console.log("📱 Blocked app info:", blockedApp);
    console.log(
      "Allow app button clicked for:",
      blockedApp?.app_name,
      "duration:",
      durationMinutes
    );
    try {
      if (blockedApp) {
        console.log(
          "Invoking allow_app with:",
          blockedApp.app_name,
          "for",
          durationMinutes ? `${durationMinutes} minutes` : "indefinitely"
        );
        console.log("📞 About to call invoke('allow_app', ...)");
        console.log("📦 Invoke parameters:", {
          app_name: blockedApp.app_name,
          duration_minutes: durationMinutes || null,
        });

        const result = await invoke("allow_app", {
          appName: blockedApp.app_name,
          durationMinutes: durationMinutes || null, // null for indefinite
        });

        console.log("✅ invoke('allow_app') completed successfully");
        console.log("allow_app result:", result);

        // Also try to get the current allowed apps to verify
        try {
          const allowedApps = await invoke("get_focus_mode_allowed_apps");
          console.log("Current allowed apps after allowing:", allowedApps);
        } catch (allowedError) {
          console.error("Failed to get allowed apps:", allowedError);
        }

        console.log("Successfully allowed app, now hiding overlay");
      }
      const hideResult = await invoke("hide_focus_overlay");
      console.log("hide_focus_overlay result:", hideResult);
      console.log("Overlay hidden");
    } catch (error) {
      console.error("Failed to allow app - detailed error:", error);
      console.error("Error type:", typeof error);
      if (error && typeof error === "object") {
        console.error("Error keys:", Object.keys(error));
        console.error("Error as string:", String(error));
      }
    }
  };

  if (!blockedApp) {
    return (
      <div
        className={`fixed inset-0 w-full h-full ${
          theme === "dark" ? "bg-slate-900" : "bg-blue-100"
        } flex items-center justify-center`}
      >
        <div
          className={`text-xl ${
            theme === "dark" ? "text-white" : "text-gray-800"
          }`}
        >
          <p>Loading overlay...</p>
          <p className="text-sm mt-2">URL: {window.location.href}</p>
          <p className="text-sm">Search: {window.location.search}</p>
        </div>
      </div>
    );
  }

  return (
    <div
      style={{
        position: "fixed",
        top: "0",
        left: "0",
        width: "100%",
        height: "100%",
        backgroundColor: theme === "dark" ? "#0f172a" : "#dbeafe",
        margin: "0",
        padding: "0",
        zIndex: "9999",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
      }}
    >
      <div
        style={{
          position: "absolute",
          top: "5vh",
          left: "33vw",
          backgroundColor:
            theme === "dark"
              ? "rgba(0, 0, 0, 0.2)"
              : "rgba(255, 255, 255, 0.8)",
          backdropFilter: "blur(10px)",
          borderRadius: "24px",
          padding: "48px",
          textAlign: "center",
          border: "1px solid rgba(255, 255, 255, 0.2)",
          width: "600px",
          height: "500px",
          pointerEvents: "auto",
          zIndex: 999,
        }}
        onClick={() => console.log("Container clicked")}
      >
        {/* App Icon/Header */}
        <div style={{ marginBottom: "2rem" }}>
          <div
            style={{
              width: "96px",
              height: "96px",
              margin: "0 auto 1.5rem auto",
              backgroundColor: "rgba(234, 179, 8, 0.2)",
              borderRadius: "50%",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
            }}
          >
            <svg
              style={{ width: "48px", height: "48px", color: "#fbbf24" }}
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.732-.833-2.502 0L4.232 18.5c-.77.833.192 2.5 1.732 2.5z"
              />
            </svg>
          </div>
          <h1
            style={{
              fontSize: "2.25rem",
              fontWeight: "bold",
              color: theme === "dark" ? "white" : "#1f2937",
              marginBottom: "0.5rem",
              margin: "0 0 0.5rem 0",
            }}
          >
            🛡️ Focus Mode Active
          </h1>
          <p
            style={{
              color: theme === "dark" ? "#bfdbfe" : "#3b82f6",
              fontSize: "1.125rem",
              margin: "0",
            }}
          >
            Stay focused on what matters most
          </p>
        </div>

        {/* Blocked App Info */}
        <div
          style={{
            marginBottom: "2rem",
            padding: "1.5rem",
            backgroundColor:
              theme === "dark"
                ? "rgba(255, 255, 255, 0.05)"
                : "rgba(0, 0, 0, 0.05)",
            borderRadius: "1rem",
            border: `1px solid ${
              theme === "dark"
                ? "rgba(255, 255, 255, 0.1)"
                : "rgba(0, 0, 0, 0.1)"
            }`,
          }}
        >
          <h2
            style={{
              fontSize: "1.5rem",
              fontWeight: "600",
              color: theme === "dark" ? "white" : "#1f2937",
              marginBottom: "0.5rem",
              margin: "0 0 0.5rem 0",
            }}
          >
            <span style={{ color: "#f87171" }}>{blockedApp.app_name}</span> is
            blocked
          </h2>
          <p
            style={{
              color: theme === "dark" ? "#bfdbfe" : "#3b82f6",
              fontSize: "1.125rem",
              marginBottom: "1rem",
              margin: "0 0 1rem 0",
            }}
          >
            {blockedApp.reason}
          </p>
          <div
            style={{
              fontSize: "0.875rem",
              color: theme === "dark" ? "#93c5fd" : "#2563eb",
              margin: "0",
            }}
          >
            Blocked at {new Date(blockedApp.timestamp).toLocaleTimeString()}
          </div>
        </div>

        {/* Action Buttons */}
        <div style={{ marginBottom: "2rem" }}>
          <button
            onClick={handleStayFocused}
            style={{
              width: "100%",
              padding: "1rem 2rem",
              backgroundColor: "#16a34a",
              color: "white",
              fontWeight: "600",
              borderRadius: "0.75rem",
              border: "none",
              cursor: "pointer",
              fontSize: "1rem",
              marginBottom: "1rem",
              transition: "background-color 0.2s",
            }}
            onMouseOver={(e) =>
              ((e.target as HTMLElement).style.backgroundColor = "#15803d")
            }
            onMouseOut={(e) =>
              ((e.target as HTMLElement).style.backgroundColor = "#16a34a")
            }
          >
            Stay Focused ({countdown}s)
          </button>

          <div
            style={{
              display: "grid",
              gridTemplateColumns: "1fr 1fr",
              gap: "1rem",
            }}
          >
            <button
              onClick={() => {
                console.log("Allow This App button clicked!");
                handleAllowApp(30); // Allow for 30 minutes
              }}
              style={{
                padding: "0.75rem 1.5rem",
                backgroundColor: "#2563eb",
                color: "white",
                fontWeight: "500",
                borderRadius: "0.75rem",
                border: "none",
                cursor: "pointer",
                fontSize: "1rem",
                transition: "background-color 0.2s",
                zIndex: 1000,
                position: "relative",
                pointerEvents: "auto",
              }}
              onMouseOver={(e) => {
                console.log("Button hover");
                (e.target as HTMLElement).style.backgroundColor = "#1d4ed8";
              }}
              onMouseOut={(e) => {
                console.log("Button mouse out");
                (e.target as HTMLElement).style.backgroundColor = "#2563eb";
              }}
            >
              Allow This App (30 min)
            </button>

            <button
              onClick={handleDisableFocusMode}
              style={{
                padding: "0.75rem 1.5rem",
                backgroundColor: "#dc2626",
                color: "white",
                fontWeight: "500",
                borderRadius: "0.75rem",
                border: "none",
                cursor: "pointer",
                fontSize: "1rem",
                transition: "background-color 0.2s",
              }}
              onMouseOver={(e) =>
                ((e.target as HTMLElement).style.backgroundColor = "#b91c1c")
              }
              onMouseOut={(e) =>
                ((e.target as HTMLElement).style.backgroundColor = "#dc2626")
              }
            >
              Disable Focus Mode
            </button>
          </div>
        </div>

        {/* Tips */}
        <div
          style={{
            color: theme === "dark" ? "#bfdbfe" : "#3b82f6",
            fontSize: "0.875rem",
          }}
        >
          <p style={{ marginBottom: "0.5rem", margin: "0 0 0.5rem 0" }}>
            💡 <strong>Tips:</strong>
          </p>
          <ul
            style={{
              listStyle: "disc",
              listStylePosition: "inside",
              textAlign: "left",
              maxWidth: "400px",
              margin: "0 auto",
              padding: "0",
            }}
          >
            <li style={{ marginBottom: "0.25rem" }}>
              Focus mode helps you stay productive
            </li>
            <li style={{ marginBottom: "0.25rem" }}>
              Temporarily allowed apps expire after 30 minutes
            </li>
            <li style={{ marginBottom: "0.25rem" }}>
              You can adjust allowed categories in settings
            </li>
          </ul>
        </div>

        {/* Footer */}
        <div
          style={{
            marginTop: "2rem",
            paddingTop: "1.5rem",
            borderTop: `1px solid ${
              theme === "dark"
                ? "rgba(255, 255, 255, 0.2)"
                : "rgba(0, 0, 0, 0.2)"
            }`,
          }}
        >
          <p
            style={{
              color: theme === "dark" ? "#93c5fd" : "#2563eb",
              fontSize: "0.875rem",
              margin: "0",
            }}
          >
            Powered by <strong>Velosi</strong> • Productivity Tracker
          </p>
        </div>
      </div>
    </div>
  );
};

export default FocusOverlay;
