import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardHeader, CardContent, CardTitle } from "./ui/card";
import { Button } from "./ui/button";
import { Input } from "./ui/input";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "./ui/dialog";
import { Plus, Folder, Trash2 } from "lucide-react";

interface Category {
  id: string;
  name: string;
  color: string;
  description?: string;
  parent_id?: string;
  subcategories?: Category[];
}

interface AppMapping {
  [categoryId: string]: {
    apps: string[];
  };
}

interface UrlMapping {
  [categoryId: string]: {
    urls: string[];
  };
}

function Categorization() {
  const [categories, setCategories] = useState<Category[]>([]);
  const [appMappings, setAppMappings] = useState<AppMapping>({});
  const [urlMappings, setUrlMappings] = useState<UrlMapping>({});
  const [isLoading, setIsLoading] = useState(true);
  const [activeTab, setActiveTab] = useState<"apps" | "urls">("apps");
  const [isAddCategoryOpen, setIsAddCategoryOpen] = useState(false);
  const [isAddSubcategoryOpen, setIsAddSubcategoryOpen] = useState(false);
  const [newCategory, setNewCategory] = useState({
    name: "",
    color: "#3b82f6",
  });

  // Search states for App mappings tab
  const [categorySearch, _setCategorySearch] = useState("");
  const [appSearch, _setAppSearch] = useState("");

  // Two-panel state for App mappings
  const [selectedCategory, setSelectedCategory] = useState<Category | null>(
    null
  );
  const [newAppName, setNewAppName] = useState("");
  const [isAddingApp, setIsAddingApp] = useState(false);
  const [newUrlName, setNewUrlName] = useState("");
  const [isAddingUrl, setIsAddingUrl] = useState(false);
  const [selectedParentCategory, setSelectedParentCategory] = useState<
    string | null
  >(null);

  useEffect(() => {
    console.log("Categories component mounted, loading data...");
    loadCategories();
    loadAppMappings();
    loadUrlMappings();
  }, []);

  const loadCategories = async () => {
    try {
      setIsLoading(true);
      console.log("🔄 Loading categories...");
      const response = await invoke<Category[]>("get_categories");
      console.log("✅ Categories response:", response);
      const loadedCategories = response || [];
      console.log(
        "📊 Extracted categories:",
        loadedCategories,
        "Count:",
        loadedCategories.length
      );
      setCategories(loadedCategories);
      console.log("✅ Categories state updated");
    } catch (error) {
      console.error("❌ Failed to load categories:", error);
      setCategories([]);
    } finally {
      setIsLoading(false);
      console.log("✅ Loading finished, isLoading set to false");
    }
  };

  const loadAppMappings = async () => {
    try {
      console.log("🔄 Loading app mappings...");
      const response = await invoke<{
        mappings: Array<{ category: string; apps: string[] }>;
      }>("get_app_mappings");
      console.log("✅ App mappings response:", response);

      // Transform the array format to object format expected by frontend
      const transformedMappings: AppMapping = {};
      if (response?.mappings && Array.isArray(response.mappings)) {
        response.mappings.forEach((mapping) => {
          if (mapping.category && mapping.apps) {
            transformedMappings[mapping.category] = {
              apps: mapping.apps,
            };
          }
        });
      }

      console.log("📊 Transformed app mappings:", transformedMappings);
      setAppMappings(transformedMappings);
    } catch (error) {
      console.error("❌ Failed to load app mappings:", error);
      setAppMappings({});
    }
  };

  const loadUrlMappings = async () => {
    try {
      console.log("🔄 Loading URL mappings...");
      const response = await invoke<{
        mappings: Array<{ category: string; urls: string[] }>;
      }>("get_url_mappings");
      console.log("✅ URL mappings response:", response);

      // Transform the array format to object format expected by frontend
      const transformedMappings: UrlMapping = {};
      if (response?.mappings && Array.isArray(response.mappings)) {
        response.mappings.forEach((mapping) => {
          if (mapping.category && mapping.urls) {
            transformedMappings[mapping.category] = {
              urls: mapping.urls,
            };
          }
        });
      }

      console.log("📊 Transformed URL mappings:", transformedMappings);
      setUrlMappings(transformedMappings);
    } catch (error) {
      console.error("❌ Failed to load URL mappings:", error);
      setUrlMappings({});
    }
  };

  const handleAddCategory = async (isSubcategory = false) => {
    if (!newCategory.name.trim()) {
      alert("Please enter a category name");
      return;
    }

    try {
      const categoryData = {
        name: newCategory.name.trim(),
        color: newCategory.color,
        parent_id: isSubcategory ? selectedParentCategory : null,
      };

      console.log("Adding category:", categoryData);
      await invoke("add_category", {
        name: categoryData.name,
        color: categoryData.color,
      });

      await loadCategories();
      setNewCategory({ name: "", color: "#3b82f6" });
      setIsAddCategoryOpen(false);
      setIsAddSubcategoryOpen(false);
      setSelectedParentCategory(null);
    } catch (error) {
      console.error("Failed to add category:", error);
      alert("Failed to add category: " + error);
    }
  };

  // delete handler intentionally removed; feature not used in UI at the moment

  const handleAddApp = async () => {
    if (!selectedCategory || !newAppName.trim()) {
      return;
    }

    setIsAddingApp(true);
    try {
      console.log(
        `Adding app "${newAppName}" to category "${selectedCategory.id}"`
      );
      await invoke("add_app_mapping", {
        categoryId: selectedCategory.id,
        appName: newAppName.trim(),
      });

      setNewAppName("");
      await loadAppMappings();
    } catch (error) {
      console.error("Failed to add app mapping:", error);
      alert("Failed to add app: " + error);
    } finally {
      setIsAddingApp(false);
    }
  };

  const handleRemoveApp = async (appName: string) => {
    if (!selectedCategory) return;

    try {
      console.log(
        `Removing app "${appName}" from category "${selectedCategory.id}"`
      );
      await invoke("remove_app_mapping", {
        categoryId: selectedCategory.id,
        appName: appName,
      });

      await loadAppMappings();
    } catch (error) {
      console.error("Failed to remove app mapping:", error);
      alert("Failed to remove app: " + error);
    }
  };

  const handleAddUrl = async () => {
    if (!selectedCategory || !newUrlName.trim()) return;

    try {
      setIsAddingUrl(true);
      console.log(
        `Adding URL "${newUrlName}" to category "${selectedCategory.id}"`
      );
      await invoke("add_url_mapping", {
        categoryId: selectedCategory.id,
        urlPattern: newUrlName.trim(),
      });

      setNewUrlName("");
      await loadUrlMappings();
    } catch (error) {
      console.error("Failed to add URL mapping:", error);
      alert("Failed to add URL: " + error);
    } finally {
      setIsAddingUrl(false);
    }
  };

  const handleRemoveUrl = async (urlPattern: string) => {
    if (!selectedCategory) return;

    try {
      console.log(
        `Removing URL "${urlPattern}" from category "${selectedCategory.id}"`
      );
      await invoke("remove_url_mapping", {
        categoryId: selectedCategory.id,
        urlPattern: urlPattern,
      });

      await loadUrlMappings();
    } catch (error) {
      console.error("Failed to remove URL mapping:", error);
      alert("Failed to remove URL: " + error);
    }
  };

  // Helper function to get filtered categories based on search
  const getFilteredCategories = () => {
    // Ensure categories is always an array
    if (!Array.isArray(categories)) {
      return [];
    }

    if (!categorySearch || categorySearch.trim() === "") {
      return categories;
    }

    return categories.filter((category) =>
      category.name.toLowerCase().includes(categorySearch.toLowerCase())
    );
  };

  // Helper function to get apps for the selected category with search filtering
  const getFilteredApps = () => {
    if (!selectedCategory) {
      console.log("🔍 getFilteredApps: No category selected");
      return [];
    }

    console.log("🔍 getFilteredApps: Selected category:", selectedCategory);
    console.log("🔍 getFilteredApps: Available app mappings:", appMappings);

    // Try to find mapping by category ID first, then by name
    const mapping =
      appMappings[selectedCategory.id] || appMappings[selectedCategory.name];
    const apps = mapping?.apps || [];

    console.log("🔍 getFilteredApps: Found mapping:", mapping);
    console.log("🔍 getFilteredApps: Apps:", apps);

    // Ensure apps is an array
    if (!Array.isArray(apps)) {
      return [];
    }

    if (!appSearch || appSearch.trim() === "") {
      return apps;
    }

    return apps.filter((app) =>
      app.toLowerCase().includes(appSearch.toLowerCase())
    );
  };

  // Helper function to get URLs for the selected category with search filtering
  const getFilteredUrls = () => {
    if (!selectedCategory) {
      console.log("🔍 getFilteredUrls: No category selected");
      return [];
    }

    console.log("🔍 getFilteredUrls: Selected category:", selectedCategory);
    console.log("🔍 getFilteredUrls: Available URL mappings:", urlMappings);

    // Try to find mapping by category ID first, then by name
    const mapping =
      urlMappings[selectedCategory.id] || urlMappings[selectedCategory.name];
    const urls = mapping?.urls || [];

    console.log("🔍 getFilteredUrls: Found mapping:", mapping);
    console.log("🔍 getFilteredUrls: URLs:", urls);

    // Ensure urls is an array
    if (!Array.isArray(urls)) {
      return [];
    }

    if (!appSearch || appSearch.trim() === "") {
      return urls;
    }

    return urls.filter((url) =>
      url.toLowerCase().includes(appSearch.toLowerCase())
    );
  };

  return (
    <div className="p-6 max-w-7xl mx-auto">
      <div className="mb-6">
        <div className="flex justify-between items-center">
          <div>
            <h1 className="text-3xl font-bold">Categorization</h1>
            <p className="text-muted-foreground">
              Manage app to category mappings for activity tracking
            </p>
          </div>
          <div className="flex gap-2">
            <Button onClick={() => setIsAddCategoryOpen(true)}>
              <Plus className="w-4 h-4 mr-2" />
              Add Category
            </Button>
          </div>
        </div>
      </div>

      {/* App Mappings Section */}
      <div className="space-y-4">
        {(() => {
          console.log("Render state:", {
            isLoading,
            categories,
            categoriesLength: categories?.length,
            isArray: Array.isArray(categories),
          });
          if (isLoading) {
            return (
              <div className="text-center py-8">
                <p className="text-muted-foreground">Loading categories...</p>
              </div>
            );
          }
          if (!Array.isArray(categories) || categories.length === 0) {
            return (
              <div className="text-center py-8">
                <p className="text-muted-foreground">
                  No categories available. Please add some categories first.
                </p>
              </div>
            );
          }
          return null; // Will render the grid below
        })()}
        {!isLoading && Array.isArray(categories) && categories.length > 0 && (
          <div className="grid grid-cols-2 gap-6 h-[600px]">
            {/* Left Panel - Categories */}
            <Card className="flex flex-col">
              <CardHeader className="pb-3">
                <CardTitle className="text-lg">Categories</CardTitle>
                <p className="text-sm text-muted-foreground">
                  Select a category to manage its app and URL mappings
                </p>
              </CardHeader>
              <CardContent className="flex-1 overflow-y-auto">
                <div className="space-y-2">
                  {getFilteredCategories().map((category) => {
                    // Try to find mappings by category ID first, then by name
                    const appMapping =
                      appMappings[category.id] || appMappings[category.name];
                    const urlMapping =
                      urlMappings[category.id] || urlMappings[category.name];
                    const hasApps =
                      appMapping?.apps && appMapping.apps.length > 0;
                    const hasUrls =
                      urlMapping?.urls && urlMapping.urls.length > 0;
                    const isSelected = selectedCategory?.id === category.id;

                    return (
                      <div
                        key={category.id}
                        className={`p-3 rounded-lg border cursor-pointer transition-colors ${
                          isSelected
                            ? "bg-primary/10 border-primary"
                            : "bg-background hover:bg-muted"
                        }`}
                        onClick={() => setSelectedCategory(category)}
                      >
                        <div className="flex items-center justify-between">
                          <div className="flex items-center gap-3">
                            <div
                              className="w-6 h-6 rounded-md flex items-center justify-center"
                              style={{
                                backgroundColor: category.color || "#3b82f6",
                              }}
                            >
                              <Folder className="w-3 h-3 text-white" />
                            </div>
                            <div>
                              <p className="font-medium">{category.name}</p>
                            </div>
                          </div>
                          <div className="flex items-center gap-2 text-sm text-muted-foreground">
                            {(hasApps || hasUrls) && (
                              <div className="w-2 h-2 bg-green-500 rounded-full" />
                            )}
                          </div>
                        </div>
                      </div>
                    );
                  })}
                </div>
              </CardContent>
            </Card>

            {/* Right Panel - Apps & URLs */}
            <Card className="flex flex-col">
              <CardHeader className="pb-3">
                <div className="flex items-center justify-between">
                  <div>
                    <CardTitle className="text-lg">
                      {selectedCategory
                        ? `Mappings for ${selectedCategory?.name || "Unknown"}`
                        : "Select a Category"}
                    </CardTitle>
                    <p className="text-sm text-muted-foreground">
                      {selectedCategory
                        ? "Add or remove applications and URLs for this category"
                        : "Choose a category from the left panel"}
                    </p>
                  </div>
                  {selectedCategory && (
                    <div
                      className="w-8 h-8 rounded-lg flex items-center justify-center"
                      style={{
                        backgroundColor: selectedCategory?.color || "#3b82f6",
                      }}
                    >
                      <Folder className="w-4 h-4 text-white" />
                    </div>
                  )}
                </div>

                {/* Tabs */}
                {selectedCategory && (
                  <div className="flex gap-1 mt-3 p-1 bg-muted rounded-lg">
                    <Button
                      variant={activeTab === "apps" ? "default" : "ghost"}
                      size="sm"
                      className="flex-1"
                      onClick={() => setActiveTab("apps")}
                    >
                      Apps
                    </Button>
                    <Button
                      variant={activeTab === "urls" ? "default" : "ghost"}
                      size="sm"
                      className="flex-1"
                      onClick={() => setActiveTab("urls")}
                    >
                      URLs
                    </Button>
                  </div>
                )}
              </CardHeader>
              <CardContent className="flex-1 overflow-y-auto">
                {selectedCategory ? (
                  <div className="space-y-4">
                    {/* Apps Tab Content */}
                    {activeTab === "apps" && (
                      <>
                        {/* Add app input */}
                        <div className="flex gap-2">
                          <Input
                            placeholder="Enter app name..."
                            value={newAppName}
                            onChange={(e) => setNewAppName(e.target.value)}
                            onKeyPress={(e) => {
                              if (e.key === "Enter" && newAppName.trim()) {
                                handleAddApp();
                              }
                            }}
                            className="flex-1"
                          />
                          <Button
                            onClick={handleAddApp}
                            disabled={!newAppName.trim() || isAddingApp}
                            size="sm"
                          >
                            <Plus className="w-4 h-4" />
                            {isAddingApp ? "Adding..." : "Add"}
                          </Button>
                        </div>

                        {/* Apps list */}
                        <div className="space-y-2">
                          {getFilteredApps().length === 0 ? (
                            <div className="text-center py-8">
                              <p className="text-muted-foreground text-sm">
                                {appSearch
                                  ? "No apps match your search"
                                  : "No apps assigned to this category yet"}
                              </p>
                            </div>
                          ) : (
                            getFilteredApps().map((app, index) => (
                              <div
                                key={index}
                                className="flex items-center justify-between p-2 bg-secondary rounded-md"
                              >
                                <span className="text-sm">{app}</span>
                                <Button
                                  variant="ghost"
                                  size="sm"
                                  onClick={() => handleRemoveApp(app)}
                                  className="h-6 w-6 p-0 text-destructive hover:text-destructive"
                                >
                                  <Trash2 className="w-3 h-3" />
                                </Button>
                              </div>
                            ))
                          )}
                        </div>
                      </>
                    )}

                    {/* URLs Tab Content */}
                    {activeTab === "urls" && (
                      <>
                        {/* Add URL input */}
                        <div className="flex gap-2">
                          <Input
                            placeholder="Enter URL or domain..."
                            value={newUrlName}
                            onChange={(e) => setNewUrlName(e.target.value)}
                            onKeyPress={(e) => {
                              if (e.key === "Enter" && newUrlName.trim()) {
                                handleAddUrl();
                              }
                            }}
                            className="flex-1"
                          />
                          <Button
                            onClick={handleAddUrl}
                            disabled={!newUrlName.trim() || isAddingUrl}
                            size="sm"
                          >
                            <Plus className="w-4 h-4" />
                            {isAddingUrl ? "Adding..." : "Add"}
                          </Button>
                        </div>

                        {/* URL Pattern Examples */}
                        <div className="p-3 bg-blue-50 border border-blue-200 rounded-md text-sm">
                          <p className="text-blue-700 font-medium">
                            URL Pattern Examples:
                          </p>
                          <p className="text-blue-600 mt-1">
                            • Domain: "github.com" matches all GitHub pages
                            <br />
                            • Subdomain: "docs.github.com" for specific sections
                            <br />• Exact URL: "https://example.com/page" for
                            specific pages
                          </p>
                        </div>

                        {/* URLs list */}
                        <div className="space-y-2">
                          {getFilteredUrls().length === 0 ? (
                            <div className="text-center py-8">
                              <p className="text-muted-foreground text-sm">
                                {appSearch
                                  ? "No URLs match your search"
                                  : "No URLs assigned to this category yet"}
                              </p>
                            </div>
                          ) : (
                            getFilteredUrls().map((url, index) => (
                              <div
                                key={index}
                                className="flex items-center justify-between p-2 bg-secondary rounded-md"
                              >
                                <span className="text-sm">{url}</span>
                                <Button
                                  variant="ghost"
                                  size="sm"
                                  onClick={() => handleRemoveUrl(url)}
                                  className="h-6 w-6 p-0 text-destructive hover:text-destructive"
                                >
                                  <Trash2 className="w-3 h-3" />
                                </Button>
                              </div>
                            ))
                          )}
                        </div>

                        {/* Warning about URL assignments */}
                        <div className="p-3 bg-amber-50 border border-amber-200 rounded-md text-sm">
                          <p className="text-amber-700 font-medium">
                            Note: Each URL pattern can only be assigned to one
                            category.
                          </p>
                          <p className="text-amber-600 mt-1">
                            If you try to add a URL pattern that's already
                            assigned elsewhere, you'll be notified of the
                            conflict.
                          </p>
                        </div>
                      </>
                    )}
                  </div>
                ) : (
                  <div className="flex items-center justify-center h-full">
                    <div className="text-center">
                      <Folder className="w-12 h-12 text-muted-foreground mx-auto mb-3" />
                      <p className="text-muted-foreground">
                        Select a category to view its app and URL mappings
                      </p>
                    </div>
                  </div>
                )}
              </CardContent>
            </Card>
          </div>
        )}
      </div>

      {/* Add Category Dialog */}
      <Dialog open={isAddCategoryOpen} onOpenChange={setIsAddCategoryOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Add New Category</DialogTitle>
            <DialogDescription>
              Create a new category for organizing activities
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div>
              <label className="text-sm font-medium">Name</label>
              <Input
                placeholder="Category name"
                value={newCategory.name}
                onChange={(e) =>
                  setNewCategory((prev) => ({
                    ...prev,
                    name: e.target.value,
                  }))
                }
              />
            </div>
            <div>
              <label className="text-sm font-medium">Color</label>
              <Input
                type="color"
                value={newCategory.color}
                onChange={(e) =>
                  setNewCategory((prev) => ({
                    ...prev,
                    color: e.target.value,
                  }))
                }
                placeholder="#3b82f6"
              />
            </div>
          </div>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => setIsAddCategoryOpen(false)}
            >
              Cancel
            </Button>
            <Button onClick={() => handleAddCategory()}>Add Category</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Add Subcategory Dialog */}
      <Dialog
        open={isAddSubcategoryOpen}
        onOpenChange={setIsAddSubcategoryOpen}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Add Subcategory</DialogTitle>
            <DialogDescription>
              Create a new subcategory under{" "}
              {Array.isArray(categories) && selectedParentCategory
                ? categories.find((c) => c.id === selectedParentCategory)
                    ?.name || "Unknown Category"
                : "Unknown Category"}
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div>
              <label className="text-sm font-medium">Name</label>
              <Input
                placeholder="Subcategory name"
                value={newCategory.name}
                onChange={(e) =>
                  setNewCategory((prev) => ({
                    ...prev,
                    name: e.target.value,
                  }))
                }
              />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="text-sm font-medium">Color</label>
                <Input
                  type="color"
                  value={newCategory.color}
                  onChange={(e) =>
                    setNewCategory((prev) => ({
                      ...prev,
                      color: e.target.value,
                    }))
                  }
                  placeholder="#3b82f6"
                />
              </div>
            </div>
          </div>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => setIsAddSubcategoryOpen(false)}
            >
              Cancel
            </Button>
            <Button onClick={() => handleAddCategory(true)}>
              Add Subcategory
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}

export default Categorization;
