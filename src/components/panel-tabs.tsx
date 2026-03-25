export type TabId =
	| "cpu"
	| "memory"
	| "disk"
	| "network"
	| "process"
	| "settings";

interface PanelTabsProps {
	activeTab: TabId;
	onTabChange: (tab: TabId) => void;
}

const tabs: { id: TabId; label: string; icon: string }[] = [
	{ id: "cpu", label: "CPU", icon: "⚡" },
	{ id: "memory", label: "Memory", icon: "◻" },
	{ id: "disk", label: "Disk", icon: "◉" },
	{ id: "network", label: "Network", icon: "⇅" },
	{ id: "process", label: "Process", icon: "⊞" },
	{ id: "settings", label: "Settings", icon: "⚙" },
];

export function PanelTabs({ activeTab, onTabChange }: PanelTabsProps) {
	return (
		<div className="flex border-b border-panel-border bg-panel-card/50">
			{tabs.map((tab) => (
				<button
					key={tab.id}
					type="button"
					onClick={() => onTabChange(tab.id)}
					className={`flex-1 flex items-center justify-center gap-1 px-1.5 py-2.5 text-[10px] font-bold transition-colors duration-150 ${
						activeTab === tab.id
							? "text-accent border-b-2 border-accent"
							: "text-text-secondary hover:text-text-primary"
					}`}
				>
					<span>{tab.icon}</span>
					<span>{tab.label}</span>
				</button>
			))}
		</div>
	);
}
