import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";
import type { MetricSnapshot } from "../types";

const BUFFER_SIZE = 60; // 60 samples × 2s = 120s window

export interface MetricsState {
	current: MetricSnapshot | null;
	history: MetricSnapshot[];
}

/**
 * Subscribe to metric-snapshot events from the Rust backend.
 * Returns the latest snapshot and a rolling 60-entry history buffer.
 */
export function useMetrics(): MetricsState {
	const [state, setState] = useState<MetricsState>({
		current: null,
		history: [],
	});
	const bufferRef = useRef<MetricSnapshot[]>([]);

	useEffect(() => {
		const unlisten = listen<MetricSnapshot>("metric-snapshot", (event) => {
			const buf = bufferRef.current;
			buf.push(event.payload);
			if (buf.length > BUFFER_SIZE) {
				buf.shift();
			}
			setState({ current: event.payload, history: [...buf] });
		});

		return () => {
			unlisten.then((fn) => fn());
		};
	}, []);

	return state;
}
