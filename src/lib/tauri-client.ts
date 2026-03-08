import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type {
  AppSettings,
  BootstrapPayload,
  ScanSkillsRequest,
  TaskHandle,
  TaskProgress,
} from '../types/app'

export const EVENT_TASK_PROGRESS = 'task:progress'
export const EVENT_TASK_COMPLETED = 'task:completed'
export const EVENT_TASK_FAILED = 'task:failed'

export const bootstrapApp = () => invoke<BootstrapPayload>('bootstrap_app')

export const getSettings = () => invoke<AppSettings>('get_settings')

export const saveSettings = (settings: AppSettings) =>
  invoke<AppSettings>('save_settings', { settings })

export const scanSkills = (request: ScanSkillsRequest) =>
  invoke<TaskHandle>('scan_skills', { request })

export const onTaskProgress = (
  handler: (event: TaskProgress) => void,
) => listen<TaskProgress>(EVENT_TASK_PROGRESS, ({ payload }) => handler(payload))

export const onTaskCompleted = (
  handler: (event: TaskProgress) => void,
) => listen<TaskProgress>(EVENT_TASK_COMPLETED, ({ payload }) => handler(payload))

export const onTaskFailed = (handler: (event: TaskProgress) => void) =>
  listen<TaskProgress>(EVENT_TASK_FAILED, ({ payload }) => handler(payload))
