import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { resolveSkillsTargetLabel, type SkillsTargetOption } from '../lib/skills-targets'
import { ProjectDistributionFields } from './ProjectDistributionFields'
import { ProjectDistributionResultPanel } from './ProjectDistributionResultPanel'
import type {
  InjectTemplateRequest,
  InjectTemplateResult,
  TemplateRecord,
} from '../types/app'

interface TemplateInjectModalProps {
  open: boolean
  template: TemplateRecord | null
  targets: SkillsTargetOption[]
  injecting: boolean
  result: InjectTemplateResult | null
  validSkillCount: number
  missingSkillCount: number
  onClose: () => void
  onSubmit: (request: InjectTemplateRequest) => Promise<void>
}

type TemplateInjectModalContentProps = Omit<TemplateInjectModalProps, 'open' | 'template'> & {
  template: TemplateRecord
}

export function TemplateInjectModal({
  open,
  ...props
}: TemplateInjectModalProps) {
  if (!open || !props.template) return null

  return (
    <TemplateInjectModalContent
      key={`template-inject-${props.template.id}`}
      {...props}
      template={props.template}
    />
  )
}

function TemplateInjectModalContent({
  template,
  targets,
  injecting,
  result,
  validSkillCount,
  missingSkillCount,
  onClose,
  onSubmit,
}: TemplateInjectModalContentProps) {
  const { t } = useTranslation()
  const [projectRoot, setProjectRoot] = useState('')
  const [targetType, setTargetType] = useState<'tag' | 'custom'>('tag')
  const [targetAgentId, setTargetAgentId] = useState<string>(targets[0]?.id ?? '')
  const [customRelativePath, setCustomRelativePath] = useState('')
  const [installMode, setInstallMode] = useState<'symlink' | 'copy'>('symlink')

  const resolvedTargetPath = useMemo(() => {
    if (!projectRoot.trim()) return ''

    if (targetType === 'custom') {
      return customRelativePath.trim()
        ? `${projectRoot.replace(/[\\/]+$/, '')}/${customRelativePath.replace(/^[\\/]+/, '')}`
        : ''
    }

    const target = targets.find((item) => item.id === targetAgentId)
    return target ? `${projectRoot.replace(/[\\/]+$/, '')}/${target.relativePath}` : ''
  }, [customRelativePath, projectRoot, targetAgentId, targetType, targets])

  const canSubmit =
    !injecting &&
    projectRoot.trim().length > 0 &&
    validSkillCount > 0 &&
    ((targetType === 'tag' && targetAgentId.length > 0) ||
      (targetType === 'custom' && customRelativePath.trim().length > 0))

  const chooseProjectDirectory = async () => {
    const selected = await openDialog({
      directory: true,
      multiple: false,
    })

    if (typeof selected === 'string') {
      setProjectRoot(selected)
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-base-content/45 p-4 backdrop-blur-sm md:p-6">
      <div className="flex max-h-[92vh] w-full max-w-4xl flex-col overflow-hidden rounded-box border border-base-300 bg-base-100 shadow-2xl">
        <div className="flex items-start justify-between gap-4 border-b border-base-300 px-6 py-5">
          <div>
            <h3 className="text-2xl font-semibold">{t('templates.inject.title')}</h3>
            <p className="mt-2 text-sm text-base-content/60">
              {t('templates.inject.subtitle', { name: template.name })}
            </p>
          </div>
          <button className="btn btn-ghost btn-circle" aria-label={t('common.close')} onClick={onClose}>
            <span className="text-xl font-semibold leading-none">x</span>
          </button>
        </div>

        <div className="space-y-6 overflow-y-auto p-6">
          <ProjectDistributionFields
            projectRoot={projectRoot}
            targetType={targetType}
            targetAgentId={targetAgentId}
            customRelativePath={customRelativePath}
            installMode={installMode}
            targets={targets}
            resolvedTargetPath={resolvedTargetPath}
            titleTarget={t('templates.inject.targetTitle')}
            titleMode={t('templates.inject.modeTitle')}
            titlePreview={t('templates.inject.previewTitle')}
            labelProjectRoot={t('templates.inject.projectRoot')}
            labelProjectPicker={t('templates.inject.projectPicker')}
            labelChooseDirectory={t('templates.inject.chooseDirectory')}
            labelTargetTag={t('templates.inject.targetTag')}
            labelTargetCustom={t('templates.inject.targetCustom')}
            labelCustomRelativePath={t('templates.inject.customRelativePath')}
            placeholderProjectRoot={t('templates.inject.projectRootPlaceholder')}
            placeholderCustomRelativePath={t('templates.inject.customRelativePathPlaceholder')}
            noTargetPreviewText={t('templates.inject.noTargetPreview')}
            previewMeta={
              <div className="flex flex-wrap gap-2 text-sm text-base-content/60">
                <span>{t('templates.inject.validSkills', { count: validSkillCount })}</span>
                <span>{t('templates.inject.missingSkills', { count: missingSkillCount })}</span>
              </div>
            }
            onProjectRootChange={setProjectRoot}
            onTargetTypeChange={setTargetType}
            onTargetAgentIdChange={setTargetAgentId}
            onCustomRelativePathChange={setCustomRelativePath}
            onInstallModeChange={setInstallMode}
            onChooseProjectDirectory={() => void chooseProjectDirectory()}
            renderModeLabel={(mode) => t(`templates.inject.modes.${mode}`)}
            renderTargetLabel={(target) => resolveSkillsTargetLabel(target, t)}
          />

          <ProjectDistributionResultPanel
            result={result}
            titleInstalled={t('templates.inject.result.installed', { count: result?.installed.length ?? 0 })}
            titleSkipped={t('templates.inject.result.skipped', { count: result?.skipped.length ?? 0 })}
            titleFailed={t('templates.inject.result.failed', { count: result?.failed.length ?? 0 })}
          />
        </div>

        <div className="flex justify-end gap-3 border-t border-base-300 px-6 py-5">
          <button className="btn btn-ghost" onClick={onClose}>
            {t('common.close')}
          </button>
          <button
            className="btn btn-primary h-10 min-h-[2.5rem] border-none bg-primary px-8 text-[var(--text-inverse)] shadow-[var(--shadow-neon-primary)] transition-all duration-300 hover:bg-primary hover:brightness-110 hover:shadow-[0_0_30px_rgba(var(--color-primary),0.6)] disabled:bg-base-300 disabled:text-base-content/30 disabled:shadow-none"
            disabled={!canSubmit}
            onClick={() =>
              void onSubmit({
                templateId: template.id,
                projectRoot: projectRoot.trim(),
                targetType,
                targetAgentId: targetType === 'tag' ? targetAgentId : null,
                customRelativePath: targetType === 'custom' ? customRelativePath.trim() : null,
                installMode,
              }).catch(() => undefined)
            }
          >
            {injecting ? t('templates.inject.injecting') : t('templates.inject.confirm')}
          </button>
        </div>
      </div>
    </div>
  )
}
