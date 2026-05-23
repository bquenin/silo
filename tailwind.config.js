/** @type {import('tailwindcss').Config} */
export default {
  darkMode: 'class',
  content: ['./index.html', './src/**/*.{js,jsx,ts,tsx}'],
  theme: {
    extend: {
      colors: {
        bg: {
          DEFAULT: '#0a0a0c',
          subtle: '#111114',
          surface: '#16161a',
          elevated: '#1c1c22',
          border: '#26262e',
        },
        fg: {
          DEFAULT: '#e8e8ea',
          muted: '#8b8b91',
          dim: '#56565c',
        },
        accent: {
          gdi: '#e6c34a',
          nod: '#d44848',
          scrin: '#4ad4cf',
          DEFAULT: '#7be03e',
          dim: '#4f9c25',
        },
      },
      fontFamily: {
        sans: ['Inter', 'system-ui', '-apple-system', 'Segoe UI', 'sans-serif'],
        mono: ['JetBrains Mono', 'ui-monospace', 'monospace'],
      },
    },
  },
  plugins: [],
};
