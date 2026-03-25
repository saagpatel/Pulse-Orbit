import { useState } from "react";
import { KillModal } from "../components/kill-modal";
import { formatBytes, formatPercent } from "../lib/format";
import type { MetricSnapshot, ProcessInfo } from "../types";

interface ProcessViewProps {
	current: MetricSnapshot | null;
}

export function ProcessView({ current }: ProcessViewProps) {
	const [killTarget, setKillTarget] = useState<ProcessInfo | null>(null);

	if (!current) {
		return (
			<div className="p-4 text-text-secondary text-xs">Waiting for data...</div>
		);
	}

	return (
		<div className="p-4">
			<table className="w-full text-xs">
				<thead>
					<tr className="text-text-secondary text-[10px] uppercase tracking-wider">
						<th className="text-left pb-2 font-bold">Name</th>
						<th className="text-right pb-2 font-bold w-14">PID</th>
						<th className="text-right pb-2 font-bold w-14">CPU</th>
						<th className="text-right pb-2 font-bold w-16">RAM</th>
						<th className="w-8 pb-2" />
					</tr>
				</thead>
				<tbody>
					{current.processes.map((proc) => (
						<tr
							key={proc.pid}
							className="border-t border-panel-border/50 hover:bg-panel-card/50 transition-colors duration-100"
						>
							<td className="py-1.5 truncate max-w-[140px]" title={proc.name}>
								{proc.name}
							</td>
							<td className="py-1.5 text-right font-mono text-text-secondary">
								{proc.pid}
							</td>
							<td className="py-1.5 text-right font-mono">
								{formatPercent(proc.cpu_percent)}
							</td>
							<td className="py-1.5 text-right font-mono text-text-secondary">
								{formatBytes(proc.memory_bytes)}
							</td>
							<td className="py-1.5 text-right">
								<button
									type="button"
									onClick={() => setKillTarget(proc)}
									className="text-text-secondary hover:text-status-critical transition-colors duration-150 text-[10px]"
									title={`Kill ${proc.name}`}
								>
									✕
								</button>
							</td>
						</tr>
					))}
				</tbody>
			</table>

			{killTarget && (
				<KillModal process={killTarget} onClose={() => setKillTarget(null)} />
			)}
		</div>
	);
}
