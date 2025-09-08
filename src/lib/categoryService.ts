import { invoke } from "@tauri-apps/api/core";

interface Category {
  id: string;
  name: string;
  description: string;
  color: string;
  icon: string;
}

interface AppMapping {
  category: string;
  apps: string[];
}

interface CategoriesConfig {
  categories: Category[];
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
      const categoriesConfig: CategoriesConfig = await invoke(
        "load_categories"
      );
      const mappingsConfig: MappingsConfig = await invoke("load_app_mappings");

      this.categories = categoriesConfig.categories;

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
          this.appMappings.set(appName, mapping.category);
          this.appMappings.set(appName.toLowerCase(), mapping.category);
        });
      });
    });
  }

  private loadDefaultCategories(): void {
    // Fallback categories if loading fails
    this.categories = [
      {
        id: "development",
        name: "Development",
        description: "Code editors, IDEs, and development tools",
        color: "#3b82f6",
        icon: "code",
      },
      {
        id: "productive",
        name: "Productive",
        description: "Office applications and productivity tools",
        color: "#10b981",
        icon: "briefcase",
      },
      {
        id: "communication",
        name: "Communication",
        description: "Email, messaging, and communication tools",
        color: "#f59e0b",
        icon: "message-circle",
      },
      {
        id: "social",
        name: "Social",
        description: "Social media and networking applications",
        color: "#ef4444",
        icon: "users",
      },
      {
        id: "entertainment",
        name: "Entertainment",
        description: "Media players, games, and entertainment apps",
        color: "#8b5cf6",
        icon: "play",
      },
      {
        id: "unknown",
        name: "Unknown",
        description: "Uncategorized applications",
        color: "#6b7280",
        icon: "help-circle",
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
        description: "Uncategorized application",
        color: "#6b7280",
        icon: "help-circle",
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
