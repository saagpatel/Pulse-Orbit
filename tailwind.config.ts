import type { Config } from "tailwindcss";

export default {
	content: ["./index.html", "./src/**/*.{ts,tsx}"],
	theme: {
		extend: {
			colors: {
				panel: {
					bg: "#0f1117",
					card: "#1a1d27",
					border: "#2d3148",
				},
				accent: {
					DEFAULT: "#6366f1",
					muted: "rgba(99, 102, 241, 0.2)",
				},
				status: {
					success: "#22c55e",
					warning: "#f59e0b",
					critical: "#ef4444",
				},
				text: {
					primary: "#e2e8f0",
					secondary: "#64748b",
				},
			},
			fontFamily: {
				display: ["'Space Grotesk'", "system-ui", "sans-serif"],
				mono: ["'JetBrains Mono'", "ui-monospace", "monospace"],
			},
		},
	},
	plugins: [],
} satisfies Config;
