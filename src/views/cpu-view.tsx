import { useCallback } from "react";
import { HistorySection } from "../components/history-section";
import { MetricBar } from "../components/metric-bar";
import { SparklineChart } from "../components/sparkline-chart";
import { formatPercent } from "../lib/format";
import type { MetricSnapshot } from "../types";

interface CpuViewProps {
	current: MetricSnapshot | null;
	history: MetricSnapshot[];
}

const cpuAccessor = (s: MetricSnapshot) => s.cpu.total_percent;

export function CpuView({ current, history }: CpuViewProps) {
	if (!current) {
		return (
			<div className="p-4 text-text-secondary text-xs">Waiting for data...</div>
		);
	}

	const { cpu } = current;
	const coreTypeColor = useCallback(
		(type: "efficiency" | "performance" | "unknown") => {
			switch (type) {
				case "efficiency":
					return "bg-status-success";
				case "performance":
					return "bg-accent";
				default:
					return "bg-text-secondary";
			}
		},
		[],
	);

	return (
		<div className="p-4 space-y-4">
			{/* Total CPU */}
			<div>
				<MetricBar
					label="Total CPU"
					value={cpu.total_percent}
					max={100}
					format={formatPercent}
				/>
				<div className="mt-2">
					<SparklineChart data={history} accessor={cpuAccessor} />
				</div>
			</div>

			{/* E/P Breakdown */}
			{cpu.m_series_breakdown && (
				<div className="bg-panel-card rounded-lg p-3 space-y-2">
					<h4 className="text-[10px] font-bold text-text-secondary uppercase tracking-wider">
						Core Clusters
					</h4>
					<MetricBar
						label={`E-cores (${cpu.m_series_breakdown.efficiency_core_count})`}
						value={cpu.m_series_breakdown.efficiency_cores_avg}
						max={100}
						format={formatPercent}
						color="bg-status-success"
					/>
					<MetricBar
						label={`P-cores (${cpu.m_series_breakdown.performance_core_count})`}
						value={cpu.m_series_breakdown.performance_cores_avg}
						max={100}
						format={formatPercent}
					/>
				</div>
			)}

			{/* Per-core Grid */}
			<div>
				<h4 className="text-[10px] font-bold text-text-secondary uppercase tracking-wider mb-2">
					Per Core
				</h4>
				<div className="grid grid-cols-2 gap-x-3 gap-y-1.5">
					{cpu.per_core.map((core) => (
						<div key={core.core_index} className="flex items-center gap-1.5">
							<span className="text-[10px] text-text-secondary w-5 shrink-0 font-mono">
								{core.core_index}
							</span>
							<MetricBar
								label=""
								value={core.percent}
								max={100}
								format={formatPercent}
								color={coreTypeColor(core.core_type)}
								compact
							/>
						</div>
					))}
				</div>
			</div>

			{/* Frequency */}
			{cpu.frequency_mhz > 0 && (
				<p className="text-[10px] text-text-secondary">
					Frequency: <span className="font-mono">{cpu.frequency_mhz} MHz</span>
				</p>
			)}

			{/* 24h History */}
			<HistorySection metricType="cpu" />
		</div>
	);
}
