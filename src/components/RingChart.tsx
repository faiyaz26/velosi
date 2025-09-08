import { PieChart, Pie, Cell, ResponsiveContainer } from "recharts";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Clock } from "lucide-react";

interface RingChartData {
  name: string;
  value: number;
  percentage?: number;
  color: string;
}

interface RingChartProps {
  data: RingChartData[];
  title: string;
  description: string;
  centerText?: string;
  centerSubText?: string;
  emptyStateText?: string;
  emptyStateSubText?: string;
}

export const RingChart = ({
  data,
  title,
  description,
  centerText,
  centerSubText,
  emptyStateText = "No data available",
  emptyStateSubText = "Start tracking to see activity",
}: RingChartProps) => {
  const formatDuration = (seconds: number): string => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);

    if (hours > 0) {
      return `${hours}h ${minutes}m`;
    }
    return `${minutes}m`;
  };

  // Calculate total for center text if not provided
  const totalSeconds = data.reduce((sum, item) => sum + item.value, 0);
  const displayCenterText = centerText || formatDuration(totalSeconds);

  return (
    <Card>
      <CardHeader className="pb-3">
        <CardTitle className="flex items-center gap-2">{title}</CardTitle>
        <CardDescription>{description}</CardDescription>
      </CardHeader>
      <CardContent className="pb-4">
        {data.length > 0 ? (
          <div className="flex justify-center">
            <ResponsiveContainer width="100%" height={250} maxHeight={250}>
              <PieChart>
                <Pie
                  data={data}
                  cx="50%"
                  cy="50%"
                  labelLine={false}
                  label={({ name, percentage, value }) => {
                    const percent =
                      percentage || ((value || 0) / totalSeconds) * 100;
                    return `${name} ${percent.toFixed(1)}%`;
                  }}
                  outerRadius={80}
                  innerRadius={50}
                  fill="#8884d8"
                  dataKey="value"
                >
                  {data.map((entry, index) => (
                    <Cell key={`cell-${index}`} fill={entry.color} />
                  ))}
                </Pie>
                {/* Center Text */}
                <text
                  x="50%"
                  y="45%"
                  textAnchor="middle"
                  dominantBaseline="middle"
                  className="fill-current text-lg font-bold"
                >
                  {displayCenterText}
                </text>
                <text
                  x="50%"
                  y="55%"
                  textAnchor="middle"
                  dominantBaseline="middle"
                  className="fill-current text-xs text-muted-foreground"
                >
                  {centerSubText || "Total Time"}
                </text>
              </PieChart>
            </ResponsiveContainer>
          </div>
        ) : (
          <div className="h-[250px] flex flex-col items-center justify-center text-muted-foreground">
            <Clock className="h-10 w-10 mb-2" />
            <p className="text-sm">{emptyStateText}</p>
            <p className="text-xs">{emptyStateSubText}</p>
          </div>
        )}
      </CardContent>
    </Card>
  );
};
