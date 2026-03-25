import { MetricBar } from "../components/metric-bar";
import { SparklineChart } from "../components/sparkline-chart";
import { formatBytes, formatBytesPerSec } from "../lib/format";
import type { MetricSnapshot } from "../types";

interface DiskViewProps {
	current: MetricSnapshot | null;
	history: MetricSnapshot[];
}

export function DiskView({ current, history }: DiskViewProps) {
	if (!current) {
		return (
			<div className="p-4 text-text-secondary text-xs">Waiting for data...</div>
		);
	}

	const devices = current.disk.devices.filter((d) => d.total_bytes > 0);

	if (devices.length === 0) {
		return (
			<div className="p-4 text-text-secondary text-xs">
				No disk devices detected.
			</div>
		);
	}

	return (
		<div className="p-4 space-y-4">
			{devices.map((device) => (
				<div
					key={device.name}
					className="bg-panel-card rounded-lg p-3 space-y-2"
				>
					<h4 className="text-[10px] font-bold text-text-secondary uppercase tracking-wider">
						{device.name}
					</h4>

					{/* Storage usage */}
					<MetricBar
						label="Storage"
						value={device.used_bytes}
						max={device.total_bytes}
						format={(v) =>
							`${formatBytes(v)} / ${formatBytes(device.total_bytes)}`
						}
					/>

					{/* Throughput */}
					<div className="grid grid-cols-2 gap-2 text-xs">
						<div>
							<span className="text-text-secondary">Read</span>
							<p className="font-mono font-bold">
								{formatBytesPerSec(device.read_bytes_per_sec)}
							</p>
						</div>
						<div>
							<span className="text-text-secondary">Write</span>
							<p className="font-mono font-bold">
								{formatBytesPerSec(device.write_bytes_per_sec)}
							</p>
						</div>
					</div>

					{/* Sparkline: combined read + write */}
					<SparklineChart
						data={history}
						accessor={(s) => {
							const dev = s.disk.devices.find((d) => d.name === device.name);
							return dev ? dev.read_bytes_per_sec + dev.write_bytes_per_sec : 0;
						}}
						max={undefined}
						height={48}
					/>
				</div>
			))}
		</div>
	);
}
