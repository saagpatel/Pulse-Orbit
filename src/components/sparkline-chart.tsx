import { useMemo } from "react";
import { Area, AreaChart, ResponsiveContainer, YAxis } from "recharts";
import type { MetricSnapshot } from "../types";

interface SparklineChartProps {
	data: MetricSnapshot[];
	accessor: (snapshot: MetricSnapshot) => number;
	max?: number;
	height?: number;
	color?: string;
}

export function SparklineChart({
	data,
	accessor,
	max = 100,
	height = 60,
	color = "#6366f1",
}: SparklineChartProps) {
	const chartData = useMemo(
		() => data.map((s) => ({ v: accessor(s) })),
		[data, accessor],
	);

	if (chartData.length < 2) return null;

	return (
		<ResponsiveContainer width="100%" height={height}>
			<AreaChart
				data={chartData}
				margin={{ top: 0, right: 0, bottom: 0, left: 0 }}
			>
				<YAxis domain={[0, max]} hide />
				<Area
					type="monotone"
					dataKey="v"
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
