import { SparklineChart } from "../components/sparkline-chart";
import { formatBytes, formatBytesPerSec } from "../lib/format";
import type { MetricSnapshot } from "../types";

interface NetworkViewProps {
	current: MetricSnapshot | null;
	history: MetricSnapshot[];
}

export function NetworkView({ current, history }: NetworkViewProps) {
	if (!current) {
		return (
			<div className="p-4 text-text-secondary text-xs">Waiting for data...</div>
		);
	}

	// Filter out inactive interfaces (no traffic this session)
	const activeInterfaces = current.network.interfaces.filter(
		(i) => i.total_rx_bytes > 0 || i.total_tx_bytes > 0,
	);

	if (activeInterfaces.length === 0) {
		return (
			<div className="p-4 text-text-secondary text-xs">
				No active network interfaces.
			</div>
		);
	}

	return (
		<div className="p-4 space-y-4">
			{activeInterfaces.map((iface) => (
				<div
					key={iface.name}
					className="bg-panel-card rounded-lg p-3 space-y-2"
				>
					<h4 className="text-[10px] font-bold text-text-secondary uppercase tracking-wider">
						{iface.name}
					</h4>

					{/* Bandwidth */}
					<div className="grid grid-cols-2 gap-2 text-xs">
						<div>
							<span className="text-text-secondary">↓ RX</span>
							<p className="font-mono font-bold text-status-success">
								{formatBytesPerSec(iface.rx_bytes_per_sec)}
							</p>
						</div>
						<div>
							<span className="text-text-secondary">↑ TX</span>
							<p className="font-mono font-bold text-accent">
								{formatBytesPerSec(iface.tx_bytes_per_sec)}
							</p>
						</div>
					</div>

					{/* Total transferred */}
					<div className="grid grid-cols-2 gap-2 text-[10px] text-text-secondary">
						<span>Total RX: {formatBytes(iface.total_rx_bytes)}</span>
						<span>Total TX: {formatBytes(iface.total_tx_bytes)}</span>
					</div>

					{/* Sparkline: combined rx + tx */}
					<SparklineChart
						data={history}
						accessor={(s) => {
							const net = s.network.interfaces.find(
								(n) => n.name === iface.name,
							);
							return net ? net.rx_bytes_per_sec + net.tx_bytes_per_sec : 0;
						}}
						max={undefined}
						height={48}
						color="#22c55e"
					/>
				</div>
			))}
		</div>
	);
}
