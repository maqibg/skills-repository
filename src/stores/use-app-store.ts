import { create } from 'zustand'
import type {
  BootstrapPayload,
  BuiltinSkillsTarget,
  RepositoryStorageInfo,
  SystemInfo,
} from '../types/app'

interface AppStoreState {
  bootstrapping: boolean
  bootstrapped: boolean
  error: string | null
  system: SystemInfo | null
  builtinSkillsTargets: BuiltinSkillsTarget[]
  repositoryStorage: RepositoryStorageInfo | null
  setBootstrapPayload: (payload: BootstrapPayload) => void
  setRepositoryStorage: (repositoryStorage: RepositoryStorageInfo) => void
  setBootstrapError: (message: string) => void
}

export const useAppStore = create<AppStoreState>((set) => ({
  bootstrapping: true,
  bootstrapped: false,
  error: null,
  system: null,
  builtinSkillsTargets: [],
  repositoryStorage: null,
  setBootstrapPayload: (payload) =>
    set({
      bootstrapping: false,
      bootstrapped: true,
      error: null,
      system: payload.system,
      builtinSkillsTargets: payload.builtinSkillsTargets,
      repositoryStorage: payload.repositoryStorage,
    }),
  setRepositoryStorage: (repositoryStorage) => set({ repositoryStorage }),
  setBootstrapError: (message) =>
    set({
      bootstrapping: false,
      error: message,
    }),
}))
