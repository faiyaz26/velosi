import { check } from "@tauri-apps/plugin-updater";
import { ask } from "@tauri-apps/plugin-dialog";
import { relaunch } from "@tauri-apps/plugin-process";

export class UpdateService {
  private checkingForUpdates = false;

  async checkForUpdates(silent = false): Promise<boolean> {
    if (this.checkingForUpdates) {
      return false;
    }

    this.checkingForUpdates = true;

    try {
      const update = await check();

      if (!update) {
        if (!silent) {
          // You can show a "No updates available" message here if needed
          console.log("No updates available");
        }
        return false;
      }

      console.log(`Update available: ${update.version}`);

      // Show update dialog
      const shouldUpdate = await ask(
        `Update to version ${update.version} is available!\n\nChanges:\n${
          update.body || "Bug fixes and improvements"
        }\n\nWould you like to update now?`,
        {
          title: "Update Available",
          kind: "info",
        }
      );

      if (shouldUpdate) {
        console.log("Downloading and installing update...");

        // Download and install the update
        await update.downloadAndInstall();

        // Ask if user wants to restart now
        const shouldRestart = await ask(
          "Update installed successfully!\n\nWould you like to restart the application now?",
          {
            title: "Update Installed",
            kind: "info",
          }
        );

        if (shouldRestart) {
          await relaunch();
        }

        return true;
      }

      return false;
    } catch (error) {
      console.error("Update check failed:", error);

      if (!silent) {
        // You can show an error message here if needed
        console.error("Failed to check for updates");
      }

      return false;
    } finally {
      this.checkingForUpdates = false;
    }
  }

  async checkForUpdatesOnStartup(): Promise<void> {
    // Check for updates silently on startup (after a delay)
    setTimeout(() => {
      this.checkForUpdates(true);
    }, 5000); // Wait 5 seconds after startup
  }
}

export const updateService = new UpdateService();
