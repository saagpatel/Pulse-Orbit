import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";
import type { HistoryRow } from "../types";

const REFETCH_INTERVAL = 5 * 60 * 1000; // 5 minutes

interface UseHistoryResult {
	data: HistoryRow[];
	loading: boolean;
}

/**
 * Fetch historical metric data from SQLite via Tauri command.
 * Re-fetches every 5 minutes to pick up new aggregates.
 */
export function useHistory(
	metricType: string,
	iface?: string,
	hours = 24,
): UseHistoryResult {
	const [data, setData] = useState<HistoryRow[]>([]);
	const [loading, setLoading] = useState(true);

	const fetchHistory = useCallback(async () => {
		try {
			const rows = await invoke<HistoryRow[]>("get_history", {
				metric_type: metricType,
				interface: iface ?? null,
				hours,
			});
			setData(rows);
		} catch (e) {
			console.error("[pulse-orbit] Failed to fetch history:", e);
		} finally {
			setLoading(false);
		}
	}, [metricType, iface, hours]);

	useEffect(() => {
		fetchHistory();
		const interval = setInterval(fetchHistory, REFETCH_INTERVAL);
		return () => clearInterval(interval);
	}, [fetchHistory]);

	return { data, loading };
}
