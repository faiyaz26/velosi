import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Sidebar } from "@/components/Sidebar";
import { Dashboard } from "@/components/Dashboard";
import { ActivityLog } from "@/components/ActivityLog";
import { Categories } from "@/components/Categories";
import { Settings } from "@/components/Settings";
import { ThemeToggle } from "@/components/ThemeToggle";
import { Button } from "@/components/ui/button";
import { Minimize2, X } from "lucide-react";
import { categoryService } from "@/lib/categoryService";

function App() {
  const [activeView, setActiveView] = useState("dashboard");

  // Initialize category service on app start
  useEffect(() => {
    categoryService.initialize().catch(console.error);
  }, []);

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

  const renderActiveView = () => {
    switch (activeView) {
      case "dashboard":
        return <Dashboard />;
      case "activity-log":
        return <ActivityLog />;
      case "categories":
        return <Categories />;
      case "settings":
        return <Settings />;
      default:
        return <Dashboard />;
    }
  };

  return (
    <div className="h-screen bg-background flex overflow-hidden">
      {/* Sidebar - Fixed */}
      <Sidebar activeView={activeView} onViewChange={setActiveView} />

      {/* Main Content */}
      <div className="flex-1 flex flex-col h-screen">
        {/* Top bar */}
        <div className="border-b border-border p-4 flex-shrink-0">
          <div className="flex items-center justify-end gap-2">
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

        {/* Main content area - Scrollable */}
        <div className="flex-1 p-6 overflow-auto">{renderActiveView()}</div>
      </div>
    </div>
  );
}

export default App;
