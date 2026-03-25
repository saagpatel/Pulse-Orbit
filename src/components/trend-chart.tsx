import { useMemo } from "react";
import {
	Area,
	AreaChart,
	CartesianGrid,
	ResponsiveContainer,
	XAxis,
	YAxis,
} from "recharts";
import { formatPercent } from "../lib/format";
import type { HistoryRow } from "../types";

interface TrendChartProps {
	data: HistoryRow[];
	height?: number;
	yMax?: number;
	color?: string;
	formatY?: (v: number) => string;
}

function formatTime(windowStart: string): string {
	// Parse "YYYY-MM-DD HH:MM:SS" → "HH:MM"
	const parts = windowStart.split(" ");
	const time = parts[1];
	return time ? time.slice(0, 5) : windowStart;
}

export function TrendChart({
	data,
	height = 120,
	yMax = 100,
	color = "#6366f1",
	formatY = formatPercent,
}: TrendChartProps) {
	const chartData = useMemo(
		() =>
			data.map((row) => ({
				time: formatTime(row.window_start),
				avg: row.value_avg,
				max: row.value_max,
			})),
		[data],
	);

	if (chartData.length === 0) {
		return (
			<div
				className="flex items-center justify-center text-text-secondary text-[10px]"
				style={{ height }}
			>
				Collecting data...
			</div>
		);
	}

	return (
		<ResponsiveContainer width="100%" height={height}>
			<AreaChart
				data={chartData}
				margin={{ top: 4, right: 4, bottom: 0, left: 0 }}
			>
				<CartesianGrid
					strokeDasharray="3 3"
					stroke="#2d3148"
					horizontal
					vertical={false}
				/>
				<XAxis
					dataKey="time"
					tick={{ fontSize: 9, fill: "#64748b" }}
					tickLine={false}
					axisLine={false}
					interval="preserveStartEnd"
				/>
				<YAxis
					domain={[0, yMax]}
					tick={{ fontSize: 9, fill: "#64748b" }}
					tickLine={false}
					axisLine={false}
					tickFormatter={formatY}
					width={40}
				/>
				<Area
					type="monotone"
					dataKey="avg"
					stroke={color}
					strokeWidth={1.5}
					fill={`${color}33`}
					dot={false}
					isAnimationActive={false}
				/>
			</AreaChart>
		</ResponsiveContainer>
	);
}
