import { Component, type ErrorInfo, type ReactNode } from 'react'

interface AppErrorBoundaryProps {
  children: ReactNode
}

interface AppErrorBoundaryState {
  hasError: boolean
  message: string
}

export class AppErrorBoundary extends Component<
  AppErrorBoundaryProps,
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
            <p className="text-lg font-semibold text-error">主界面渲染失败</p>
            <p className="mt-2 text-sm text-base-content/70">{this.state.message}</p>
            <p className="mt-4 text-xs text-base-content/50">
              这说明启动数据已经返回，但主界面渲染过程中出现异常。请把终端和页面错误信息发给我继续排查。
            </p>
          </div>
        </div>
      )
    }

    return this.props.children
  }
}
