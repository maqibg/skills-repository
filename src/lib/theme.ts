import type { ResolvedTheme, ThemeMode } from '../types/app'

export const resolveThemeMode = (
  themeMode: ThemeMode,
  systemTheme: 'light' | 'dark',
): ResolvedTheme => {
  if (themeMode === 'system') {
    return systemTheme === 'dark' ? 'skills-dark' : 'skills-light'
  }

  return themeMode === 'dark' ? 'skills-dark' : 'skills-light'
}

export const applyResolvedTheme = (theme: ResolvedTheme) => {
  document.documentElement.setAttribute('data-theme', theme)
}
