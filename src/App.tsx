import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Sidebar } from "@/components/Sidebar";
import { Dashboard } from "@/components/Dashboard";
import { ActivityLog } from "@/components/ActivityLog";
import Categorization from "@/components/Categorization";
import { FocusMode } from "@/components/FocusMode";
import { FocusOverlay } from "@/components/FocusOverlay";
import { Settings } from "@/components/Settings";
import { Button } from "@/components/ui/button";
import { Minimize2 } from "lucide-react";
import { categoryService } from "@/lib/categoryService";
import { updateService } from "@/lib/updateService";

function App() {
  const [activeView, setActiveView] = useState("dashboard");
  const [isNavigating, setIsNavigating] = useState(false);

  // Check if this is the focus overlay window
  const isOverlayWindow = window.location.pathname === "/focus-overlay";

  // Initialize category service on app start
  useEffect(() => {
    categoryService.initialize().catch(console.error);
  }, []);

  // Initialize update service on app start
  useEffect(() => {
    updateService.checkForUpdatesOnStartup();
  }, []);

  // Listen for blocked websites globally for notifications
  useEffect(() => {
    const setupWebsiteBlockListener = async () => {
      const unlisten = await listen("website-blocked", (event) => {
        const blockedWebsite = event.payload as {
          url: string;
          reason: string;
          timestamp: string;
        };

        // Show browser notification
        if (Notification.permission === "granted") {
          new Notification("Website Blocked", {
            body: `Access to ${blockedWebsite.url} was blocked`,
            icon: "/Velosi.png",
          });
        } else if (Notification.permission !== "denied") {
          Notification.requestPermission().then((permission) => {
            if (permission === "granted") {
              new Notification("Website Blocked", {
                body: `Access to ${blockedWebsite.url} was blocked`,
                icon: "/Velosi.png",
              });
            }
          });
        }
      });

      return unlisten;
    };

    setupWebsiteBlockListener();

    // Cleanup on unmount
    return () => {
      setupWebsiteBlockListener().then((unlisten) => unlisten());
    };
  }, []);

  // If this is the overlay window, render only the overlay
  if (isOverlayWindow) {
    return <FocusOverlay />;
  }

  const handleMinimize = async () => {
    try {
      await invoke("hide_window");
    } catch (error) {
      console.error("Failed to minimize window:", error);
    }
  };

  const handleViewChange = (view: string) => {
    if (view === "activity-log") {
      setIsNavigating(true);
      // Small delay to show loading state before navigation
      setTimeout(() => {
        setActiveView(view);
        setIsNavigating(false);
      }, 100);
    } else {
      setActiveView(view);
    }
  };

  const renderActiveView = () => {
    switch (activeView) {
      case "dashboard":
        return <Dashboard onNavigate={setActiveView} />;
      case "activity-log":
        return <ActivityLog isNavigating={isNavigating} />;
      case "categorization":
        return <Categorization />;
      case "focus-mode":
        return <FocusMode />;
      case "settings":
        return <Settings />;
      default:
        return <Dashboard />;
    }
  };

  return (
    <div className="h-screen bg-background flex overflow-hidden app-draggable">
      {/* Sidebar - Fixed */}
      <div className="no-drag">
        <Sidebar activeView={activeView} onViewChange={handleViewChange} />
      </div>

      {/* Main Content */}
      <div className="flex-1 flex flex-col h-screen">
        {/* Top bar */}
        <div className="border-b border-border p-4 flex-shrink-0">
          <div className="flex items-center justify-end gap-2">
            <Button
              onClick={handleMinimize}
              size="sm"
              variant="outline"
              className="flex items-center gap-1 no-drag"
            >
              <Minimize2 className="h-4 w-4" />
            </Button>
            {/* removed close button - using minimize (hide) only */}
          </div>
        </div>

        {/* Main content area - Scrollable */}
        <div className="flex-1 p-6 overflow-auto no-drag">
          {renderActiveView()}
        </div>
      </div>
    </div>
  );
}

export default App;
