import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useRef, useState } from "react";
import type { AlertThreshold } from "../types";

interface ThresholdConfig {
	metric_type: string;
	label: string;
	unit: string;
	min: number;
	max: number;
	step: number;
	defaultValue: number;
	defaultCooldown: number;
}

const THRESHOLD_CONFIGS: ThresholdConfig[] = [
	{
		metric_type: "cpu",
		label: "CPU",
		unit: "%",
		min: 0,
		max: 100,
		step: 5,
		defaultValue: 90,
		defaultCooldown: 300,
	},
	{
		metric_type: "memory",
		label: "Memory",
		unit: "%",
		min: 0,
		max: 100,
		step: 5,
		defaultValue: 85,
		defaultCooldown: 300,
	},
	{
		metric_type: "disk_write",
		label: "Disk Write",
		unit: "MB/s",
		min: 0,
		max: 1000,
		step: 10,
		defaultValue: 500,
		defaultCooldown: 300,
	},
	{
		metric_type: "net_in",
		label: "Network In",
		unit: "MB/s",
		min: 0,
		max: 1000,
		step: 10,
		defaultValue: 100,
		defaultCooldown: 300,
	},
];

export function ThresholdEditor() {
	const [thresholds, setThresholds] = useState<Map<string, AlertThreshold>>(
		new Map(),
	);
	const debounceTimers = useRef<Map<string, ReturnType<typeof setTimeout>>>(
		new Map(),
	);

	useEffect(() => {
		loadThresholds();
	}, []);

	async function loadThresholds() {
		try {
			const saved = await invoke<AlertThreshold[]>("get_thresholds");
			const map = new Map<string, AlertThreshold>();
			for (const t of saved) {
				map.set(t.metric_type, t);
			}
			setThresholds(map);
		} catch (e) {
			console.error("[pulse-orbit] Failed to load thresholds:", e);
		}
	}

	const saveThreshold = useCallback((threshold: AlertThreshold) => {
		// Debounce saves
		const existing = debounceTimers.current.get(threshold.metric_type);
		if (existing) clearTimeout(existing);

		debounceTimers.current.set(
			threshold.metric_type,
			setTimeout(async () => {
				try {
					await invoke("set_threshold", { threshold });
				} catch (e) {
					console.error("[pulse-orbit] Failed to save threshold:", e);
				}
			}, 500),
		);
	}, []);

	function getThreshold(config: ThresholdConfig): AlertThreshold {
		return (
			thresholds.get(config.metric_type) ?? {
				metric_type: config.metric_type,
				threshold: config.defaultValue,
				enabled: false,
				cooldown_seconds: config.defaultCooldown,
			}
		);
	}

	function updateThreshold(
		config: ThresholdConfig,
		partial: Partial<AlertThreshold>,
	) {
		const current = getThreshold(config);
		const updated = { ...current, ...partial };
		const newMap = new Map(thresholds);
		newMap.set(config.metric_type, updated);
		setThresholds(newMap);
		saveThreshold(updated);
	}

	return (
		<div className="space-y-3">
			<h3 className="text-xs font-bold uppercase tracking-wider text-text-secondary">
				Alert Thresholds
			</h3>
			{THRESHOLD_CONFIGS.map((config) => {
				const t = getThreshold(config);
				return (
					<div
						key={config.metric_type}
						className="bg-panel-card rounded-lg p-3 space-y-2"
					>
						<div className="flex items-center justify-between">
							<span className="text-xs font-bold">{config.label}</span>
							<button
								type="button"
								onClick={() => updateThreshold(config, { enabled: !t.enabled })}
								className={`w-8 h-4 rounded-full transition-colors duration-150 ${
									t.enabled ? "bg-accent" : "bg-panel-border"
								}`}
							>
								<div
									className={`w-3 h-3 rounded-full bg-white transition-transform duration-150 ${
										t.enabled ? "translate-x-4" : "translate-x-0.5"
									}`}
								/>
							</button>
						</div>
						<div className="flex items-center gap-2">
							<input
								type="range"
								min={config.min}
								max={config.max}
								step={config.step}
								value={t.threshold}
								onChange={(e) =>
									updateThreshold(config, { threshold: Number(e.target.value) })
								}
								className="flex-1 h-1 accent-accent"
								disabled={!t.enabled}
							/>
							<span className="font-mono text-xs w-16 text-right">
								{t.threshold} {config.unit}
							</span>
						</div>
						<div className="flex items-center gap-2 text-[10px] text-text-secondary">
							<span>Cooldown:</span>
							<input
								type="number"
								min={60}
								max={3600}
								step={60}
								value={t.cooldown_seconds}
								onChange={(e) =>
									updateThreshold(config, {
										cooldown_seconds: Number(e.target.value),
									})
								}
								className="w-16 bg-panel-bg border border-panel-border rounded px-1 py-0.5 font-mono text-text-primary"
								disabled={!t.enabled}
							/>
							<span>seconds</span>
						</div>
					</div>
				);
			})}
		</div>
	);
}
