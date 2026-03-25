import { useState } from "react";
import { PanelTabs, type TabId } from "./components/panel-tabs";
import { useMetrics } from "./hooks/use-metrics";
import { CpuView } from "./views/cpu-view";
import { MemoryView } from "./views/memory-view";
import { ProcessView } from "./views/process-view";

function App() {
	const { current, history } = useMetrics();
	const [activeTab, setActiveTab] = useState<TabId>("cpu");

	return (
		<div className="flex flex-col h-screen bg-panel-bg">
			<PanelTabs activeTab={activeTab} onTabChange={setActiveTab} />
			<div className="flex-1 overflow-y-auto">
				<div className={activeTab === "cpu" ? "block" : "hidden"}>
					<CpuView current={current} history={history} />
				</div>
				<div className={activeTab === "memory" ? "block" : "hidden"}>
					<MemoryView current={current} history={history} />
				</div>
				<div className={activeTab === "process" ? "block" : "hidden"}>
					<ProcessView current={current} />
				</div>
			</div>
		</div>
	);
}

export default App;
