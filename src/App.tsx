import { useEffect, useRef } from 'react'
import { RouterProvider } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { router } from './app/router'
import { AppErrorBoundary } from './components/common/AppErrorBoundary'
import { bootstrapApp } from './lib/tauri-client'
import { applyResolvedTheme, resolveThemeMode } from './lib/theme'
import { useAppStore } from './stores/use-app-store'
import { useSettingsStore } from './stores/use-settings-store'

function App() {
  const { i18n } = useTranslation()
  const startedRef = useRef(false)
  const bootstrapped = useAppStore((state) => state.bootstrapped)
  const setBootstrapPayload = useAppStore((state) => state.setBootstrapPayload)
  const setSettings = useSettingsStore((state) => state.setSettings)
  const settings = useSettingsStore((state) => state.settings)
  const system = useAppStore((state) => state.system)

  useEffect(() => {
    if (startedRef.current) return
    startedRef.current = true

    let mounted = true

    void bootstrapApp()
      .then((payload) => {
        if (!mounted) return
        setBootstrapPayload(payload)
        setSettings(payload.settings)
      })
      .catch((cause) => {
        if (!mounted) return
        console.error('[startup] bootstrap_app failed', cause)
      })

    return () => {
      mounted = false
    }
  }, [setBootstrapPayload, setSettings])

  useEffect(() => {
    if (!bootstrapped || !system) return

    const resolvedTheme = resolveThemeMode(settings.themeMode, system.theme)
    applyResolvedTheme(resolvedTheme)

    void i18n.changeLanguage(settings.language).catch((cause) => {
      console.error('Failed to change language:', cause)
    })
  }, [bootstrapped, i18n, settings.language, settings.themeMode, system])

  if (!bootstrapped) return null

  return (
    <AppErrorBoundary>
      <RouterProvider router={router} />
    </AppErrorBoundary>
  )
}

export default App
