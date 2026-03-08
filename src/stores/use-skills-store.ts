import { create } from 'zustand'
import { scanSkills as scanSkillsCommand } from '../lib/tauri-client'
import type { ProjectRecord, ScanSkillsRequest, ScanSkillsResult, SkillRecord } from '../types/app'

interface SkillsStoreState {
  skills: SkillRecord[]
  projects: ProjectRecord[]
  duplicates: Array<{ name: string; paths: string[] }>
  scanTaskId: string | null
  scanSkills: (request: ScanSkillsRequest) => Promise<void>
  applyScanResult: (result: ScanSkillsResult) => void
}

export const useSkillsStore = create<SkillsStoreState>((set) => ({
  skills: [],
  projects: [],
  duplicates: [],
  scanTaskId: null,
  scanSkills: async (request) => {
    const handle = await scanSkillsCommand(request)
    set({ scanTaskId: handle.taskId })
  },
  applyScanResult: (result) =>
    set({
      skills: result.skills,
      projects: result.projects,
      duplicates: result.duplicates,
      scanTaskId: null,
    }),
}))
