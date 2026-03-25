interface MetricBarProps {
	label: string;
	value: number;
	max: number;
	format: (v: number) => string;
	color?: string;
	compact?: boolean;
}

export function MetricBar({
	label,
	value,
	max,
	format,
	color = "bg-accent",
	compact = false,
}: MetricBarProps) {
	const percent = max > 0 ? Math.min((value / max) * 100, 100) : 0;

	if (compact) {
		return (
			<div className="flex items-center gap-1.5">
				<div className="h-1.5 flex-1 rounded-full bg-panel-border overflow-hidden">
					<div
						className={`h-full rounded-full transition-all duration-200 ${color}`}
						style={{ width: `${percent}%` }}
					/>
				</div>
				<span className="font-mono text-[10px] text-text-secondary w-8 text-right shrink-0">
					{format(value)}
				</span>
			</div>
		);
	}

	return (
		<div className="flex items-center gap-2">
			<span className="w-2/5 text-text-secondary text-xs truncate">
				{label}
			</span>
			<div className="w-1/2 h-2 rounded-full bg-panel-border overflow-hidden">
				<div
					className={`h-full rounded-full transition-all duration-200 ${color}`}
					style={{ width: `${percent}%` }}
				/>
			</div>
			<span className="w-[10%] font-mono text-xs text-right shrink-0">
				{format(value)}
			</span>
		</div>
	);
}
