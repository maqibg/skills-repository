import { create } from 'zustand'
import {
  updateGithubRepositorySkills as updateGithubRepositorySkillsCommand,
  updateRepositorySkill as updateRepositorySkillCommand,
  batchDistributeRepositorySkills as batchDistributeRepositorySkillsCommand,
  getRepositorySkillDeletionPreview as getRepositorySkillDeletionPreviewCommand,
  getRepositorySkillDetail as getRepositorySkillDetailCommand,
  importRepositorySkill as importRepositorySkillCommand,
  listRepositorySkills as listRepositorySkillsCommand,
  resolveRepositoryImportSource as resolveRepositoryImportSourceCommand,
  uninstallRepositorySkill as uninstallRepositorySkillCommand,
} from '../lib/tauri-client'
import type {
  BatchRepositorySkillUpdateResult,
  BatchDistributeRepositorySkillsRequest,
  BatchDistributeResult,
  InstallSkillResult,
  ImportRepositorySkillRequest,
  RepositorySkillDetail,
  RepositorySkillDeletionPreview,
  RepositorySkillSummary,
  RepositorySkillUpdateItemResult,
  ResolveRepositoryImportRequest,
  ResolveRepositoryImportResult,
  SecurityReport,
} from '../types/app'

const toBatchUpdateResult = (
  result: RepositorySkillUpdateItemResult,
): BatchRepositorySkillUpdateResult => {
  if (result.status === 'updated') {
    return { updated: [result], skipped: [], failed: [] }
  }
  if (result.status === 'skipped') {
    return { updated: [], skipped: [result], failed: [] }
  }
  return { updated: [], skipped: [], failed: [result] }
}

interface RepositoryStoreState {
  items: RepositorySkillSummary[]
  loading: boolean
  loaded: boolean
  error: string | null
  selectedDetail: RepositorySkillDetail | null
  detailLoading: boolean
  detailError: string | null
  uninstallingSkillId: string | null
  deletePreview: RepositorySkillDeletionPreview | null
  deletePreviewLoading: boolean
  deletePreviewError: string | null
  distributionOpen: boolean
  distributing: boolean
  distributionError: string | null
  lastDistributionResult: BatchDistributeResult | null
  updatingSkillIds: string[]
  bulkUpdating: boolean
  updateError: string | null
  lastUpdateResult: BatchRepositorySkillUpdateResult | null
  resolvingImport: boolean
  importing: boolean
  importError: string | null
  importBlockedReport: SecurityReport | null
  resolvedImport: ResolveRepositoryImportResult | null
  refresh: () => Promise<void>
  loadDetail: (skillId: string) => Promise<void>
  closeDetail: () => void
  uninstall: (skillId: string) => Promise<void>
  loadDeletePreview: (skillId: string) => Promise<void>
  clearDeletePreview: () => void
  openDistribution: () => void
  closeDistribution: () => void
  batchDistributeSkills: (request: BatchDistributeRepositorySkillsRequest) => Promise<BatchDistributeResult>
  updateSkill: (skillId: string) => Promise<RepositorySkillUpdateItemResult>
  updateGithubSkills: () => Promise<BatchRepositorySkillUpdateResult>
  clearUpdateState: () => void
  resolveImport: (request: ResolveRepositoryImportRequest) => Promise<ResolveRepositoryImportResult>
  importSkill: (request: ImportRepositorySkillRequest) => Promise<InstallSkillResult>
  resetImportState: () => void
  resetDistributionState: () => void
}

