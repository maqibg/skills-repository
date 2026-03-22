import { useDeferredValue, useEffect, useMemo, useState } from 'react'
import { HighlightedText } from '../components/common/HighlightedText'
import { useTranslation } from 'react-i18next'
import { RepositoryDistributeModal } from '../components/RepositoryDistributeModal'
import { RepositoryImportModal } from '../components/RepositoryImportModal'
import { normalizeDisplayPath } from '../lib/normalize-display-path'
import {
  buildRepositoryPageNumbers,
  buildRepositorySearchIndex,
  paginateRepositorySearchResults,
  searchRepositoryIndex,
} from '../lib/repository-search'
import { resolveSkillsTargets } from '../lib/skills-targets'
import { openSourceReference } from '../lib/tauri-client'
import { useAppStore } from '../stores/use-app-store'
import { useRepositoryStore } from '../stores/use-repository-store'
import { useSettingsStore } from '../stores/use-settings-store'
import type {
  BatchRepositorySkillUpdateResult,
  BatchDistributeRepositorySkillsRequest,
  ImportRepositorySkillRequest,
  RepositorySkillSummary,
  RepositorySkillUpdateItemResult,
  RepositoryImportSourceKind,
} from '../types/app'

const SEARCH_PAGE_SIZE = 10

const formatInstalledAt = (value: number, locale: string) =>
  new Intl.DateTimeFormat(locale, {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  }).format(new Date(value * 1000))

const resolveSourceLabel = (
  sourceType: string,
  sourceMarket: string | null | undefined,
  t: (key: string, options?: Record<string, unknown>) => string,
) => {
  if (sourceType === 'market') {
    return t('repository.sourceMarket', { market: sourceMarket ?? 'market' })
  }
  if (sourceType === 'github') {
    return t('repository.sourceGithub')
  }
  if (sourceType === 'local') {
    return t('repository.sourceLocal')
  }
  return t('repository.sourceUnknown')
}

const resolveStatusKey = (
  securityLevel: string,
  blocked: boolean,
  riskOverrideApplied?: boolean,
) => {
  if (riskOverrideApplied) return 'overridden'
  if (blocked) return 'blocked'
  if (securityLevel === 'safe') return 'safe'
  if (securityLevel === 'low') return 'low'
  if (securityLevel === 'medium') return 'medium'
  return 'unknown'
}

const resolveDescription = (
  value: string | null | undefined,
  t: (key: string, options?: Record<string, unknown>) => string,
) => (value?.trim() ? value : t('repository.descriptionMissing'))

const logSourceOpenFailure = (error: unknown) => {
  console.error('Failed to open source reference:', error)
}

const formatUpdateVersion = (
  value: string | null | undefined,
  t: (key: string, options?: Record<string, unknown>) => string,
) => (value?.trim() ? value : t('repository.update.versionUnknown'))

const resolveUpdateTone = (status: string) => {
  if (status === 'updated') return 'border-success/30 bg-success/5 text-success'
  if (status === 'skipped') return 'border-info/30 bg-info/5 text-info'
  return 'border-error/30 bg-error/5 text-error'
}

const extractUpdateError = (result: RepositorySkillUpdateItemResult) => {
  if (!result.details || typeof result.details !== 'object') return null
  const message = result.details.error
  return typeof message === 'string' && message.trim() ? message : null
}

const resolveUpdateMessage = (
  result: RepositorySkillUpdateItemResult,
  t: (key: string, options?: Record<string, unknown>) => string,
) => {
  if (result.status === 'updated') {
    return t('repository.update.messages.updated', {
      from: formatUpdateVersion(result.previousVersion, t),
      to: formatUpdateVersion(result.currentVersion, t),
    })
  }

  if (result.status === 'skipped') {
    return t('repository.update.messages.skipped', {
      version: formatUpdateVersion(result.currentVersion ?? result.previousVersion, t),
    })
  }

  if (result.reasonCode === 'blocked_by_security_scan') {
    return t('repository.update.messages.blocked', {
      version: formatUpdateVersion(result.currentVersion, t),
    })
  }

  return extractUpdateError(result) ?? t('repository.update.messages.failed')
}

const flattenUpdateResults = (result: BatchRepositorySkillUpdateResult | null) =>
  result ? [...result.updated, ...result.skipped, ...result.failed] : []

interface SearchableRepositorySkill extends RepositorySkillSummary {
  sourceLabel: string
  statusKey: string
  statusLabel: string
  keywords: string[]
}

