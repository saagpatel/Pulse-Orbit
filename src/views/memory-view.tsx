import { useCallback } from "react";
import { MetricBar } from "../components/metric-bar";
import { SparklineChart } from "../components/sparkline-chart";
import { formatBytes, formatPercent } from "../lib/format";
import type { MetricSnapshot } from "../types";

interface MemoryViewProps {
	current: MetricSnapshot | null;
	history: MetricSnapshot[];
}

const memAccessor = (s: MetricSnapshot) =>
	(s.memory.used_bytes / s.memory.total_bytes) * 100;

export function MemoryView({ current, history }: MemoryViewProps) {
	if (!current) {
		return (
			<div className="p-4 text-text-secondary text-xs">Waiting for data...</div>
		);
	}

	const { memory } = current;
	const memPercent = (memory.used_bytes / memory.total_bytes) * 100;

	const pressureStyle = useCallback(
		(pressure: "normal" | "warn" | "critical") => {
			switch (pressure) {
				case "normal":
					return "bg-status-success/20 text-status-success";
				case "warn":
					return "bg-status-warning/20 text-status-warning";
				case "critical":
					return "bg-status-critical/20 text-status-critical";
			}
		},
		[],
	);

	return (
		<div className="p-4 space-y-4">
			{/* Memory Usage */}
			<div>
				<MetricBar
					label="Used"
					value={memory.used_bytes}
					max={memory.total_bytes}
					format={(v) =>
						`${formatBytes(v)} / ${formatBytes(memory.total_bytes)}`
					}
				/>
				<div className="mt-1 flex justify-between text-[10px] text-text-secondary">
					<span>{formatPercent(memPercent)}</span>
					<span
						className={`px-1.5 py-0.5 rounded-full text-[9px] font-bold uppercase ${pressureStyle(memory.pressure)}`}
					>
						{memory.pressure}
					</span>
				</div>
				<div className="mt-2">
					<SparklineChart data={history} accessor={memAccessor} />
				</div>
			</div>

			{/* Swap */}
			{memory.swap_total_bytes > 0 && (
				<div className="bg-panel-card rounded-lg p-3 space-y-2">
					<h4 className="text-[10px] font-bold text-text-secondary uppercase tracking-wider">
						Swap
					</h4>
					<MetricBar
						label="Swap Used"
						value={memory.swap_used_bytes}
						max={memory.swap_total_bytes}
						format={(v) =>
							`${formatBytes(v)} / ${formatBytes(memory.swap_total_bytes)}`
						}
						color="bg-status-warning"
					/>
				</div>
			)}

			{/* Summary */}
			<div className="bg-panel-card rounded-lg p-3 grid grid-cols-2 gap-2 text-xs">
				<div>
					<span className="text-text-secondary">Total</span>
					<p className="font-mono font-bold">
						{formatBytes(memory.total_bytes)}
					</p>
				</div>
				<div>
					<span className="text-text-secondary">Used</span>
					<p className="font-mono font-bold">
						{formatBytes(memory.used_bytes)}
					</p>
				</div>
				<div>
					<span className="text-text-secondary">Available</span>
					<p className="font-mono font-bold">
						{formatBytes(memory.total_bytes - memory.used_bytes)}
					</p>
				</div>
				<div>
					<span className="text-text-secondary">Swap</span>
					<p className="font-mono font-bold">
						{memory.swap_total_bytes > 0
							? formatBytes(memory.swap_used_bytes)
							: "None"}
					</p>
				</div>
			</div>
		</div>
	);
}
