import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { Edit, Plus, X } from "lucide-react";

interface AppMappingEditorProps {
  category: string;
  categoryName: string;
  apps: string[];
  onUpdate: (category: string, apps: string[]) => void;
}

export function AppMappingEditor({
  category,
  categoryName,
  apps,
  onUpdate,
}: AppMappingEditorProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [editingApps, setEditingApps] = useState<string[]>([]);
  const [newApp, setNewApp] = useState("");

  const openEditor = () => {
    setEditingApps([...apps]);
    setIsOpen(true);
  };

  const handleSave = () => {
    onUpdate(category, editingApps);
    setIsOpen(false);
  };

  const addApp = () => {
    if (newApp.trim() && !editingApps.includes(newApp.trim())) {
      setEditingApps([...editingApps, newApp.trim()]);
      setNewApp("");
    }
  };

  const removeApp = (index: number) => {
    setEditingApps(editingApps.filter((_, i) => i !== index));
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      addApp();
    }
  };

  return (
    <>
      <Button variant="outline" size="sm" onClick={openEditor}>
        <Edit className="w-4 h-4 mr-2" />
        Edit Mapping
      </Button>

      <Dialog open={isOpen} onOpenChange={setIsOpen}>
        <DialogContent className="max-w-2xl max-h-[80vh]">
          <DialogHeader>
            <DialogTitle>Edit App Mappings for {categoryName}</DialogTitle>
            <DialogDescription>
              Add or remove applications that should be categorized as "
              {categoryName}". You can use patterns like "App Name|Alternative
              Name" for flexible matching.
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4 py-4">
            <div className="flex gap-2">
              <Input
                placeholder="Add new app (e.g., 'Visual Studio Code|VS Code')"
                value={newApp}
                onChange={(e) => setNewApp(e.target.value)}
                onKeyDown={handleKeyDown}
                className="flex-1"
              />
              <Button onClick={addApp} disabled={!newApp.trim()}>
                <Plus className="w-4 h-4" />
              </Button>
            </div>

            <div className="border rounded-md p-4 max-h-60 overflow-y-auto">
              <div className="space-y-2">
                {editingApps.length === 0 ? (
                  <p className="text-sm text-muted-foreground text-center py-4">
                    No apps mapped to this category yet.
                  </p>
                ) : (
                  editingApps.map((app, index) => (
                    <div
                      key={index}
                      className="flex items-center justify-between bg-secondary p-2 rounded"
                    >
                      <span className="text-sm">{app}</span>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => removeApp(index)}
                      >
                        <X className="w-4 h-4" />
                      </Button>
                    </div>
                  ))
                )}
              </div>
            </div>

            <div className="text-xs text-muted-foreground">
              <p>
                <strong>Tips:</strong>
              </p>
              <ul className="list-disc list-inside space-y-1 mt-1">
                <li>
                  Use "|" to separate alternative names (e.g., "Chrome|Google
                  Chrome")
                </li>
                <li>App names are case-insensitive during matching</li>
                <li>The first name in the pattern is preferred for display</li>
              </ul>
            </div>
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={() => setIsOpen(false)}>
              Cancel
            </Button>
            <Button onClick={handleSave}>Save Changes</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
