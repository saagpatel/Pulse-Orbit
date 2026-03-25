export function formatBytes(bytes: number): string {
	if (bytes >= 1_073_741_824) return `${(bytes / 1_073_741_824).toFixed(1)} GB`;
	if (bytes >= 1_048_576) return `${(bytes / 1_048_576).toFixed(1)} MB`;
	return `${(bytes / 1024).toFixed(1)} KB`;
}

export function formatPercent(value: number): string {
	return `${value.toFixed(1)}%`;
}
