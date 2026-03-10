import { useEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { TemplateEditorModal } from '../components/TemplateEditorModal'
import { TemplateInjectModal } from '../components/TemplateInjectModal'
import { resolveSkillsTargets } from '../lib/skills-targets'
import { useSettingsStore } from '../stores/use-settings-store'
import { useTemplatesStore } from '../stores/use-templates-store'
import type { InjectTemplateRequest, SaveTemplateRequest, TemplateRecord } from '../types/app'

const createEmptyDraft = (): SaveTemplateRequest => ({
  id: null,
  name: '',
  description: '',
  tags: [],
  items: [],
})

const parseTags = (value: string) =>
  value
    .split(',')
    .map((item) => item.trim())
    .filter(Boolean)

export function TemplatesPage() {
  const { t } = useTranslation()
  const settings = useSettingsStore((state) => state.settings)
  const templates = useTemplatesStore((state) => state.templates)
  const repositorySkills = useTemplatesStore((state) => state.repositorySkills)
  const loading = useTemplatesStore((state) => state.loading)
  const loaded = useTemplatesStore((state) => state.loaded)
  const saving = useTemplatesStore((state) => state.saving)
  const deleting = useTemplatesStore((state) => state.deleting)
  const injecting = useTemplatesStore((state) => state.injecting)
  const error = useTemplatesStore((state) => state.error)
  const lastInjectResult = useTemplatesStore((state) => state.lastInjectResult)
  const refresh = useTemplatesStore((state) => state.refresh)
  const saveTemplate = useTemplatesStore((state) => state.saveTemplate)
  const deleteTemplate = useTemplatesStore((state) => state.deleteTemplate)
  const injectTemplate = useTemplatesStore((state) => state.injectTemplate)
  const clearInjectResult = useTemplatesStore((state) => state.clearInjectResult)

  const [editorOpen, setEditorOpen] = useState(false)
  const [injectOpen, setInjectOpen] = useState(false)
  const [draft, setDraft] = useState<SaveTemplateRequest>(createEmptyDraft)
  const [tagsInput, setTagsInput] = useState('')
  const [activeTemplateId, setActiveTemplateId] = useState<string | null>(null)

  useEffect(() => {
    if (!loaded) {
      void refresh()
    }
  }, [loaded, refresh])

  const activeTemplate = useMemo(
    () => templates.find((item) => item.id === activeTemplateId) ?? null,
    [activeTemplateId, templates],
  )
  const repositorySkillMap = useMemo(
    () => new Map(repositorySkills.map((skill) => [skill.id, skill])),
    [repositorySkills],
  )
  const visibleTargets = useMemo(
    () =>
      resolveSkillsTargets(settings).filter((target) =>
        settings.visibleSkillsTargetIds.includes(target.id),
      ),
    [settings],
  )

  const openCreateModal = () => {
    setActiveTemplateId(null)
    setDraft(createEmptyDraft())
    setTagsInput('')
    setEditorOpen(true)
  }

  const openEditModal = (template: TemplateRecord) => {
    setActiveTemplateId(template.id)
    setDraft({
      id: template.id,
      name: template.name,
      description: template.description ?? '',
      tags: template.tags,
      items: template.items.map((item) => ({
        skillRefType: item.skillRefType,
        skillRef: item.skillRef,
        displayName: item.displayName ?? null,
        orderIndex: item.orderIndex,
      })),
    })
    setTagsInput(template.tags.join(', '))
    setEditorOpen(true)
  }

  const closeEditorModal = () => {
    setEditorOpen(false)
    setDraft(createEmptyDraft())
    setTagsInput('')
    setActiveTemplateId(null)
  }

  const handleSave = async () => {
    await saveTemplate({
      ...draft,
      id: draft.id ?? null,
      name: draft.name.trim(),
      description: draft.description?.trim() ? draft.description.trim() : null,
      tags: parseTags(tagsInput),
      items: draft.items.map((item, index) => ({
        ...item,
        orderIndex: index,
        })),
    })
    closeEditorModal()
  }

  const openInjectModal = (template: TemplateRecord) => {
    setActiveTemplateId(template.id)
    clearInjectResult()
    setInjectOpen(true)
  }

  const closeInjectModal = () => {
    setInjectOpen(false)
    clearInjectResult()
  }

  const handleInject = async (request: InjectTemplateRequest) => {
    await injectTemplate(request)
  }

  return (
    <div className="space-y-6">
      <section className="rounded-box border border-base-300 bg-base-100 p-6">
        <div className="flex items-start justify-between gap-4">
          <div>
            <h2 className="text-3xl font-semibold">{t('templates.title')}</h2>
            <p className="mt-3 max-w-3xl text-sm text-base-content/65">
              {t('templates.description')}
            </p>
          </div>
          <button className="btn btn-primary" onClick={openCreateModal}>
            {t('templates.create')}
          </button>
        </div>
      </section>

      {error ? (
        <section className="rounded-box border border-error/30 bg-error/5 p-5 text-sm leading-6 text-error">
          {error}
        </section>
      ) : null}

      <section className="rounded-box border border-base-300 bg-base-100 p-6">
        <div className="flex items-center justify-between gap-3">
          <h3 className="text-lg font-semibold">{t('templates.listTitle')}</h3>
          <span className="text-sm text-base-content/55">
            {t('templates.count', { count: templates.length })}
          </span>
        </div>

        {templates.length === 0 ? (
          <div className="mt-4 rounded-box border border-dashed border-base-300 bg-base-200/60 p-4 text-sm text-base-content/60">
            {loading ? t('templates.loading') : t('templates.empty')}
          </div>
        ) : (
          <div className="mt-4 grid gap-4 lg:grid-cols-2">
            {templates.map((template) => {
              const missingCount = template.items.filter(
                (item) => !repositorySkillMap.has(item.skillRef),
              ).length

              return (
                <article
                  key={template.id}
                  className="rounded-box border border-base-300 bg-base-200/60 p-5"
                >
                  <button className="w-full text-left" onClick={() => openEditModal(template)}>
                    <div className="flex items-start justify-between gap-4">
                      <div>
                        <p className="text-lg font-semibold">{template.name}</p>
                        <p className="mt-2 text-sm text-base-content/60">
                          {template.description ?? t('templates.noDescription')}
                        </p>
                      </div>
                      {missingCount > 0 ? (
                        <span className="badge badge-warning">
                          {t('templates.card.missingCount', { count: missingCount })}
                        </span>
                      ) : null}
                    </div>
                  </button>

                  <div className="mt-4 flex flex-wrap gap-2 text-xs text-base-content/60">
                    <span>{t('templates.card.skillCount', { count: template.items.length })}</span>
                    {template.tags.length > 0
                      ? template.tags.map((tag) => (
                          <span key={tag} className="badge badge-outline">
                            {tag}
                          </span>
                        ))
                      : <span>{t('templates.noTags')}</span>}
                  </div>

                  <div className="mt-5 flex flex-wrap justify-end gap-3">
                    <button className="btn btn-sm btn-outline" onClick={() => openInjectModal(template)}>
                      {t('templates.inject.open')}
                    </button>
                    <button
                      className="btn btn-sm btn-ghost text-error"
                      disabled={deleting}
                      onClick={() => void deleteTemplate(template.id)}
                    >
                      {deleting ? t('templates.deleting') : t('templates.delete')}
                    </button>
                  </div>
                </article>
              )
            })}
          </div>
        )}
      </section>

      <TemplateEditorModal
        open={editorOpen}
        draft={draft}
        tagsInput={tagsInput}
        repositorySkills={repositorySkills}
        saving={saving}
        onClose={closeEditorModal}
        onDraftChange={setDraft}
        onTagsInputChange={setTagsInput}
        onSave={handleSave}
      />

      <TemplateInjectModal
        open={injectOpen}
        template={activeTemplate}
        targets={visibleTargets}
        injecting={injecting}
        result={lastInjectResult}
        validSkillCount={activeTemplate?.items.filter((item) => repositorySkillMap.has(item.skillRef)).length ?? 0}
        missingSkillCount={activeTemplate?.items.filter((item) => !repositorySkillMap.has(item.skillRef)).length ?? 0}
        onClose={closeInjectModal}
        onSubmit={handleInject}
      />
    </div>
  )
}
