import { create } from 'zustand'
import i18n from '../lib/i18n'
import { saveSettings as saveSettingsCommand } from '../lib/tauri-client'
import { applyResolvedTheme, resolveThemeMode } from '../lib/theme'
import type { AppLocale, AppSettings, ThemeMode } from '../types/app'
import { useAppStore } from './use-app-store'

interface SettingsStoreState {
  settings: AppSettings
  saving: boolean
  setSettings: (settings: AppSettings) => void
  setLanguage: (language: AppLocale) => void
  setThemeMode: (themeMode: ThemeMode) => void
  setRootsText: (kind: 'projectRoots' | 'customRoots', value: string) => void
  save: () => Promise<void>
}

const defaultSettings: AppSettings = {
  language: 'en-US',
  themeMode: 'system',
  scan: {
    projectRoots: [],
    customRoots: [],
  },
  agentPreferences: {},
}

const parseLines = (value: string) =>
  value
    .split('\n')
    .map((item) => item.trim())
    .filter(Boolean)

export const useSettingsStore = create<SettingsStoreState>((set, get) => ({
  settings: defaultSettings,
  saving: false,
  setSettings: (settings) => set({ settings }),
  setLanguage: (language) =>
    set((state) => ({
      settings: { ...state.settings, language },
    })),
  setThemeMode: (themeMode) =>
    set((state) => ({
      settings: { ...state.settings, themeMode },
    })),
  setRootsText: (kind, value) =>
    set((state) => ({
      settings: {
        ...state.settings,
        scan: {
          ...state.settings.scan,
          [kind]: parseLines(value),
        },
      },
    })),
  save: async () => {
    set({ saving: true })
    try {
      const saved = await saveSettingsCommand(get().settings)
      set({ settings: saved, saving: false })

      const systemTheme = useAppStore.getState().system?.theme ?? 'light'
      applyResolvedTheme(resolveThemeMode(saved.themeMode, systemTheme))
      await i18n.changeLanguage(saved.language)
    } catch (error) {
      set({ saving: false })
      throw error
    }
  },
}))
