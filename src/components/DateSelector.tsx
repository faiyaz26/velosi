import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useState } from "react";
import { format, subDays, isAfter, startOfDay } from "date-fns";
import { Calendar, ChevronLeft, ChevronRight } from "lucide-react";

interface DateSelectorProps {
  selectedDate: Date;
  onDateChange: (date: Date) => void;
}

export function DateSelector({
  selectedDate,
  onDateChange,
}: DateSelectorProps) {
  const [inputDate, setInputDate] = useState(
    format(selectedDate, "yyyy-MM-dd")
  );

  const handlePreviousDay = () => {
    const previousDay = subDays(selectedDate, 1);
    onDateChange(previousDay);
    setInputDate(format(previousDay, "yyyy-MM-dd"));
  };

  const handleNextDay = () => {
    const nextDay = subDays(selectedDate, -1);
    const today = startOfDay(new Date());

    // Don't allow future dates
    if (!isAfter(nextDay, today)) {
      onDateChange(nextDay);
      setInputDate(format(nextDay, "yyyy-MM-dd"));
    }
  };

  const handleDateInputChange = (value: string) => {
    setInputDate(value);
    const date = new Date(value);
    if (!isNaN(date.getTime())) {
      const today = startOfDay(new Date());
      if (!isAfter(date, today)) {
        onDateChange(date);
      }
    }
  };

  const isToday =
    format(selectedDate, "yyyy-MM-dd") === format(new Date(), "yyyy-MM-dd");
  const isTomorrow = isAfter(subDays(selectedDate, -1), startOfDay(new Date()));

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Calendar className="h-5 w-5" />
          Select Date
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" onClick={handlePreviousDay}>
            <ChevronLeft className="h-4 w-4" />
          </Button>

          <Input
            type="date"
            value={inputDate}
            onChange={(e) => handleDateInputChange(e.target.value)}
            max={format(new Date(), "yyyy-MM-dd")}
            className="flex-1"
          />

          <Button
            variant="outline"
            size="sm"
            onClick={handleNextDay}
            disabled={isTomorrow}
          >
            <ChevronRight className="h-4 w-4" />
          </Button>
        </div>

        <div className="mt-2 text-center">
          <p className="text-sm text-muted-foreground">
            {isToday ? "Today" : format(selectedDate, "EEEE, MMMM d, yyyy")}
          </p>
        </div>

        <div className="mt-4 flex gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => {
              const today = new Date();
              onDateChange(today);
              setInputDate(format(today, "yyyy-MM-dd"));
            }}
            disabled={isToday}
          >
            Today
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={() => {
              const yesterday = subDays(new Date(), 1);
              onDateChange(yesterday);
              setInputDate(format(yesterday, "yyyy-MM-dd"));
            }}
          >
            Yesterday
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={() => {
              const lastWeek = subDays(new Date(), 7);
              onDateChange(lastWeek);
              setInputDate(format(lastWeek, "yyyy-MM-dd"));
            }}
          >
            Last Week
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
