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
  const [screenDimensions, setScreenDimensions] = useState({
    width: 0,
    height: 0,
  });

  useEffect(() => {
    // Calculate screen dimensions
    const updateDimensions = () => {
      setScreenDimensions({
        width: window.innerWidth,
        height: window.innerHeight,
      });
    };

    // Set initial dimensions
    updateDimensions();

    // Listen for window resize
    window.addEventListener("resize", updateDimensions);

    // Ensure the entire page structure is set for full-screen display
    document.documentElement.style.height = "100%";
    document.body.style.height = "100%";
    document.body.style.margin = "0";
    document.body.style.padding = "0";
    document.body.style.overflow = "hidden";

    const root = document.getElementById("root");
    if (root) {
      root.style.height = "100%";
    }

    // Apply box-sizing to all elements for predictable layout
    const style = document.createElement("style");
    style.textContent = `* { box-sizing: border-box; }`;
    document.head.appendChild(style);

    // Cleanup on unmount
    return () => {
      window.removeEventListener("resize", updateDimensions);
      document.body.style.overflow = "";
      if (root) {
        root.style.height = "";
      }
      document.head.removeChild(style);
    };
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

  const handleAllowApp = async () => {
    try {
      if (blockedApp) {
        await invoke("temporarily_allow_app", {
          app_name: blockedApp.app_name,
        });
      }
      await invoke("hide_focus_overlay");
    } catch (error) {
      console.error("Failed to allow app:", error);
    }
  };

  if (!blockedApp) {
    return (
      <div className="fixed inset-0 w-full h-full bg-red-500 flex items-center justify-center">
        <div className="text-white text-xl">
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
        backgroundColor: "#1e3a8a",
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
          left: "17vw",
          backgroundColor: "rgba(255, 255, 255, 0.1)",
          backdropFilter: "blur(10px)",
          borderRadius: "24px",
          padding: "48px",
          textAlign: "center",
          border: "1px solid rgba(255, 255, 255, 0.2)",
          width: "600px",
          height: "500px",
        }}
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
              color: "white",
              marginBottom: "0.5rem",
              margin: "0 0 0.5rem 0",
            }}
          >
            🛡️ Focus Mode Active
          </h1>
          <p
            style={{
              color: "#bfdbfe",
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
            backgroundColor: "rgba(255, 255, 255, 0.05)",
            borderRadius: "1rem",
            border: "1px solid rgba(255, 255, 255, 0.1)",
          }}
        >
          <h2
            style={{
              fontSize: "1.5rem",
              fontWeight: "600",
              color: "white",
              marginBottom: "0.5rem",
              margin: "0 0 0.5rem 0",
            }}
          >
            <span style={{ color: "#f87171" }}>{blockedApp.app_name}</span> is
            blocked
          </h2>
          <p
            style={{
              color: "#bfdbfe",
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
              color: "#93c5fd",
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
              onClick={handleAllowApp}
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
              }}
              onMouseOver={(e) =>
                ((e.target as HTMLElement).style.backgroundColor = "#1d4ed8")
              }
              onMouseOut={(e) =>
                ((e.target as HTMLElement).style.backgroundColor = "#2563eb")
              }
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
        <div style={{ color: "#bfdbfe", fontSize: "0.875rem" }}>
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
            borderTop: "1px solid rgba(255, 255, 255, 0.2)",
          }}
        >
          <p
            style={{
              color: "#93c5fd",
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
