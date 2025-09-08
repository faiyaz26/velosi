import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useState } from "react";
import {
  format,
  subDays,
  addDays,
  isAfter,
  startOfDay,
  differenceInDays,
} from "date-fns";
import { Calendar, ChevronLeft, ChevronRight, RefreshCw } from "lucide-react";

interface DateRange {
  startDate: Date;
  endDate: Date;
}

interface DateRangeSelectorProps {
  selectedRange: DateRange;
  onRangeChange: (range: DateRange) => void;
  onRefresh?: () => void;
  loading?: boolean;
  maxDays?: number; // Maximum number of days in range
}

export function DateRangeSelector({
  selectedRange,
  onRangeChange,
  onRefresh,
  loading = false,
  maxDays = 30, // Default to 30 days max
}: DateRangeSelectorProps) {
  const [startInputDate, setStartInputDate] = useState(
    format(selectedRange.startDate, "yyyy-MM-dd")
  );
  const [endInputDate, setEndInputDate] = useState(
    format(selectedRange.endDate, "yyyy-MM-dd")
  );

  const handlePreviousRange = () => {
    const rangeDays = differenceInDays(
      selectedRange.endDate,
      selectedRange.startDate
    );
    const newStartDate = subDays(selectedRange.startDate, rangeDays + 1);
    const newEndDate = subDays(selectedRange.endDate, rangeDays + 1);

    const newRange = { startDate: newStartDate, endDate: newEndDate };
    onRangeChange(newRange);
    setStartInputDate(format(newStartDate, "yyyy-MM-dd"));
    setEndInputDate(format(newEndDate, "yyyy-MM-dd"));
  };

  const handleNextRange = () => {
    const rangeDays = differenceInDays(
      selectedRange.endDate,
      selectedRange.startDate
    );
    const newStartDate = addDays(selectedRange.startDate, rangeDays + 1);
    const newEndDate = addDays(selectedRange.endDate, rangeDays + 1);
    const today = startOfDay(new Date());

    // Don't allow future dates
    if (!isAfter(newEndDate, today)) {
      const newRange = { startDate: newStartDate, endDate: newEndDate };
      onRangeChange(newRange);
      setStartInputDate(format(newStartDate, "yyyy-MM-dd"));
      setEndInputDate(format(newEndDate, "yyyy-MM-dd"));
    }
  };

  const handleStartDateChange = (value: string) => {
    setStartInputDate(value);
    const date = new Date(value);
    if (!isNaN(date.getTime())) {
      const today = startOfDay(new Date());
      if (!isAfter(date, today)) {
        // Ensure end date is not before start date and within max days
        let newEndDate = selectedRange.endDate;
        if (isAfter(date, newEndDate)) {
          newEndDate = date;
        }

        // Check if range exceeds max days
        const daysDiff = differenceInDays(newEndDate, date);
        if (daysDiff > maxDays) {
          newEndDate = addDays(date, maxDays);
        }

        const newRange = { startDate: date, endDate: newEndDate };
        onRangeChange(newRange);
        setEndInputDate(format(newEndDate, "yyyy-MM-dd"));
      }
    }
  };

  const handleEndDateChange = (value: string) => {
    setEndInputDate(value);
    const date = new Date(value);
    if (!isNaN(date.getTime())) {
      const today = startOfDay(new Date());
      if (!isAfter(date, today)) {
        // Ensure start date is not after end date and within max days
        let newStartDate = selectedRange.startDate;
        if (isAfter(newStartDate, date)) {
          newStartDate = date;
        }

        // Check if range exceeds max days
        const daysDiff = differenceInDays(date, newStartDate);
        if (daysDiff > maxDays) {
          newStartDate = subDays(date, maxDays);
        }

        const newRange = { startDate: newStartDate, endDate: date };
        onRangeChange(newRange);
        setStartInputDate(format(newStartDate, "yyyy-MM-dd"));
      }
    }
  };

  const handleTodayClick = () => {
    const today = startOfDay(new Date());
    const newRange = { startDate: today, endDate: today };
    onRangeChange(newRange);
    setStartInputDate(format(today, "yyyy-MM-dd"));
    setEndInputDate(format(today, "yyyy-MM-dd"));
  };

  const handleLastWeekClick = () => {
    const today = startOfDay(new Date());
    const lastWeek = subDays(today, 6);
    const newRange = { startDate: lastWeek, endDate: today };
    onRangeChange(newRange);
    setStartInputDate(format(lastWeek, "yyyy-MM-dd"));
    setEndInputDate(format(today, "yyyy-MM-dd"));
  };

  const handleLastMonthClick = () => {
    const today = startOfDay(new Date());
    const lastMonth = subDays(today, 29);
    const newRange = { startDate: lastMonth, endDate: today };
    onRangeChange(newRange);
    setStartInputDate(format(lastMonth, "yyyy-MM-dd"));
    setEndInputDate(format(today, "yyyy-MM-dd"));
  };

  const rangeDays =
    differenceInDays(selectedRange.endDate, selectedRange.startDate) + 1;
  const isNextDisabled = isAfter(
    addDays(selectedRange.endDate, 1),
    startOfDay(new Date())
  );

  return (
    <Card>
      <CardHeader className="pb-3">
        <CardTitle className="flex items-center gap-2">
          <Calendar className="h-5 w-5" />
          Select Date Range
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Quick Selection Buttons */}
        <div className="flex flex-wrap gap-2">
          <Button variant="outline" size="sm" onClick={handleTodayClick}>
            Today
          </Button>
          <Button variant="outline" size="sm" onClick={handleLastWeekClick}>
            Last 7 Days
          </Button>
          <Button variant="outline" size="sm" onClick={handleLastMonthClick}>
            Last 30 Days
          </Button>
        </div>

        {/* Date Range Inputs */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div className="space-y-2">
            <label className="text-sm font-medium">Start Date</label>
            <Input
              type="date"
              value={startInputDate}
              onChange={(e) => handleStartDateChange(e.target.value)}
              max={format(new Date(), "yyyy-MM-dd")}
            />
          </div>
          <div className="space-y-2">
            <label className="text-sm font-medium">End Date</label>
            <Input
              type="date"
              value={endInputDate}
              onChange={(e) => handleEndDateChange(e.target.value)}
              max={format(new Date(), "yyyy-MM-dd")}
            />
          </div>
        </div>

        {/* Navigation and Info */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={handlePreviousRange}
              disabled={loading}
            >
              <ChevronLeft className="h-4 w-4" />
              Previous
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={handleNextRange}
              disabled={loading || isNextDisabled}
            >
              Next
              <ChevronRight className="h-4 w-4" />
            </Button>
          </div>

          <div className="flex items-center gap-2">
            <span className="text-sm text-muted-foreground">
              {rangeDays} day{rangeDays !== 1 ? "s" : ""} selected
            </span>
            {onRefresh && (
              <Button
                variant="outline"
                size="sm"
                onClick={onRefresh}
                disabled={loading}
              >
                <RefreshCw
                  className={`h-4 w-4 ${loading ? "animate-spin" : ""}`}
                />
              </Button>
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

export type { DateRange };
