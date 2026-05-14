import type { AppConfig, AppColorShortcutsConfig, AppThemeConfig, AppToolShortcutsConfig } from "./types";

let activeAppConfig: AppConfig | null = null;

export function setActiveAppConfig(config: AppConfig) {
  activeAppConfig = config;
  applyThemeConfig(config.theme);
}

export function getActiveAppConfig() {
  return activeAppConfig;
}

export function applyThemeConfig(theme: AppThemeConfig) {
  const rootStyle = document.documentElement.style;
  const themeVariables: Record<string, string> = {
    "--bg": theme.bg,
    "--bg-dark": theme.bgDark,
    "--bg-darker": theme.bgDarker,
    "--bg-highlight": theme.bgHighlight,
    "--bg-panel": theme.bgPanel,
    "--fg": theme.fg,
    "--fg-dark": theme.fgDark,
    "--fg-gutter": theme.fgGutter,
    "--blue": theme.blue,
    "--cyan": theme.cyan,
    "--green": theme.green,
    "--yellow": theme.yellow,
    "--orange": theme.orange,
    "--red": theme.red,
    "--magenta": theme.magenta,
    "--purple": theme.purple,
  };

  for (const [variableName, value] of Object.entries(themeVariables)) {
    rootStyle.setProperty(variableName, value);
  }
}

export function normalizedToolShortcutEntries(shortcuts: AppToolShortcutsConfig) {
  return Object.entries(shortcuts)
    .map(([tool, key]) => [normalizeShortcutKey(key), tool] as const)
    .filter((entry): entry is readonly [string, string] => Boolean(entry[0]));
}

export function normalizedColorShortcutEntries(shortcuts: AppColorShortcutsConfig) {
  return Object.entries(shortcuts)
    .map(([color, key]) => [normalizeShortcutKey(key), color] as const)
    .filter((entry): entry is readonly [string, string] => Boolean(entry[0]));
}

function normalizeShortcutKey(value: string) {
  const trimmed = value.trim().toLowerCase();
  return trimmed.length > 0 ? trimmed : "";
}
