import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { AppMappingEditor } from "./AppMappingEditor";
import { Plus, Trash2, Settings2, Folder } from "lucide-react";

interface Category {
  id: string;
  name: string;
  color: string;
  parent_id?: string;
  subcategories?: Category[];
}

interface AppMapping {
  category: string;
  apps: string[];
}

export function Categories() {
  const [activeTab, setActiveTab] = useState("categories");
  const [categories, setCategories] = useState<Category[]>([]);
  const [appMappings, setAppMappings] = useState<Record<string, AppMapping>>(
    {}
  );
  const [isAddCategoryOpen, setIsAddCategoryOpen] = useState(false);
  const [isAddSubcategoryOpen, setIsAddSubcategoryOpen] = useState(false);
  const [selectedParentCategory, setSelectedParentCategory] = useState<string>(
    ""
  );
  const [newCategory, setNewCategory] = useState({
    name: "",
    color: "#3b82f6",
  });

  useEffect(() => {
    console.log("Categories component mounted, loading data...");
    loadCategories();
    loadAppMappings();
  }, []);

  const loadCategories = async () => {
    try {
      const result = await invoke<any>("get_categories");
      console.log("Raw categories result:", result);

      // Extract categories array from the JSON structure
      const categoriesArray = result?.categories || [];
      console.log("Categories array:", categoriesArray);

      setCategories(categoriesArray);
    } catch (error) {
      console.error("Failed to load categories:", error);
    }
  };

  const loadAppMappings = async () => {
    try {
      const result = await invoke<any>("get_app_mappings");
      console.log("Raw app mappings result:", result);

      // Transform mappings array to Record<string, AppMapping>
      const mappingsArray = result?.mappings || [];
      const mappingsRecord: Record<string, AppMapping> = {};

      mappingsArray.forEach((mapping: any) => {
        if (mapping.category && mapping.apps) {
          mappingsRecord[mapping.category] = {
            category: mapping.category,
            apps: mapping.apps,
          };
        }
      });

      console.log("Transformed app mappings:", mappingsRecord);
      setAppMappings(mappingsRecord);
    } catch (error) {
      console.error("Failed to load app mappings:", error);
    }
  };

  const handleAddCategory = async (isSubcategory = false) => {
    if (!newCategory.name.trim()) return;

    try {
      await invoke("add_category", {
        name: newCategory.name,
        color: newCategory.color,
        parentId: isSubcategory ? selectedParentCategory : undefined,
      });

      setNewCategory({ name: "", color: "#3b82f6" });
      setIsAddCategoryOpen(false);
      setIsAddSubcategoryOpen(false);
      setSelectedParentCategory("");
      await loadCategories();
    } catch (error) {
      console.error("Failed to add category:", error);
    }
  };

  const handleDeleteCategory = async (categoryId: string) => {
    try {
      await invoke("delete_category", { id: categoryId });
      await loadCategories();
    } catch (error) {
      console.error("Failed to delete category:", error);
    }
  };

  const renderCategoryCard = (category: Category) => {
    return (
      <Card key={category.id} className="mb-4">
        <CardHeader className="pb-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div
                className="w-8 h-8 rounded-lg flex items-center justify-center"
                style={{ backgroundColor: category.color }}
              >
                <Folder className="w-4 h-4 text-white" />
              </div>
              <div>
                <CardTitle className="text-lg">{category.name}</CardTitle>
                {category.subcategories &&
                  category.subcategories.length > 0 && (
                    <p className="text-sm text-muted-foreground">
                      {category.subcategories.length} subcategories
                    </p>
                  )}
              </div>
            </div>
            <div className="flex gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => {
                  setSelectedParentCategory(category.id);
                  setIsAddSubcategoryOpen(true);
                }}
              >
                <Plus className="w-4 h-4" />
                Add Sub
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => handleDeleteCategory(category.id)}
              >
                <Trash2 className="w-4 h-4" />
              </Button>
            </div>
          </div>
        </CardHeader>
        {category.subcategories && category.subcategories.length > 0 && (
          <CardContent>
            <div className="grid gap-2">
              {category.subcategories.map((subcat) => (
                <div
                  key={subcat.id}
                  className="flex items-center gap-2 p-2 bg-muted rounded-md"
                >
                  <div
                    className="w-4 h-4 rounded-full"
                    style={{ backgroundColor: subcat.color }}
                  />
                  <span className="text-sm">{subcat.name}</span>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="ml-auto h-6 w-6 p-0"
                    onClick={() => handleDeleteCategory(subcat.id)}
                  >
                    <Trash2 className="w-3 h-3" />
                  </Button>
                </div>
              ))}
            </div>
          </CardContent>
        )}
      </Card>
    );
  };

  return (
    <div className="p-6 max-w-7xl mx-auto">
      <div className="mb-6">
        <h1 className="text-3xl font-bold">Categories</h1>
        <p className="text-muted-foreground">
          Manage categories and app mappings for activity tracking
        </p>
      </div>

      <Tabs
        value={activeTab}
        onValueChange={setActiveTab}
        className="space-y-6"
      >
        <TabsList>
          <TabsTrigger value="categories">Categories</TabsTrigger>
          <TabsTrigger value="mappings">App Mappings</TabsTrigger>
        </TabsList>

        <TabsContent value="categories" className="space-y-4">
          <div className="flex justify-between items-center">
            <h2 className="text-xl font-semibold">Category Management</h2>
            <div className="flex gap-2">
              <Button
                variant="outline"
                onClick={() => {
                  console.log("Manual reload triggered");
                  loadCategories();
                  loadAppMappings();
                }}
              >
                <Settings2 className="w-4 h-4 mr-2" />
                Reload Data
              </Button>
              <Button onClick={() => setIsAddCategoryOpen(true)}>
                <Plus className="w-4 h-4 mr-2" />
                Add Category
              </Button>
            </div>
          </div>

          <div className="space-y-4">
            {categories.length === 0 ? (
              <div className="text-center py-8">
                <p className="text-muted-foreground">
                  No categories found. Debug: {categories.length} categories
                  loaded
                </p>
                <p className="text-sm text-muted-foreground mt-2">
                  Check browser console for loading errors
                </p>
              </div>
            ) : (
              categories.map((category) => renderCategoryCard(category))
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
                <Button onClick={() => handleAddCategory()}>
                  Add Category
                </Button>
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
                  {
                    categories.find((c) => c.id === selectedParentCategory)
                      ?.name
                  }
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
        </TabsContent>

        <TabsContent value="mappings" className="space-y-4">
          <div className="flex flex-col gap-4">
            <div className="flex justify-between items-center">
              <div>
                <h2 className="text-xl font-semibold">
                  App to Category Mappings
                </h2>
                <p className="text-sm text-muted-foreground">
                  Default system mappings that categorize applications
                  automatically. You can override these for custom behavior.
                </p>
              </div>
              <Button variant="outline" onClick={loadAppMappings}>
                <Settings2 className="w-4 h-4 mr-2" />
                Refresh Mappings
              </Button>
            </div>

            <div className="flex items-center gap-4 text-xs">
              <div className="flex items-center gap-2">
                <div className="w-3 h-3 rounded-full bg-blue-100 border-2 border-blue-400" />
                <span>Default Mappings</span>
              </div>
              <div className="flex items-center gap-2">
                <div className="w-3 h-3 rounded-full bg-orange-100 border-2 border-orange-400" />
                <span>Custom Overrides</span>
              </div>
            </div>
          </div>

          {/* App Mappings Display */}
          {Object.entries(appMappings).length === 0 ? (
            <div className="text-center py-8">
              <p className="text-muted-foreground">
                No app mappings available. Debug:{" "}
                {Object.keys(appMappings).length} mappings loaded
              </p>
              <p className="text-sm text-muted-foreground mt-2">
                Check browser console for loading errors
              </p>
            </div>
          ) : (
            <div className="grid gap-4">
              {Object.entries(appMappings).map(([categoryName, mapping]) => {
                const category = categories.find(
                  (c) => c.name === categoryName
                );
                return (
                  <Card key={categoryName} className="relative">
                    <div className="absolute top-3 right-3">
                      <div
                        className="w-3 h-3 rounded-full bg-blue-100 border-2 border-blue-400"
                        title="Default Mapping"
                      />
                    </div>
                    <CardHeader className="pb-3">
                      <div className="flex items-center gap-3">
                        <div
                          className="w-8 h-8 rounded-lg flex items-center justify-center"
                          style={{
                            backgroundColor: category?.color || "#3b82f6",
                          }}
                        >
                          <Folder className="w-4 h-4 text-white" />
                        </div>
                        <CardTitle className="text-lg">
                          {category?.name || categoryName}
                        </CardTitle>
                      </div>
                    </CardHeader>
                    <CardContent>
                      <div className="space-y-3">
                        <p className="text-sm text-muted-foreground">
                          Applications mapped to this category:
                        </p>
                        <div className="flex flex-wrap gap-2">
                          {mapping.apps.map((app: string, index: number) => (
                            <span
                              key={index}
                              className="px-2 py-1 bg-secondary text-secondary-foreground text-xs rounded-md"
                            >
                              {app}
                            </span>
                          ))}
                        </div>
                        <AppMappingEditor
                          category={categoryName}
                          categoryName={category?.name || categoryName}
                          apps={mapping.apps}
                          onUpdate={(
                            category: string,
                            updatedApps: string[]
                          ) => {
                            // Handle app mapping updates
                            console.log(
                              "Updated apps for",
                              category,
                              ":",
                              updatedApps
                            );
                          }}
                        />
                      </div>
                    </CardContent>
                  </Card>
                );
              })}
            </div>
          )}
        </TabsContent>
      </Tabs>
    </div>
  );
}

export default Categories;
