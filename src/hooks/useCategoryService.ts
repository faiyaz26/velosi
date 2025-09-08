import { useEffect, useState } from "react";
import { categoryService } from "@/lib/categoryService";

export function useCategoryService() {
  const [isInitialized, setIsInitialized] = useState(false);

  useEffect(() => {
    const initializeService = async () => {
      await categoryService.waitForInitialization();
      setIsInitialized(true);
    };

    initializeService();
  }, []);

  return {
    isInitialized,
    categoryService,
  };
}
