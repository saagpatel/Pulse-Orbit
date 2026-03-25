import { useState } from "react";
import { useHistory } from "../hooks/use-history";
import { TrendChart } from "./trend-chart";

interface HistorySectionProps {
	metricType: string;
	iface?: string;
	yMax?: number;
	color?: string;
	formatY?: (v: number) => string;
}

/**
 * Collapsible 24h history chart section.
 * Only fetches data when expanded.
 */
export function HistorySection({
	metricType,
	iface,
	yMax,
	color,
	formatY,
}: HistorySectionProps) {
	const [expanded, setExpanded] = useState(false);

	return (
		<div>
			<button
				type="button"
				onClick={() => setExpanded(!expanded)}
				className="text-[10px] text-text-secondary hover:text-text-primary transition-colors duration-150 flex items-center gap-1"
			>
				<span>{expanded ? "▾" : "▸"}</span>
				<span>24h History</span>
			</button>
			{expanded && (
				<HistoryContent
					metricType={metricType}
					iface={iface}
					yMax={yMax}
					color={color}
					formatY={formatY}
				/>
			)}
		</div>
	);
}

function HistoryContent({
	metricType,
	iface,
	yMax,
	color,
	formatY,
}: HistorySectionProps) {
	const { data } = useHistory(metricType, iface);

	return (
		<div className="mt-2">
			<TrendChart data={data} yMax={yMax} color={color} formatY={formatY} />
		</div>
	);
}
