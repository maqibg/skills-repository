import { create } from 'zustand'
import {
  deleteTemplate as deleteTemplateCommand,
  listTemplates as listTemplatesCommand,
  saveTemplate as saveTemplateCommand,
} from '../lib/tauri-client'
import type { SaveTemplateRequest, TemplateRecord } from '../types/app'

interface TemplatesStoreState {
  templates: TemplateRecord[]
  selectedTemplateId: string | null
  loading: boolean
  saving: boolean
  deleting: boolean
  loaded: boolean
  error: string | null
  refresh: () => Promise<void>
  selectTemplate: (templateId: string | null) => void
  saveTemplate: (request: SaveTemplateRequest) => Promise<TemplateRecord>
  deleteTemplate: (templateId: string) => Promise<void>
}

const toErrorMessage = (error: unknown) =>
  error instanceof Error ? error.message : String(error)

export const useTemplatesStore = create<TemplatesStoreState>((set, get) => ({
  templates: [],
  selectedTemplateId: null,
  loading: false,
  saving: false,
  deleting: false,
  loaded: false,
  error: null,
  refresh: async () => {
    set({ loading: true, error: null })
    try {
      const templates = await listTemplatesCommand()
      const selectedTemplateId = get().selectedTemplateId
      const nextSelectedId =
        selectedTemplateId && templates.some((item) => item.id === selectedTemplateId)
          ? selectedTemplateId
          : null

      set({
        templates,
        selectedTemplateId: nextSelectedId,
        loading: false,
        loaded: true,
      })
    } catch (error) {
      set({
        loading: false,
        loaded: true,
        error: toErrorMessage(error),
      })
    }
  },
  selectTemplate: (templateId) => {
    set({ selectedTemplateId: templateId, error: null })
  },
  saveTemplate: async (request) => {
    set({ saving: true, error: null })
    try {
      const template = await saveTemplateCommand(request)
      const templates = await listTemplatesCommand()
      set({
        templates,
        selectedTemplateId: template.id,
        saving: false,
      })
      return template
    } catch (error) {
      set({ saving: false, error: toErrorMessage(error) })
      throw error
    }
  },
  deleteTemplate: async (templateId) => {
    set({ deleting: true, error: null })
    try {
      await deleteTemplateCommand(templateId)
      const templates = await listTemplatesCommand()
      set({
        templates,
        selectedTemplateId: null,
        deleting: false,
      })
    } catch (error) {
      set({ deleting: false, error: toErrorMessage(error) })
      throw error
    }
  },
}))
