import { type ClassValue, clsx } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

// Common category utility functions
interface ActivityCategory {
  Productive?: null;
  Social?: null;
  Entertainment?: null;
  Development?: null;
  Communication?: null;
  Unknown?: null;
}

export function getCategoryColor(
  category: ActivityCategory | string | any,
  categoryService: any,
  isInitialized: boolean
): string {
  let categoryKey = "unknown";

  // Handle both string and object category formats
  if (typeof category === "string") {
    categoryKey = category;
  } else if (typeof category === "object" && category) {
    const keys = Object.keys(category);
    categoryKey = keys.length > 0 ? keys[0] : "unknown";
  }

  if (isInitialized && categoryService) {
    const categoryInfo = categoryService.getCategoryById(
      categoryKey.toLowerCase()
    );
    if (categoryInfo) {
      return categoryInfo.color;
    }
  }

  // Fallback colors (hex values for charts)
  const fallbackColors: { [key: string]: string } = {
    Development: "#3B82F6",     // Blue
    Productive: "#10B981",      // Green
    Communication: "#F59E0B",   // Yellow
    Social: "#EF4444",          // Red
    Entertainment: "#8B5CF6",   // Purple
    Unknown: "#64748b",         // Gray
  };

  return fallbackColors[categoryKey] || fallbackColors.Unknown;
}

export function getCategoryName(
  category: ActivityCategory | string | any,
  categoryService: any,
  isInitialized: boolean
): string {
  let categoryKey = "unknown";

  // Handle both string and object category formats
  if (typeof category === "string") {
    categoryKey = category;
  } else if (typeof category === "object" && category) {
    const keys = Object.keys(category);
    categoryKey = keys.length > 0 ? keys[0] : "unknown";
  }

  if (isInitialized && categoryService) {
    const categoryInfo = categoryService.getCategoryById(
      categoryKey.toLowerCase()
    );
    if (categoryInfo) {
      return categoryInfo.name;
    }
  }

  // Fallback to the original enum name (e.g., "Social", "Development")
  return categoryKey;
}

export function getCategoryColorClass(
  category: ActivityCategory | string | any,
  categoryService: any,
  isInitialized: boolean
): string {
  const hexColor = getCategoryColor(category, categoryService, isInitialized);
  
  // Convert hex color to Tailwind bg class
  const colorMap: { [key: string]: string } = {
    "#3B82F6": "bg-blue-500",
    "#10B981": "bg-green-500",
    "#F59E0B": "bg-yellow-500",
    "#EF4444": "bg-red-500",
    "#8B5CF6": "bg-purple-500",
    "#64748b": "bg-gray-500",
  };

  return colorMap[hexColor] || `bg-[${hexColor}]`;
}
