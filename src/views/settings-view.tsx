import { invoke } from "@tauri-apps/api/core";
import { disable, enable, isEnabled } from "@tauri-apps/plugin-autostart";
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
	const [autoLaunch, setAutoLaunch] = useState(false);
	const [pollingInterval, setPollingInterval] = useState(2000);

	useEffect(() => {
		loadDbInfo();
		loadAutoLaunch();
		loadPollingInterval();
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

	async function loadAutoLaunch() {
		try {
			setAutoLaunch(await isEnabled());
		} catch (e) {
			console.error("[pulse-orbit] Failed to check auto-launch:", e);
		}
	}

	async function toggleAutoLaunch() {
		try {
			if (autoLaunch) {
				await disable();
			} else {
				await enable();
			}
			setAutoLaunch(!autoLaunch);
		} catch (e) {
			console.error("[pulse-orbit] Failed to toggle auto-launch:", e);
		}
	}

	async function loadPollingInterval() {
		try {
			const value = await invoke<string>("get_setting", {
				key: "polling_interval_ms",
			});
			setPollingInterval(Number(value) || 2000);
		} catch {
			// Default 2000ms if not set
		}
	}

	async function updatePollingInterval(ms: number) {
		setPollingInterval(ms);
		try {
			await invoke("set_setting", {
				key: "polling_interval_ms",
				value: String(ms),
			});
		} catch (e) {
			console.error("[pulse-orbit] Failed to save polling interval:", e);
		}
	}

	return (
		<div className="p-4 space-y-6">
			<ThresholdEditor />

			{/* General Settings */}
			<div className="space-y-2">
				<h3 className="text-xs font-bold uppercase tracking-wider text-text-secondary">
					General
				</h3>
				<div className="bg-panel-card rounded-lg p-3 space-y-3">
					{/* Auto-launch */}
					<div className="flex items-center justify-between">
						<div>
							<p className="text-xs font-bold">Auto-launch at login</p>
							<p className="text-[10px] text-text-secondary">
								Start Pulse Orbit when you log in
							</p>
						</div>
						<button
							type="button"
							onClick={toggleAutoLaunch}
							className={`w-8 h-4 rounded-full transition-colors duration-150 ${
								autoLaunch ? "bg-accent" : "bg-panel-border"
							}`}
						>
							<div
								className={`w-3 h-3 rounded-full bg-white transition-transform duration-150 ${
									autoLaunch ? "translate-x-4" : "translate-x-0.5"
								}`}
							/>
						</button>
					</div>

					{/* Polling interval */}
					<div className="flex items-center justify-between">
						<div>
							<p className="text-xs font-bold">Polling interval</p>
							<p className="text-[10px] text-text-secondary">
								How often metrics are sampled
							</p>
						</div>
						<select
							value={pollingInterval}
							onChange={(e) => updatePollingInterval(Number(e.target.value))}
							className="bg-panel-bg border border-panel-border rounded px-2 py-1 text-xs font-mono text-text-primary"
						>
							<option value={1000}>1s</option>
							<option value={2000}>2s</option>
							<option value={5000}>5s</option>
							<option value={10000}>10s</option>
						</select>
					</div>
				</div>
			</div>

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
					<p className="text-text-secondary">v2.0.0 — macOS System Monitor</p>
					<p className="text-text-secondary mt-1">
						Retention: 24h • Aggregation: 5-min windows
					</p>
				</div>
			</div>
		</div>
	);
}
