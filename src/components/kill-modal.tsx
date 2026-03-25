import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import { formatBytes } from "../lib/format";
import type { ProcessInfo } from "../types";

interface KillModalProps {
	process: ProcessInfo;
	onClose: () => void;
}

export function KillModal({ process, onClose }: KillModalProps) {
	const [error, setError] = useState<string | null>(null);
	const [killing, setKilling] = useState(false);

	async function handleConfirm() {
		setKilling(true);
		setError(null);
		try {
			await invoke("kill_process", { pid: process.pid });
			onClose();
		} catch (e) {
			setError(String(e));
			setKilling(false);
		}
	}

	return (
		<div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
			<div className="bg-panel-card border border-panel-border rounded-lg p-5 w-[320px] shadow-xl">
				<h3 className="text-sm font-bold mb-3">Kill Process</h3>
				<div className="space-y-1 mb-4 text-xs">
					<p>
						<span className="text-text-secondary">Name:</span>{" "}
						<span className="font-mono">{process.name}</span>
					</p>
					<p>
						<span className="text-text-secondary">PID:</span>{" "}
						<span className="font-mono">{process.pid}</span>
					</p>
					<p>
						<span className="text-text-secondary">Memory:</span>{" "}
						<span className="font-mono">
							{formatBytes(process.memory_bytes)}
						</span>
					</p>
				</div>
				<p className="text-xs text-text-secondary mb-4">
					This will send SIGTERM to the process. Unsaved work may be lost.
				</p>
				{error && (
					<p className="text-xs text-status-critical mb-3 bg-status-critical/10 rounded px-2 py-1">
						{error}
					</p>
				)}
				<div className="flex gap-2">
					<button
						type="button"
						onClick={onClose}
						className="flex-1 px-3 py-2 text-xs rounded bg-panel-border text-text-primary hover:bg-panel-border/80 transition-colors duration-150"
					>
						Cancel
					</button>
					<button
						type="button"
						onClick={handleConfirm}
						disabled={killing}
						className="flex-1 px-3 py-2 text-xs rounded bg-status-critical text-white font-bold hover:bg-status-critical/80 transition-colors duration-150 disabled:opacity-50"
					>
						{killing ? "Killing..." : "Confirm Kill"}
					</button>
				</div>
			</div>
		</div>
	);
}
