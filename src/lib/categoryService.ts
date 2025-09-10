import { invoke } from "@tauri-apps/api/core";

interface Category {
  id: string;
  name: string;
  color: string;
  parent_id?: string;
  created_at: string;
  updated_at: string;
}

interface AppMapping {
  category_id: string;
  apps: string[];
}

interface MappingsConfig {
  mappings: AppMapping[];
}

class CategoryService {
  private categories: Category[] = [];
  private appMappings: Map<string, string> = new Map();
  private initialized = false;

  async initialize(): Promise<void> {
    if (this.initialized) return;

    try {
      // Load categories and mappings from Tauri backend
      const categories: Category[] = await invoke("load_categories");
      const mappingsConfig: MappingsConfig = await invoke("get_app_mappings");

      this.categories = categories;

      // Build app name to category mapping
      this.buildAppMappings(mappingsConfig.mappings);

      this.initialized = true;
      console.log(
        "Category service initialized with",
        this.categories.length,
        "categories and",
        this.appMappings.size,
        "app mappings"
      );
    } catch (error) {
      console.error("Failed to initialize category service:", error);
      // Fallback to default categories
      this.loadDefaultCategories();
      this.initialized = true;
    }
  }

  private buildAppMappings(mappings: AppMapping[]): void {
    this.appMappings.clear();

    mappings.forEach((mapping) => {
      mapping.apps.forEach((appString) => {
        // Split by | to handle multiple app name variations
        const appNames = appString.split("|").map((name) => name.trim());
        appNames.forEach((appName) => {
          // Store both exact match and lowercase for case-insensitive matching
          this.appMappings.set(appName, mapping.category_id);
          this.appMappings.set(appName.toLowerCase(), mapping.category_id);
        });
      });
    });
  }

  private loadDefaultCategories(): void {
    // Fallback categories if loading fails
    const now = new Date().toISOString();
    this.categories = [
      {
        id: "development",
        name: "Development",
        color: "#3b82f6",
        parent_id: undefined,
        created_at: now,
        updated_at: now,
      },
      {
        id: "productive",
        name: "Productive",
        color: "#10b981",
        parent_id: undefined,
        created_at: now,
        updated_at: now,
      },
      {
        id: "communication",
        name: "Communication",
        color: "#f59e0b",
        parent_id: undefined,
        created_at: now,
        updated_at: now,
      },
      {
        id: "social",
        name: "Social",
        color: "#ef4444",
        parent_id: undefined,
        created_at: now,
        updated_at: now,
      },
      {
        id: "entertainment",
        name: "Entertainment",
        color: "#8b5cf6",
        parent_id: undefined,
        created_at: now,
        updated_at: now,
      },
      {
        id: "unknown",
        name: "Unknown",
        color: "#6b7280",
        parent_id: undefined,
        created_at: now,
        updated_at: now,
      },
    ];
  }

  getCategories(): Category[] {
    return this.categories;
  }

  getCategoryById(id: string): Category | undefined {
    return this.categories.find((cat) => cat.id === id);
  }

  getCategoryByAppName(appName: string): Category {
    if (!this.initialized) {
      console.warn("Category service not initialized");
      return this.getUnknownCategory();
    }

    // Try exact match first
    let categoryId = this.appMappings.get(appName);

    // Try lowercase match if exact match fails
    if (!categoryId) {
      categoryId = this.appMappings.get(appName.toLowerCase());
    }

    // Try partial matching for app names that might include version numbers or extra text
    if (!categoryId) {
      for (const [mappedName, catId] of this.appMappings.entries()) {
        if (
          appName.toLowerCase().includes(mappedName.toLowerCase()) ||
          mappedName.toLowerCase().includes(appName.toLowerCase())
        ) {
          categoryId = catId;
          break;
        }
      }
    }

    const category = categoryId ? this.getCategoryById(categoryId) : undefined;
    return category || this.getUnknownCategory();
  }

  private getUnknownCategory(): Category {
    return (
      this.getCategoryById("unknown") || {
        id: "unknown",
        name: "Unknown",
        color: "#6b7280",
        parent_id: undefined,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      }
    );
  }

  getCategoryColor(categoryId: string): string {
    const category = this.getCategoryById(categoryId);
    return category?.color || "#6b7280";
  }

  getCategoryName(categoryId: string): string {
    const category = this.getCategoryById(categoryId);
    return category?.name || "Unknown";
  }

  // Method to get category in the format expected by the current app
  getCategoryObject(appName: string): { [key: string]: any } {
    const category = this.getCategoryByAppName(appName);
    return { [category.name]: category };
  }

  // Method to check if service is initialized
  isInitialized(): boolean {
    return this.initialized;
  }

  // Method to wait for initialization
  async waitForInitialization(): Promise<void> {
    if (this.initialized) return;

    // Wait for initialization with a timeout
    let attempts = 0;
    const maxAttempts = 50; // 5 seconds with 100ms intervals

    while (!this.initialized && attempts < maxAttempts) {
      await new Promise((resolve) => setTimeout(resolve, 100));
      attempts++;
    }

    if (!this.initialized) {
      console.warn(
        "Category service initialization timeout, using fallback categories"
      );
      this.loadDefaultCategories();
      this.initialized = true;
    }
  }
}

// Export singleton instance
export const categoryService = new CategoryService();
