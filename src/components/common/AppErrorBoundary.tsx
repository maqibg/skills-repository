import { Component, type ErrorInfo, type ReactNode } from 'react'
import { useTranslation } from 'react-i18next'

interface AppErrorBoundaryProps {
  children: ReactNode
}

interface AppErrorBoundaryContentProps extends AppErrorBoundaryProps {
  title: string
  description: string
}

interface AppErrorBoundaryState {
  hasError: boolean
  message: string
}

class AppErrorBoundaryContent extends Component<
  AppErrorBoundaryContentProps,
  AppErrorBoundaryState
> {
  state: AppErrorBoundaryState = {
    hasError: false,
    message: '',
  }

  static getDerivedStateFromError(error: Error): AppErrorBoundaryState {
    return {
      hasError: true,
      message: error.message,
    }
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error('[ui] AppErrorBoundary caught an error', error, errorInfo)
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="flex min-h-screen items-center justify-center bg-base-200 px-6 text-base-content">
          <div className="max-w-2xl rounded-box border border-error/40 bg-base-100 p-8 shadow-sm">
            <p className="text-lg font-semibold text-error">{this.props.title}</p>
            <p className="mt-2 text-sm text-base-content/70">{this.state.message}</p>
            <p className="mt-4 text-xs text-base-content/50">{this.props.description}</p>
          </div>
        </div>
      )
    }

    return this.props.children
  }
}

export function AppErrorBoundary({ children }: AppErrorBoundaryProps) {
  const { t } = useTranslation()

  return (
    <AppErrorBoundaryContent
      title={t('errors.uiRenderTitle')}
      description={t('errors.uiRenderDescription')}
    >
      {children}
    </AppErrorBoundaryContent>
  )
}
