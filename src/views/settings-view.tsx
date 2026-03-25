import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { ThresholdEditor } from "../components/threshold-editor";
import { formatBytes } from "../lib/format";

interface DbInfo {
	row_count: number;
	file_size_bytes: number;
	path: string;
}

export function SettingsView() {
	const [dbInfo, setDbInfo] = useState<DbInfo | null>(null);

	useEffect(() => {
		loadDbInfo();
		const interval = setInterval(loadDbInfo, 30_000);
		return () => clearInterval(interval);
	}, []);

	async function loadDbInfo() {
		try {
			const info = await invoke<DbInfo>("get_db_info");
			setDbInfo(info);
		} catch (e) {
			console.error("[pulse-orbit] Failed to load DB info:", e);
		}
	}

	return (
		<div className="p-4 space-y-6">
			<ThresholdEditor />

			{/* DB Info */}
			<div className="space-y-2">
				<h3 className="text-xs font-bold uppercase tracking-wider text-text-secondary">
					Data Storage
				</h3>
				<div className="bg-panel-card rounded-lg p-3">
					{dbInfo ? (
						<div className="grid grid-cols-2 gap-2 text-xs">
							<div>
								<span className="text-text-secondary">DB Size</span>
								<p className="font-mono font-bold">
									{formatBytes(dbInfo.file_size_bytes)}
								</p>
							</div>
							<div>
								<span className="text-text-secondary">Metric Rows</span>
								<p className="font-mono font-bold">
									{dbInfo.row_count.toLocaleString()}
								</p>
							</div>
							<div className="col-span-2">
								<span className="text-text-secondary">Path</span>
								<p
									className="font-mono text-[10px] truncate"
									title={dbInfo.path}
								>
									{dbInfo.path}
								</p>
							</div>
						</div>
					) : (
						<p className="text-xs text-text-secondary">Loading...</p>
					)}
				</div>
			</div>

			{/* About */}
			<div className="space-y-2">
				<h3 className="text-xs font-bold uppercase tracking-wider text-text-secondary">
					About
				</h3>
				<div className="bg-panel-card rounded-lg p-3 text-xs">
					<p className="font-bold">Pulse Orbit</p>
					<p className="text-text-secondary">v0.1.0 — macOS System Monitor</p>
					<p className="text-text-secondary mt-1">
						Retention: 24h • Aggregation: 5-min windows
					</p>
				</div>
			</div>
		</div>
	);
}
