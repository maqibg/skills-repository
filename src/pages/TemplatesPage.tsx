import { useEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useTemplatesStore } from '../stores/use-templates-store'
import type { SaveTemplateRequest } from '../types/app'

const createEmptyDraft = (): SaveTemplateRequest => ({
  id: null,
  name: '',
  description: '',
  tags: [],
})

const parseTags = (value: string) =>
  value
    .split(',')
    .map((item) => item.trim())
    .filter(Boolean)

export function TemplatesPage() {
  const { t } = useTranslation()
  const templates = useTemplatesStore((state) => state.templates)
  const selectedTemplateId = useTemplatesStore((state) => state.selectedTemplateId)
  const loading = useTemplatesStore((state) => state.loading)
  const loaded = useTemplatesStore((state) => state.loaded)
  const saving = useTemplatesStore((state) => state.saving)
  const deleting = useTemplatesStore((state) => state.deleting)
  const error = useTemplatesStore((state) => state.error)
  const refresh = useTemplatesStore((state) => state.refresh)
  const selectTemplate = useTemplatesStore((state) => state.selectTemplate)
  const saveTemplate = useTemplatesStore((state) => state.saveTemplate)
  const deleteTemplate = useTemplatesStore((state) => state.deleteTemplate)

  const [draft, setDraft] = useState<SaveTemplateRequest>(createEmptyDraft)
  const [tagsInput, setTagsInput] = useState('')

  useEffect(() => {
    if (!loaded) {
      void refresh()
    }
  }, [loaded, refresh])

  const selectedTemplate = useMemo(
    () => templates.find((item) => item.id === selectedTemplateId) ?? null,
    [selectedTemplateId, templates],
  )

  const resetDraft = () => {
    selectTemplate(null)
    setDraft(createEmptyDraft())
    setTagsInput('')
  }

  const loadTemplateIntoDraft = (templateId: string) => {
    const template = templates.find((item) => item.id === templateId)
    if (!template) return

    selectTemplate(template.id)
    setDraft({
      id: template.id,
      name: template.name,
      description: template.description ?? '',
      tags: template.tags,
    })
    setTagsInput(template.tags.join(', '))
  }

  const handleSave = async () => {
    const saved = await saveTemplate({
      id: draft.id ?? null,
      name: draft.name.trim(),
      description: draft.description?.trim() ? draft.description.trim() : null,
      tags: parseTags(tagsInput),
    })

    setDraft({
      id: saved.id,
      name: saved.name,
      description: saved.description ?? '',
      tags: saved.tags,
    })
    setTagsInput(saved.tags.join(', '))
  }

  const handleDelete = async () => {
    if (!selectedTemplateId) return
    await deleteTemplate(selectedTemplateId)
    resetDraft()
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
          <button className="btn btn-outline" onClick={resetDraft}>
            {t('templates.create')}
          </button>
        </div>
      </section>

      {error ? (
        <section className="rounded-box border border-error/30 bg-error/5 p-5 text-sm leading-6 text-error">
          {error}
        </section>
      ) : null}

      <section className="grid gap-6 xl:grid-cols-[1fr_1.4fr]">
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
            <div className="mt-4 space-y-3">
              {templates.map((template) => (
                <button
                  key={template.id}
                  className={`w-full rounded-box border p-4 text-left transition-colors ${
                    selectedTemplateId === template.id
                      ? 'border-primary bg-primary/5'
                      : 'border-base-300 bg-base-200/60 hover:bg-base-200'
                  }`}
                  onClick={() => loadTemplateIntoDraft(template.id)}
                >
                  <p className="font-semibold">{template.name}</p>
                  <p className="mt-2 text-sm text-base-content/60">
                    {template.description ?? t('templates.noDescription')}
                  </p>
                  <div className="mt-3 flex flex-wrap gap-2 text-xs text-base-content/55">
                    {template.tags.length === 0 ? (
                      <span>{t('templates.noTags')}</span>
                    ) : (
                      template.tags.map((tag) => (
                        <span key={`${template.id}-${tag}`} className="badge badge-ghost">
                          {tag}
                        </span>
                      ))
                    )}
                  </div>
                </button>
              ))}
            </div>
          )}
        </section>

        <section className="rounded-box border border-base-300 bg-base-100 p-6">
          <div className="flex items-center justify-between gap-3">
            <h3 className="text-lg font-semibold">
              {selectedTemplate ? t('templates.editTitle') : t('templates.createTitle')}
            </h3>
          </div>

          <div className="mt-4 grid gap-4">
            <label className="form-control">
              <span className="label-text">{t('templates.fields.name')}</span>
              <input
                className="input input-bordered"
                value={draft.name}
                onChange={(event) =>
                  setDraft((state) => ({ ...state, name: event.target.value }))
                }
                placeholder={t('templates.placeholders.name')}
              />
            </label>

            <label className="form-control">
              <span className="label-text">{t('templates.fields.description')}</span>
              <textarea
                className="textarea textarea-bordered min-h-28"
                value={draft.description ?? ''}
                onChange={(event) =>
                  setDraft((state) => ({ ...state, description: event.target.value }))
                }
                placeholder={t('templates.placeholders.description')}
              />
            </label>

            <label className="form-control">
              <span className="label-text">{t('templates.fields.tags')}</span>
              <input
                className="input input-bordered"
                value={tagsInput}
                onChange={(event) => setTagsInput(event.target.value)}
                placeholder={t('templates.placeholders.tags')}
              />
            </label>
          </div>

          <div className="mt-6 rounded-box border border-dashed border-base-300 bg-base-200/60 p-4 text-sm text-base-content/60">
            {t('templates.emptyTemplateNotice')}
          </div>

          <div className="mt-6 flex flex-wrap justify-end gap-3">
            {selectedTemplate ? (
              <button
                className="btn btn-ghost text-error"
                onClick={() => void handleDelete()}
                disabled={deleting}
              >
                {deleting ? t('templates.deleting') : t('templates.delete')}
              </button>
            ) : null}
            <button
              className="btn btn-primary"
              onClick={() => void handleSave()}
              disabled={saving || draft.name.trim().length === 0}
            >
              {saving ? t('templates.saving') : t('templates.save')}
            </button>
          </div>
        </section>
      </section>
    </div>
  )
}
