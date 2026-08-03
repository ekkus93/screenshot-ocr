/** @type {import('tailwindcss').Config} */
export default {
  darkMode: "class",
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        // Warm-neutral "ink" for dark surfaces — deliberately not pure black.
        ink: {
          950: "#14181a",
          900: "#1b2023",
          800: "#262c2f",
          700: "#333b3e",
        },
        // Cool-neutral "paper" for light surfaces — deliberately not warm cream.
        paper: {
          50: "#f1f3ef",
          100: "#e7eae4",
        },
        // Muted blue-grey used for borders and secondary text in both themes.
        steel: {
          200: "#d7dcd6",
          400: "#8b969c",
          500: "#6b7680",
          600: "#4b5560",
          800: "#2a3034",
        },
        // Primary accent — "phosphor" amber, the scan/recognition color.
        // 700 is a darkened, text-safe variant for use on light backgrounds.
        phosphor: {
          500: "#f2a93b",
          600: "#dc9526",
          700: "#8a5c17",
        },
        // Secondary accent — "signal" teal, used for confirmed/success states.
        signal: {
          500: "#3fbbae",
          600: "#2e9c90",
          700: "#1f6b63",
        },
        // Error accent — warm coral, kept visually distinct from phosphor.
        alert: {
          500: "#e2574c",
          600: "#c8402f",
          700: "#9c2b22",
        },
      },
      fontFamily: {
        sans: [
          '"IBM Plex Sans"',
          "Inter",
          "ui-sans-serif",
          "system-ui",
          "-apple-system",
          "sans-serif",
        ],
        mono: [
          '"JetBrains Mono"',
          '"IBM Plex Mono"',
          '"Cascadia Code"',
          "ui-monospace",
          "SFMono-Regular",
          "Menlo",
          "monospace",
        ],
      },
      keyframes: {
        "scan-sweep": {
          "0%": { top: "-6%", opacity: "0" },
          "10%": { opacity: "1" },
          "90%": { opacity: "1" },
          "100%": { top: "104%", opacity: "0" },
        },
      },
      animation: {
        "scan-sweep": "scan-sweep 1.8s ease-in-out infinite",
      },
    },
  },
  plugins: [],
};