export const useRepositoryStore = create<RepositoryStoreState>((set, get) => ({
  items: [],
  loading: false,
  loaded: false,
  error: null,
  selectedDetail: null,
  detailLoading: false,
  detailError: null,
  uninstallingSkillId: null,
  deletePreview: null,
  deletePreviewLoading: false,
  deletePreviewError: null,
  distributionOpen: false,
  distributing: false,
  distributionError: null,
  lastDistributionResult: null,
  updatingSkillIds: [],
  bulkUpdating: false,
  updateError: null,
  lastUpdateResult: null,
  resolvingImport: false,
  importing: false,
  importError: null,
  importBlockedReport: null,
  resolvedImport: null,
  refresh: async () => {
    set({ loading: true, error: null })
    try {
      const items = await listRepositorySkillsCommand()
      set({ items, loading: false, loaded: true, error: null })
    } catch (error) {
      set({
        loading: false,
        loaded: true,
        error: error instanceof Error ? error.message : String(error),
      })
    }
  },
  loadDetail: async (skillId) => {
    set({ detailLoading: true, detailError: null, selectedDetail: null })
    try {
      const selectedDetail = await getRepositorySkillDetailCommand(skillId)
      set({ selectedDetail, detailLoading: false, detailError: null })
    } catch (error) {
      set({
        detailLoading: false,
        detailError: error instanceof Error ? error.message : String(error),
      })
    }
  },
  closeDetail: () => set({ selectedDetail: null, detailLoading: false, detailError: null }),
  loadDeletePreview: async (skillId) => {
    set({ deletePreviewLoading: true, deletePreviewError: null, deletePreview: null })
    try {
      const deletePreview = await getRepositorySkillDeletionPreviewCommand(skillId)
      set({ deletePreview, deletePreviewLoading: false, deletePreviewError: null })
    } catch (error) {
      set({
        deletePreviewLoading: false,
        deletePreviewError: error instanceof Error ? error.message : String(error),
      })
      throw error
    }
  },
  clearDeletePreview: () =>
    set({ deletePreview: null, deletePreviewLoading: false, deletePreviewError: null }),
  uninstall: async (skillId) => {
    set({ uninstallingSkillId: skillId })
    try {
      const result = await uninstallRepositorySkillCommand(skillId)
      set((state) => ({
        items: state.items.filter((item) => item.id !== result.skillId),
        selectedDetail: state.selectedDetail?.id === result.skillId ? null : state.selectedDetail,
        deletePreview: state.deletePreview?.skillId === result.skillId ? null : state.deletePreview,
        deletePreviewError: null,
      }))
    } catch (error) {
      set({
        deletePreviewError: error instanceof Error ? error.message : String(error),
      })
    } finally {
      set({ uninstallingSkillId: null })
    }
  },
  openDistribution: () =>
    set({
      distributionOpen: true,
      distributionError: null,
      lastDistributionResult: null,
    }),
  closeDistribution: () =>
    set({
      distributionOpen: false,
      distributionError: null,
      lastDistributionResult: null,
    }),
  batchDistributeSkills: async (request) => {
    set({ distributing: true, distributionError: null, lastDistributionResult: null })
    try {
      const result = await batchDistributeRepositorySkillsCommand(request)
      set({
        distributing: false,
        distributionError: null,
        lastDistributionResult: result,
      })
      return result
    } catch (error) {
      set({
        distributing: false,
        distributionError: error instanceof Error ? error.message : String(error),
      })
      throw error
    }
  },
  updateSkill: async (skillId) => {
    set((state) => ({
      updatingSkillIds: state.updatingSkillIds.includes(skillId)
        ? state.updatingSkillIds
        : [...state.updatingSkillIds, skillId],
      updateError: null,
      lastUpdateResult: null,
    }))

    try {
      const result = await updateRepositorySkillCommand(skillId)
      set({
        updatingSkillIds: get().updatingSkillIds.filter((item) => item !== skillId),
        updateError: null,
        lastUpdateResult: toBatchUpdateResult(result),
      })
      if (result.status === 'updated') {
        try {
          const items = await listRepositorySkillsCommand()
          set({ items })
        } catch (error) {
          set({
            updateError: error instanceof Error ? error.message : String(error),
          })
        }
      }
      return result
    } catch (error) {
      set((state) => ({
        updatingSkillIds: state.updatingSkillIds.filter((item) => item !== skillId),
        updateError: error instanceof Error ? error.message : String(error),
      }))
      throw error
    }
  },
  updateGithubSkills: async () => {
    set({ bulkUpdating: true, updateError: null, lastUpdateResult: null })
    try {
      const result = await updateGithubRepositorySkillsCommand()
      set({
        bulkUpdating: false,
        updateError: null,
        lastUpdateResult: result,
      })
      if (result.updated.length > 0) {
        try {
          const items = await listRepositorySkillsCommand()
          set({ items })
        } catch (error) {
          set({
            updateError: error instanceof Error ? error.message : String(error),
          })
        }
      }
      return result
    } catch (error) {
      set({
        bulkUpdating: false,
        updateError: error instanceof Error ? error.message : String(error),
      })
      throw error
    }
  },
  clearUpdateState: () =>
    set({
      updatingSkillIds: [],
      bulkUpdating: false,
      updateError: null,
      lastUpdateResult: null,
    }),
  resolveImport: async (request) => {
    set({ resolvingImport: true, importError: null, importBlockedReport: null, resolvedImport: null })
    try {
      const resolvedImport = await resolveRepositoryImportSourceCommand(request)
      set({ resolvingImport: false, resolvedImport, importError: null, importBlockedReport: null })
      return resolvedImport
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      set({ resolvingImport: false, importError: message })
      throw error
    }
  },
  importSkill: async (request) => {
    set({ importing: true, importError: null, importBlockedReport: null })
    try {
      const result = await importRepositorySkillCommand(request)
      const items = await listRepositorySkillsCommand()
      set({
        items,
        importing: false,
        importError: null,
        importBlockedReport: result.blocked ? result.securityReport ?? null : null,
        resolvedImport: result.blocked ? get().resolvedImport : null,
        loaded: true,
      })
      return result
    } catch (error) {
      set({
        importing: false,
        importError: error instanceof Error ? error.message : String(error),
        importBlockedReport: null,
      })
      throw error
    }
  },
  resetImportState: () =>
    set({
      resolvingImport: false,
      importing: false,
      importError: null,
      importBlockedReport: null,
      resolvedImport: null,
    }),
  resetDistributionState: () =>
    set({
      distributing: false,
      distributionError: null,
      lastDistributionResult: null,
    }),
}))