export function RepositoryPage() {
  const { t, i18n } = useTranslation()
  const [importOpen, setImportOpen] = useState(false)
  const [query, setQuery] = useState('')
  const [currentPage, setCurrentPage] = useState(1)
  const deferredQuery = useDeferredValue(query)
  const settings = useSettingsStore((state) => state.settings)
  const builtinSkillsTargets = useAppStore((state) => state.builtinSkillsTargets)
  const items = useRepositoryStore((state) => state.items)
  const loading = useRepositoryStore((state) => state.loading)
  const loaded = useRepositoryStore((state) => state.loaded)
  const error = useRepositoryStore((state) => state.error)
  const selectedDetail = useRepositoryStore((state) => state.selectedDetail)
  const detailLoading = useRepositoryStore((state) => state.detailLoading)
  const detailError = useRepositoryStore((state) => state.detailError)
  const uninstallingSkillId = useRepositoryStore((state) => state.uninstallingSkillId)
  const deletePreview = useRepositoryStore((state) => state.deletePreview)
  const deletePreviewLoading = useRepositoryStore((state) => state.deletePreviewLoading)
  const deletePreviewError = useRepositoryStore((state) => state.deletePreviewError)
  const distributionOpen = useRepositoryStore((state) => state.distributionOpen)
  const distributing = useRepositoryStore((state) => state.distributing)
  const distributionError = useRepositoryStore((state) => state.distributionError)
  const lastDistributionResult = useRepositoryStore((state) => state.lastDistributionResult)
  const updatingSkillIds = useRepositoryStore((state) => state.updatingSkillIds)
  const bulkUpdating = useRepositoryStore((state) => state.bulkUpdating)
  const updateError = useRepositoryStore((state) => state.updateError)
  const lastUpdateResult = useRepositoryStore((state) => state.lastUpdateResult)
  const resolvingImport = useRepositoryStore((state) => state.resolvingImport)
  const importing = useRepositoryStore((state) => state.importing)
  const importError = useRepositoryStore((state) => state.importError)
  const importBlockedReport = useRepositoryStore((state) => state.importBlockedReport)
  const resolvedImport = useRepositoryStore((state) => state.resolvedImport)
  const refresh = useRepositoryStore((state) => state.refresh)
  const loadDetail = useRepositoryStore((state) => state.loadDetail)
  const closeDetail = useRepositoryStore((state) => state.closeDetail)
  const loadDeletePreview = useRepositoryStore((state) => state.loadDeletePreview)
  const clearDeletePreview = useRepositoryStore((state) => state.clearDeletePreview)
  const uninstall = useRepositoryStore((state) => state.uninstall)
  const openDistribution = useRepositoryStore((state) => state.openDistribution)
  const closeDistribution = useRepositoryStore((state) => state.closeDistribution)
  const batchDistributeSkills = useRepositoryStore((state) => state.batchDistributeSkills)
  const resetDistributionState = useRepositoryStore((state) => state.resetDistributionState)
  const updateSkill = useRepositoryStore((state) => state.updateSkill)
  const updateGithubSkills = useRepositoryStore((state) => state.updateGithubSkills)
  const clearUpdateState = useRepositoryStore((state) => state.clearUpdateState)
  const resolveImport = useRepositoryStore((state) => state.resolveImport)
  const importSkill = useRepositoryStore((state) => state.importSkill)
  const resetImportState = useRepositoryStore((state) => state.resetImportState)

  useEffect(() => {
    void refresh()
  }, [refresh])

  const visibleTargets = resolveSkillsTargets(builtinSkillsTargets, settings).filter((target) =>
    settings.visibleSkillsTargetIds.includes(target.id),
  )
  const githubItems = useMemo(
    () => items.filter((item) => item.sourceType === 'github'),
    [items],
  )
  const searchableItems = useMemo<SearchableRepositorySkill[]>(
    () =>
      items.map((item) => {
        const statusKey = resolveStatusKey(item.securityLevel, item.blocked, item.riskOverrideApplied)
        return {
          ...item,
          sourceLabel: resolveSourceLabel(item.sourceType, item.sourceMarket, t),
          statusKey,
          statusLabel: t(`repository.statusValues.${statusKey}`),
          keywords: [item.sourceType, item.sourceMarket ?? '', item.securityLevel].filter(Boolean),
        }
      }),
    [items, t],
  )
  const searchIndex = useMemo(
    () => buildRepositorySearchIndex(searchableItems),
    [searchableItems],
  )
  const searchResults = useMemo(
    () => searchRepositoryIndex(searchIndex, deferredQuery),
    [searchIndex, deferredQuery],
  )
  const isSearching = deferredQuery.trim().length > 0
  const paginatedResults = useMemo(
    () => paginateRepositorySearchResults(searchResults, currentPage, SEARCH_PAGE_SIZE),
    [currentPage, searchResults],
  )
  const visibleResults = isSearching ? paginatedResults.items : searchResults
  const searchPageNumbers = useMemo(
    () => buildRepositoryPageNumbers(paginatedResults.page, paginatedResults.pageCount),
    [paginatedResults.page, paginatedResults.pageCount],
  )
  const updateItems = useMemo(() => flattenUpdateResults(lastUpdateResult), [lastUpdateResult])
  const searchQueryDisplay = deferredQuery.trim()
  const visibleRangeStart = paginatedResults.total === 0 ? 0 : paginatedResults.startIndex + 1
  const visibleRangeEnd = paginatedResults.endIndex

  const openImportModal = () => {
    resetImportState()
    setImportOpen(true)
  }

  const closeImportModal = () => {
    setImportOpen(false)
    resetImportState()
  }

  const handleOpenDistribution = () => {
    resetDistributionState()
    openDistribution()
  }

  const handleCloseDistribution = () => {
    closeDistribution()
    resetDistributionState()
  }

  const handleOpenDeletePreview = async (skillId: string) => {
    await loadDeletePreview(skillId)
  }

  const handleCloseDeletePreview = () => {
    clearDeletePreview()
  }

  const handleResolveImport = async (sourceKind: RepositoryImportSourceKind, input: string) => {
    await resolveImport({ sourceKind, input })
  }

  const handleImportSkill = async (request: ImportRepositorySkillRequest) => {
    return importSkill(request)
  }

  const handleBatchDistributeSkills = async (request: BatchDistributeRepositorySkillsRequest) => {
    await batchDistributeSkills(request)
  }

  const handleUpdateSkill = async (skillId: string) => {
    await updateSkill(skillId)
  }

  const handleUpdateGithubSkills = async () => {
    await updateGithubSkills()
  }

  const handleClearSearch = () => {
    setQuery('')
    setCurrentPage(1)
  }

  const handleQueryChange = (value: string) => {
    setQuery(value)
    setCurrentPage(1)
  }

  return (
    <div className="space-y-8 p-8">
      {/* Header Section */}
      <section className="relative overflow-hidden rounded-lg border border-[var(--border-subtle)] bg-base-100 p-8 shadow-[inset_0_0_20px_rgba(var(--color-primary),0.05)]">
        <div className="absolute top-0 left-0 h-1 w-full bg-gradient-to-r from-primary/0 via-primary/50 to-primary/0 opacity-20"></div>
        <div className="flex flex-col gap-6 md:flex-row md:items-center md:justify-between">
          <div className="flex-1">
            <h2 className="text-3xl font-bold tracking-tight text-base-content">{t('repository.title')}</h2>
            <p className="mt-3 max-w-3xl text-sm leading-relaxed text-base-content/70">
              {t('repository.description')}
            </p>
          </div>
          <div className="flex flex-wrap gap-3">
            <button
              className="btn btn-primary h-10 w-40 min-h-[2.5rem] border-none bg-primary px-4 text-[var(--text-inverse)] transition-all duration-300 hover:bg-primary hover:shadow-[var(--shadow-neon-primary)]"
              onClick={openImportModal}
            >
              <i className="hn hn-download-alt text-lg"></i>
              {t('repository.import.open')}
            </button>
            <button
              className="btn btn-outline h-10 min-h-[2.5rem] border-[var(--border-subtle)] px-4 text-base-content hover:border-primary hover:bg-primary/10 hover:text-primary"
              disabled={githubItems.length === 0 || bulkUpdating}
              onClick={() => void handleUpdateGithubSkills()}
            >
              {bulkUpdating ? (
                <span className="loading loading-spinner loading-sm"></span>
              ) : (
                <i className="hn hn-check text-lg"></i>
              )}
              {bulkUpdating ? t('repository.update.running') : t('repository.update.open')}
            </button>
            <button
              className="btn btn-outline h-10 w-40 min-h-[2.5rem] border-[var(--border-subtle)] px-4 text-base-content hover:border-primary hover:bg-primary/10 hover:text-primary"
              disabled={items.length === 0}
              onClick={handleOpenDistribution}
            >
              <i className="hn hn-share text-lg"></i>
              {t('repository.distribute.open')}
            </button>
          </div>
        </div>
      </section>

      {/* Skills List Section */}
      <section className="overflow-hidden rounded-lg border border-[var(--border-subtle)] bg-base-100 shadow-[inset_0_0_20px_rgba(var(--color-primary),0.02)]">
        <div className="border-b border-[var(--border-subtle)] bg-base-200/30 px-6 py-5">
          <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
            <div>
              <p className="text-sm font-semibold text-base-content/75">{t('repository.title')}</p>
              <div className="mt-1 flex flex-wrap items-center gap-2 text-xs text-base-content/50">
                <span>{t('repository.update.githubCount', { count: githubItems.length })}</span>
                {isSearching ? (
                  <span>
                    {t('repository.search.pageSummary', {
                      start: visibleRangeStart,
                      end: visibleRangeEnd,
                      total: paginatedResults.total,
                    })}
                  </span>
                ) : null}
              </div>
            </div>
            <label className="input input-bordered flex min-w-0 items-center gap-2 border-[var(--border-subtle)] bg-base-100 lg:w-96">
              <i className="hn hn-search text-base-content/45" aria-hidden />
              <input
                type="text"
                className="grow"
                value={query}
                onChange={(event) => handleQueryChange(event.target.value)}
                placeholder={t('repository.searchPlaceholder')}
                aria-label={t('repository.searchPlaceholder')}
              />
              {query ? (
                <button
                  type="button"
                  className="btn btn-ghost btn-xs btn-circle"
                  onClick={handleClearSearch}
                  aria-label={t('repository.search.clear')}
                >
                  <i className="hn hn-times text-xs"></i>
                </button>
              ) : null}
            </label>
          </div>

          {isSearching ? (
            <div className="mt-4 flex flex-col gap-3 rounded-lg border border-[var(--border-subtle)] bg-base-100/70 px-4 py-3 sm:flex-row sm:items-center sm:justify-between">
              <div>
                <p className="text-sm font-semibold text-base-content">
                  {t('repository.search.activeQuery', { query: searchQueryDisplay })}
                </p>
                <p className="mt-1 text-xs text-base-content/50">
                  {t('repository.search.pageSummary', {
                    start: visibleRangeStart,
                    end: visibleRangeEnd,
                    total: paginatedResults.total,
                  })}
                </p>
              </div>
              <div className="flex flex-wrap items-center gap-2">
                {paginatedResults.pageCount > 1 ? (
                  <span className="badge border-0 bg-base-200/80 px-3 py-3 text-base-content/65">
                    {t('repository.search.currentPage', {
                      page: paginatedResults.page,
                      total: paginatedResults.pageCount,
                    })}
                  </span>
                ) : null}
                <button className="btn btn-outline btn-sm" onClick={handleClearSearch} type="button">
                  {t('repository.search.clear')}
                </button>
              </div>
            </div>
          ) : null}

          {updateError ? (
            <div className="mt-4 rounded-lg border border-error/30 bg-error/5 px-4 py-3 text-sm text-error">
              {updateError}
            </div>
          ) : null}

          {lastUpdateResult ? (
            <div className="mt-4 rounded-lg border border-[var(--border-subtle)] bg-base-100/70 p-4">
              <div className="flex flex-wrap items-center justify-between gap-3">
                <div>
                  <p className="text-sm font-semibold text-base-content">
                    {t('repository.update.resultTitle')}
                  </p>
                  <p className="mt-1 text-xs text-base-content/50">
                    {t('repository.update.resultCount', { count: updateItems.length })}
                  </p>
                </div>
                <div className="flex flex-wrap gap-2 text-xs">
                  <span className="badge border-0 bg-success/10 text-success">
                    {t('repository.update.updatedCount', { count: lastUpdateResult.updated.length })}
                  </span>
                  <span className="badge border-0 bg-info/10 text-info">
                    {t('repository.update.skippedCount', { count: lastUpdateResult.skipped.length })}
                  </span>
                  <span className="badge border-0 bg-error/10 text-error">
                    {t('repository.update.failedCount', { count: lastUpdateResult.failed.length })}
                  </span>
                  <button
                    className="btn btn-ghost btn-xs"
                    onClick={clearUpdateState}
                    type="button"
                  >
                    {t('common.close')}
                  </button>
                </div>
              </div>

              <div className="mt-4 space-y-3">
                {updateItems.map((result) => (
                  <div
                    key={`${result.skillId}:${result.status}:${result.reasonCode}`}
                    className={`rounded-lg border px-4 py-3 text-sm ${resolveUpdateTone(result.status)}`}
                  >
                    <div className="flex flex-wrap items-center justify-between gap-2">
                      <p className="font-medium">{result.skillName}</p>
                      <span className="badge badge-outline border-current/20 text-current">
                        {t(`repository.update.statuses.${result.status}`)}
                      </span>
                    </div>
                    <p className="mt-2 leading-6">{resolveUpdateMessage(result, t)}</p>
                    {result.copyDistributionCount > 0 ? (
                      <p className="mt-2 text-xs opacity-80">
                        {t('repository.update.copyDistributionCount', {
                          count: result.copyDistributionCount,
                        })}
                      </p>
                    ) : null}
                  </div>
                ))}
              </div>
            </div>
          ) : null}
        </div>

        {loading ? (
          <div className="flex flex-col items-center justify-center p-12 text-center">
            <span className="loading loading-spinner loading-lg text-primary"></span>
            <p className="mt-4 text-sm text-base-content/60">{t('repository.loading')}</p>
          </div>
        ) : error ? (
          <div className="flex items-center gap-3 bg-error/10 p-6 text-error">
              <i className="hn hn-exclaimation text-lg"></i>
              <span className="text-sm font-medium">{error}</span>
           </div>
        ) : loaded && items.length === 0 ? (
          <div className="flex flex-col items-center justify-center p-16 text-center">
            <div className="mb-4 rounded-full bg-base-200 p-4 text-base-content/30">
              <i className="hn hn-box-usd text-3xl"></i>
            </div>
            <p className="text-base font-medium text-base-content/60">{t('repository.empty')}</p>
          </div>
        ) : loaded && isSearching && searchResults.length === 0 ? (
          <div className="flex flex-col items-center justify-center p-16 text-center">
            <div className="mb-4 rounded-full bg-primary/10 p-4 text-primary/70">
              <i className="hn hn-search text-3xl"></i>
            </div>
            <h3 className="text-lg font-semibold text-base-content">{t('repository.search.emptyTitle')}</h3>
            <p className="mt-3 max-w-xl text-sm leading-6 text-base-content/60">
              {t('repository.search.emptyDescription', { query: searchQueryDisplay })}
            </p>
            <button type="button" className="btn btn-primary mt-6" onClick={handleClearSearch}>
              {t('repository.search.clear')}
            </button>
          </div>
        ) : (
          <div>
            <div className="overflow-x-auto">
              <table className="table-fixed table w-full">
                <thead>
                  <tr className="border-b border-[var(--border-subtle)] bg-base-200/50 text-xs font-bold uppercase tracking-wider text-base-content/40">
                    <th className="py-4 pl-6 text-left">{t('common.name')}</th>
                    <th className="w-28 text-center">{t('repository.source')}</th>
                    <th className="w-32 text-center">{t('repository.installedAt')}</th>
                    <th className="w-28 text-center">{t('common.status')}</th>
                    <th className="w-48 pr-6 text-center">{t('repository.actions')}</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-[var(--border-subtle)]">
                  {visibleResults.map(({ item, highlights }) => (
                    <tr key={item.id} className="group transition-colors hover:bg-base-200/50">
                      <td className="py-4 pl-6 text-left">
                        <div className="flex items-start gap-3">
                          <div className="mt-1 flex h-8 w-8 shrink-0 items-center justify-center rounded bg-primary/10 text-primary">
                            <i className="hn hn-code-block text-base leading-none"></i>
                          </div>
                          <div className="min-w-0">
                            <HighlightedText
                              text={item.name}
                              ranges={highlights.name}
                              className="block font-bold text-base-content/90 transition-colors group-hover:text-primary"
                            />
                            <HighlightedText
                              text={resolveDescription(item.description, t)}
                              ranges={highlights.description}
                              className="mt-1 block line-clamp-2 text-sm leading-6 text-base-content/50"
                              highlightClassName="rounded-sm bg-primary/12 px-0.5 text-base-content"
                            />
                            {isSearching ? (
                              <div className="mt-2 flex flex-wrap gap-2 text-xs text-base-content/40">
                                <span className="rounded bg-base-200/80 px-2 py-1 font-mono">
                                  <HighlightedText
                                    text={item.slug}
                                    ranges={highlights.slug}
                                    className="font-mono"
                                    highlightClassName="rounded-sm bg-primary/12 px-0.5 text-primary"
                                  />
                                </span>
                              </div>
                            ) : null}
                          </div>
                        </div>
                      </td>
                      <td className="text-center text-sm text-base-content/70">
                        <HighlightedText
                          text={item.sourceLabel}
                          ranges={highlights.source}
                          className="text-sm text-base-content/70"
                        />
                      </td>
                      <td className="text-center font-mono text-xs text-base-content/50">
                        {formatInstalledAt(item.installedAt, i18n.language)}
                      </td>
                      <td className="text-center">
                        <span className={`inline-flex whitespace-nowrap badge badge-sm gap-1 border-0 font-bold ${
                          item.statusKey === 'blocked' ? 'bg-error/20 text-error' :
                          item.statusKey === 'overridden' ? 'bg-warning/20 text-warning' :
                          item.statusKey === 'safe' ? 'bg-success/20 text-success' :
                          item.statusKey === 'low' ? 'bg-success/10 text-success/80' :
                          'bg-warning/20 text-warning'
                        }`}>
                          <i className={`hn ${
                            item.statusKey === 'blocked' ? 'hn-lock' :
                            item.statusKey === 'overridden' ? 'hn-shield' :
                            item.statusKey === 'safe' ? 'hn-check-circle' :
                            'hn-exclaimation'
                          } text-xs`}></i>
                          {t(`repository.statusValues.${item.statusKey}`)}
                        </span>
                      </td>
                      <td className="pr-6 text-center">
                        <div className="flex justify-center gap-2 opacity-0 transition-opacity group-hover:opacity-100">
                          {item.sourceType === 'github' ? (
                            <button
                              className="btn btn-ghost btn-sm px-3 text-xs text-primary hover:bg-primary/10"
                              onClick={() => void handleUpdateSkill(item.id)}
                              disabled={bulkUpdating || updatingSkillIds.includes(item.id)}
                              title={t('repository.update.single')}
                            >
                              {updatingSkillIds.includes(item.id) ? (
                                <span className="loading loading-spinner loading-xs"></span>
                              ) : null}
                              {updatingSkillIds.includes(item.id)
                                ? t('repository.update.running')
                                : t('repository.update.single')}
                            </button>
                          ) : null}
                          <button
                            className="btn btn-square btn-ghost btn-sm text-base-content/70 hover:bg-primary/10 hover:text-primary"
                            onClick={() => void loadDetail(item.id)}
                            title={t('repository.view')}
                          >
                            <i className="hn hn-eye"></i>
                          </button>
                          <button
                            className="btn btn-square btn-ghost btn-sm text-error/70 hover:bg-error/10 hover:text-error"
                            onClick={() => void handleOpenDeletePreview(item.id)}
                            disabled={bulkUpdating || uninstallingSkillId === item.id}
                            title={t('repository.uninstall')}
                          >
                            {uninstallingSkillId === item.id ? (
                              <span className="loading loading-spinner loading-xs"></span>
                            ) : (
                              <i className="hn hn-trash"></i>
                            )}
                          </button>
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>

            {isSearching && paginatedResults.pageCount > 1 ? (
              <div className="flex flex-col gap-4 border-t border-[var(--border-subtle)] px-6 py-5 sm:flex-row sm:items-center sm:justify-between">
                <p className="text-sm text-base-content/60">
                  {t('repository.search.pageSummary', {
                    start: visibleRangeStart,
                    end: visibleRangeEnd,
                    total: paginatedResults.total,
                  })}
                </p>
                <div className="join self-start sm:self-auto">
                  <button
                    type="button"
                    className="btn btn-sm join-item border-[var(--border-subtle)] bg-base-100"
                    onClick={() => setCurrentPage(Math.max(1, paginatedResults.page - 1))}
                    disabled={paginatedResults.page <= 1}
                  >
                    {t('repository.search.previous')}
                  </button>
                  {searchPageNumbers.map((pageNumber) => (
                    <button
                      key={pageNumber}
                      type="button"
                      className={`btn btn-sm join-item border-[var(--border-subtle)] ${
                        pageNumber === paginatedResults.page
                          ? 'btn-primary'
                          : 'bg-base-100 text-base-content/70'
                      }`}
                      onClick={() => setCurrentPage(pageNumber)}
                    >
                      {pageNumber}
                    </button>
                  ))}
                  <button
                    type="button"
                    className="btn btn-sm join-item border-[var(--border-subtle)] bg-base-100"
                    onClick={() =>
                      setCurrentPage(Math.min(paginatedResults.pageCount, paginatedResults.page + 1))
                    }
                    disabled={paginatedResults.page >= paginatedResults.pageCount}
                  >
                    {t('repository.search.next')}
                  </button>
                </div>
              </div>
            ) : null}
          </div>
        )}
      </section>

      {/* Detail Modal - Enhanced */}
      {selectedDetail || detailLoading || detailError ? (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-[var(--bg-modal-overlay)] p-6 backdrop-blur-sm transition-all duration-300">
          <div className="relative flex max-h-[85vh] w-full max-w-4xl flex-col overflow-hidden rounded-xl border border-[var(--border-subtle)] bg-[var(--bg-modal-panel)] shadow-[0_0_50px_rgba(0,0,0,0.5)]">
            
            {/* Modal Header */}
            <div className="flex items-start justify-between gap-4 border-b border-[var(--border-subtle)] bg-base-100/50 px-8 py-6 backdrop-blur-md">
              <div className="min-w-0">
                <h3 className="truncate text-2xl font-bold text-base-content">
                  {selectedDetail?.name ?? t('repository.detailTitle')}
                </h3>
                {selectedDetail ? (
                  <div className="mt-3 flex flex-col gap-2">
                    <div className="rounded-lg border border-[var(--border-subtle)] bg-base-200/40 p-4">
                      <p className="text-xs font-semibold uppercase tracking-wide text-base-content/40">
                        {t('repository.summaryTitle')}
                      </p>
                      <p className="mt-2 text-sm leading-6 text-base-content/75">
                        {resolveDescription(selectedDetail.description, t)}
                      </p>
                    </div>
                    <p className="break-all font-mono text-xs text-base-content/40">
                      {normalizeDisplayPath(selectedDetail.canonicalPath)}
                    </p>
                    <div className="flex flex-wrap items-center gap-3">
                      <span className="badge badge-outline border-[var(--border-subtle)] text-xs text-base-content/60">
                        {formatInstalledAt(selectedDetail.installedAt, i18n.language)}
                      </span>
                      <span className="badge badge-outline border-[var(--border-subtle)] text-xs text-base-content/60">
                        {resolveSourceLabel(selectedDetail.sourceType, selectedDetail.sourceMarket, t)}
                      </span>
                      {selectedDetail.sourceUrl && (
                        <button
                          type="button"
                          onClick={() =>
                            void openSourceReference(selectedDetail.sourceUrl!).catch(
                              logSourceOpenFailure,
                            )
                          }
                          className="flex items-center gap-1 text-xs text-primary hover:underline"
                        >
                          <i className="hn hn-external-link"></i>
                          {t('repository.source')}
                        </button>
                      )}
                    </div>
                  </div>
                ) : null}
              </div>
              <button 
                className="btn btn-circle btn-ghost btn-sm text-base-content/50 hover:bg-base-content/10 hover:text-base-content" 
                onClick={closeDetail}
              >
                <i className="hn hn-times text-lg"></i>
              </button>
            </div>

            {/* Modal Content */}
            <div className="overflow-y-auto p-8 custom-scrollbar">
              {detailLoading ? (
                <div className="flex justify-center py-12">
                  <span className="loading loading-spinner loading-lg text-primary"></span>
                </div>
              ) : detailError ? (
                <div className="rounded border border-error/20 bg-error/5 p-4 text-sm text-error">
                  {detailError}
                </div>
              ) : selectedDetail ? (
                <div className="prose prose-base max-w-none dark:prose-invert">
                  <pre className="overflow-x-auto whitespace-pre-wrap rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-input)] p-6 font-mono text-sm leading-relaxed text-base-content/80 shadow-inner">
                    {selectedDetail.skillMarkdown}
                  </pre>
                </div>
              ) : null}
            </div>
          </div>
        </div>
      ) : null}

      {deletePreview || deletePreviewLoading || deletePreviewError ? (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-[var(--bg-modal-overlay)] p-6 backdrop-blur-sm transition-all duration-300">
          <div className="relative flex max-h-[85vh] w-full max-w-2xl flex-col overflow-hidden rounded-xl border border-[var(--border-subtle)] bg-[var(--bg-modal-panel)] shadow-[0_0_50px_rgba(0,0,0,0.5)]">
            <div className="flex items-start justify-between gap-4 border-b border-[var(--border-subtle)] bg-base-100/50 px-8 py-6 backdrop-blur-md">
              <div className="min-w-0">
                <h3 className="truncate text-2xl font-bold text-base-content">
                  {t('repository.deleteConfirmTitle')}
                </h3>
                <p className="mt-2 text-sm leading-relaxed text-base-content/60">
                  {deletePreview
                    ? t('repository.deleteConfirmBody', { name: deletePreview.skillName })
                    : t('repository.deleteConfirmLoading')}
                </p>
              </div>
              <button
                className="btn btn-circle btn-ghost btn-sm text-base-content/50 hover:bg-base-content/10 hover:text-base-content"
                onClick={handleCloseDeletePreview}
              >
                <i className="hn hn-times text-lg"></i>
              </button>
            </div>

            <div className="overflow-y-auto p-8 custom-scrollbar">
              {deletePreviewLoading ? (
                <div className="flex justify-center py-12">
                  <span className="loading loading-spinner loading-lg text-primary"></span>
                </div>
              ) : deletePreviewError ? (
                <div className="rounded border border-error/20 bg-error/5 p-4 text-sm text-error">
                  {deletePreviewError}
                </div>
              ) : deletePreview ? (
                <div className="space-y-5">
                  <div className="rounded-lg border border-warning/20 bg-warning/5 p-4 text-sm leading-6 text-base-content/75">
                    {t('repository.deleteConfirmWarning')}
                  </div>
                  <div className="rounded-lg border border-[var(--border-subtle)] bg-base-200/30 p-4">
                    <p className="text-xs font-semibold uppercase tracking-wide text-base-content/40">
                      {t('repository.deleteCanonicalPath')}
                    </p>
                    <p className="mt-2 break-all font-mono text-xs text-base-content/50">
                      {normalizeDisplayPath(deletePreview.canonicalPath)}
                    </p>
                  </div>
                  <div className="rounded-lg border border-[var(--border-subtle)] bg-base-200/30 p-4">
                    <div className="flex items-center justify-between gap-3">
                      <p className="text-xs font-semibold uppercase tracking-wide text-base-content/40">
                        {t('repository.deleteDistributedPaths')}
                      </p>
                      <span className="badge badge-outline">
                        {t('repository.deleteDistributedCount', {
                          count: deletePreview.distributionPaths.length,
                        })}
                      </span>
                    </div>
                    {deletePreview.distributionPaths.length === 0 ? (
                      <p className="mt-2 text-sm text-base-content/60">
                        {t('repository.deleteNoDistributions')}
                      </p>
                    ) : (
                      <ul className="mt-3 space-y-2">
                        {deletePreview.distributionPaths.map((path) => (
                          <li
                            key={path}
                            className="break-all rounded bg-base-100/60 px-3 py-2 font-mono text-xs text-base-content/55"
                          >
                            {normalizeDisplayPath(path)}
                          </li>
                        ))}
                      </ul>
                    )}
                  </div>
                </div>
              ) : null}
            </div>

            <div className="flex items-center justify-end gap-3 border-t border-[var(--border-subtle)] px-8 py-5">
              <button
                className="btn btn-ghost"
                onClick={handleCloseDeletePreview}
                disabled={uninstallingSkillId !== null}
              >
                {t('common.cancel')}
              </button>
              <button
                className="btn btn-error"
                onClick={() => deletePreview ? void uninstall(deletePreview.skillId) : undefined}
                disabled={!deletePreview || uninstallingSkillId === deletePreview.skillId}
              >
                {deletePreview && uninstallingSkillId === deletePreview.skillId ? (
                  <span className="loading loading-spinner loading-xs"></span>
                ) : (
                  <i className="hn hn-trash"></i>
                )}
                {t('repository.confirmUninstall')}
              </button>
            </div>
          </div>
        </div>
      ) : null}


      <RepositoryImportModal
        open={importOpen}
        resolving={resolvingImport}
        importing={importing}
        importError={importError}
        importBlockedReport={importBlockedReport}
        resolvedImport={resolvedImport}
        existingSlugs={items.map((item) => item.slug)}
        onReset={resetImportState}
        onClose={closeImportModal}
        onResolve={handleResolveImport}
        onImport={handleImportSkill}
      />

      <RepositoryDistributeModal
        open={distributionOpen}
        repositorySkills={items}
        targets={visibleTargets}
        distributing={distributing}
        error={distributionError}
        result={lastDistributionResult}
        onClose={handleCloseDistribution}
        onSubmit={handleBatchDistributeSkills}
      />
    </div>
  )
}
