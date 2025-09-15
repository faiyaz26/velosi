import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import {
  Globe,
  AlertCircle,
  CheckCircle2,
  Copy,
  ExternalLink,
  Settings,
} from "lucide-react";

interface WebsiteBlockerStatus {
  running: boolean;
  method: string;
  platform: string;
  proxy_address?: string;
  proxy_port?: number;
}

export function ProxySetupGuide() {
  const [
    websiteBlockerStatus,
    setWebsiteBlockerStatus,
  ] = useState<WebsiteBlockerStatus | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  const checkWebsiteBlockerStatus = async () => {
    setIsLoading(true);
    try {
      const status = await invoke<WebsiteBlockerStatus>(
        "get_website_blocker_status"
      );
      setWebsiteBlockerStatus(status);
    } catch (error) {
      console.error("Failed to get website blocker status:", error);
    } finally {
      setIsLoading(false);
    }
  };

  const startWebsiteBlocker = async () => {
    try {
      await invoke("start_website_blocker");
      await checkWebsiteBlockerStatus();
    } catch (error) {
      console.error("Failed to start website blocker:", error);
    }
  };

  const stopWebsiteBlocker = async () => {
    try {
      await invoke("stop_website_blocker");
      await checkWebsiteBlockerStatus();
    } catch (error) {
      console.error("Failed to stop website blocker:", error);
    }
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center space-x-3">
        <Globe className="h-8 w-8 text-blue-500" />
        <div>
          <h2 className="text-xl font-bold text-foreground">
            Website Blocking Setup
          </h2>
          <p className="text-muted-foreground">
            Configure your system to block distracting websites during focus
            mode
          </p>
        </div>
      </div>

      {/* Proxy Status */}
      <Card className="p-6">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center space-x-3">
            <div
              className={`w-4 h-4 rounded-full ${
                proxyStatus?.running ? "bg-green-500" : "bg-red-500"
              }`}
            />
            <div>
              <h3 className="font-semibold text-foreground">
                Proxy Server Status
              </h3>
              <p className="text-sm text-muted-foreground">
                {proxyStatus?.running
                  ? `Running on ${proxyStatus.address}`
                  : "Not running"}
              </p>
            </div>
          </div>
          <div className="space-x-2">
            <Button variant="outline" size="sm" onClick={checkProxyStatus}>
              {isLoading ? "Checking..." : "Check Status"}
            </Button>
            {proxyStatus?.running ? (
              <Button variant="destructive" size="sm" onClick={stopProxy}>
                Stop Proxy
              </Button>
            ) : (
              <Button variant="default" size="sm" onClick={startProxy}>
                Start Proxy
              </Button>
            )}
          </div>
        </div>

        {proxyStatus?.running && (
          <div className="p-4 bg-green-50 dark:bg-green-950/20 rounded-lg border border-green-200 dark:border-green-800">
            <div className="flex items-start space-x-2">
              <CheckCircle2 className="h-5 w-5 text-green-500 mt-0.5" />
              <div>
                <p className="font-medium text-green-700 dark:text-green-300">
                  Proxy server is running
                </p>
                <p className="text-sm text-green-600 dark:text-green-400">
                  Configure your browser or system to use the proxy settings
                  below
                </p>
              </div>
            </div>
          </div>
        )}
      </Card>

      {/* Configuration Instructions */}
      {proxyStatus?.running && (
        <Card className="p-6">
          <div className="flex items-center space-x-2 mb-4">
            <Settings className="h-5 w-5 text-primary" />
            <h3 className="font-semibold text-foreground">
              Browser Configuration
            </h3>
          </div>

          <div className="space-y-4">
            {/* Proxy Settings */}
            <div className="p-4 bg-muted rounded-lg">
              <h4 className="font-medium text-foreground mb-2">
                Proxy Settings
              </h4>
              <div className="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <span className="text-muted-foreground">Protocol:</span>
                  <div className="flex items-center space-x-2">
                    <code className="bg-background px-2 py-1 rounded">
                      HTTP
                    </code>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => copyToClipboard("HTTP")}
                    >
                      <Copy className="h-3 w-3" />
                    </Button>
                  </div>
                </div>
                <div>
                  <span className="text-muted-foreground">Address:</span>
                  <div className="flex items-center space-x-2">
                    <code className="bg-background px-2 py-1 rounded">
                      127.0.0.1
                    </code>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => copyToClipboard("127.0.0.1")}
                    >
                      <Copy className="h-3 w-3" />
                    </Button>
                  </div>
                </div>
                <div>
                  <span className="text-muted-foreground">Port:</span>
                  <div className="flex items-center space-x-2">
                    <code className="bg-background px-2 py-1 rounded">
                      62828
                    </code>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => copyToClipboard("62828")}
                    >
                      <Copy className="h-3 w-3" />
                    </Button>
                  </div>
                </div>
                <div>
                  <span className="text-muted-foreground">Full Address:</span>
                  <div className="flex items-center space-x-2">
                    <code className="bg-background px-2 py-1 rounded">
                      127.0.0.1:62828
                    </code>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => copyToClipboard("127.0.0.1:62828")}
                    >
                      <Copy className="h-3 w-3" />
                    </Button>
                  </div>
                </div>
              </div>
            </div>

            {/* Browser-specific instructions */}
            <div className="space-y-3">
              <h4 className="font-medium text-foreground">
                Browser Configuration Steps
              </h4>

              {/* Chrome */}
              <div className="p-3 border rounded-lg">
                <div className="flex items-center justify-between mb-2">
                  <h5 className="font-medium text-foreground">Google Chrome</h5>
                  <ExternalLink className="h-4 w-4 text-muted-foreground" />
                </div>
                <ol className="text-sm text-muted-foreground space-y-1 list-decimal list-inside">
                  <li>Go to Chrome Settings → Advanced → System</li>
                  <li>Click "Open your computer's proxy settings"</li>
                  <li>Enable "Use a proxy server"</li>
                  <li>Set Address: 127.0.0.1, Port: 62828</li>
                  <li>Click OK to save</li>
                </ol>
              </div>

              {/* Firefox */}
              <div className="p-3 border rounded-lg">
                <div className="flex items-center justify-between mb-2">
                  <h5 className="font-medium text-foreground">Firefox</h5>
                  <ExternalLink className="h-4 w-4 text-muted-foreground" />
                </div>
                <ol className="text-sm text-muted-foreground space-y-1 list-decimal list-inside">
                  <li>Go to Firefox Settings → General → Network Settings</li>
                  <li>Click "Settings..." button</li>
                  <li>Select "Manual proxy configuration"</li>
                  <li>HTTP Proxy: 127.0.0.1, Port: 62828</li>
                  <li>Check "Use this proxy server for all protocols"</li>
                  <li>Click OK</li>
                </ol>
              </div>

              {/* Safari */}
              <div className="p-3 border rounded-lg">
                <div className="flex items-center justify-between mb-2">
                  <h5 className="font-medium text-foreground">Safari</h5>
                  <ExternalLink className="h-4 w-4 text-muted-foreground" />
                </div>
                <ol className="text-sm text-muted-foreground space-y-1 list-decimal list-inside">
                  <li>Go to System Preferences → Network</li>
                  <li>Select your network connection</li>
                  <li>Click "Advanced..." → Proxies tab</li>
                  <li>Check "Web Proxy (HTTP)"</li>
                  <li>Set Web Proxy Server: 127.0.0.1:62828</li>
                  <li>Click OK and Apply</li>
                </ol>
              </div>
            </div>
          </div>
        </Card>
      )}

      {/* Warning */}
      <Card className="p-6">
        <div className="flex items-start space-x-3">
          <AlertCircle className="h-5 w-5 text-amber-500 mt-0.5" />
          <div>
            <h3 className="font-semibold text-foreground mb-2">
              Important Notes
            </h3>
            <ul className="text-sm text-muted-foreground space-y-1 list-disc list-inside">
              <li>
                The proxy only works when both Velosi and focus mode are active
              </li>
              <li>
                Remember to disable proxy settings when not using focus mode
              </li>
              <li>
                Some system-level apps might bypass browser proxy settings
              </li>
              <li>
                For system-wide blocking, configure proxy in System Preferences
              </li>
            </ul>
          </div>
        </div>
      </Card>
    </div>
  );
}
