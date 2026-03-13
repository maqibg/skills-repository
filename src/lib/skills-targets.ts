import type { AppSettings, BuiltinSkillsTarget, CustomSkillsTarget } from '../types/app'

export interface SkillsTargetOption {
  id: string
  label: string
  labelKey?: string | null
  relativePath: string
  isCustom: boolean
}

export const resolveSkillsTargets = (
  builtinSkillsTargets: BuiltinSkillsTarget[],
  settings: AppSettings,
): SkillsTargetOption[] => [
  ...builtinSkillsTargets.map((target) => ({ ...target, isCustom: false })),
  ...settings.customSkillsTargets.map((target) => ({
    ...target,
    isCustom: true,
  })),
]

export const resolveSkillsTargetLabel = (
  target: Pick<SkillsTargetOption, 'label' | 'labelKey'>,
  translate: (key: string, options?: Record<string, unknown>) => string,
) => (target.labelKey ? translate(target.labelKey, { defaultValue: target.label }) : target.label)

export const createCustomSkillsTargetId = (label: string) => {
  const normalized = label
    .toLowerCase()
    .trim()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')

  return `custom-${normalized || crypto.randomUUID()}`
}

export const normalizeRelativeSkillsPath = (relativePath: string) =>
  relativePath.replace(/\\/g, '/').trim()

export const hasSkillsTarget = (
  builtinSkillsTargets: BuiltinSkillsTarget[],
  settings: AppSettings,
  targetId: string,
) => resolveSkillsTargets(builtinSkillsTargets, settings).some((target) => target.id === targetId)

export const removeCustomSkillsTarget = (
  customSkillsTargets: CustomSkillsTarget[],
  targetId: string,
) => customSkillsTargets.filter((target) => target.id !== targetId)
