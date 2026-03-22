export interface HighlightRange {
  start: number
  end: number
}

export interface RepositorySearchSourceItem {
  id: string
  name: string
  slug: string
  description?: string | null
  sourceLabel: string
  statusLabel: string
  keywords?: string[]
}

interface IndexedField {
  normalized: string
  map: Array<{ start: number; end: number }>
}

interface RepositorySearchFields {
  name: IndexedField
  slug: IndexedField
  description: IndexedField
  source: IndexedField
  status: IndexedField
}

interface SearchFieldMatch {
  key: keyof RepositorySearchFields
  score: number
  ranges: HighlightRange[]
}

export interface RepositorySearchIndexItem<TItem extends RepositorySearchSourceItem> {
  item: TItem
  fields: RepositorySearchFields
  searchableText: string
}

export interface RepositorySearchMatch<TItem extends RepositorySearchSourceItem> {
  item: TItem
  score: number
  highlights: {
    name: HighlightRange[]
    slug: HighlightRange[]
    description: HighlightRange[]
    source: HighlightRange[]
  }
}

export interface RepositorySearchPage<TItem> {
  items: TItem[]
  page: number
  pageCount: number
  pageSize: number
  total: number
  startIndex: number
  endIndex: number
}

const FIELD_WEIGHTS: Record<keyof RepositorySearchFields, number> = {
  name: 5,
  slug: 4,
  description: 3,
  source: 2,
  status: 1,
}

const normalizeSearchValue = (value: string) =>
  value.normalize('NFKC').toLocaleLowerCase()

const normalizeSearchQuery = (value: string) =>
  normalizeSearchValue(value)
    .replace(/\s+/g, ' ')
    .trim()

const extractTokens = (query: string) =>
  normalizeSearchQuery(query)
    .split(' ')
    .filter(Boolean)

const buildNormalizedMap = (value: string): IndexedField => {
  const map: Array<{ start: number; end: number }> = []
  let normalized = ''
  let offset = 0

  for (const character of value) {
    const start = offset
    offset += character.length
    const normalizedCharacter = normalizeSearchValue(character)

    for (const normalizedUnit of normalizedCharacter) {
      normalized += normalizedUnit
      map.push({ start, end: offset })
    }
  }

  return { normalized, map }
}

const buildHighlightRange = (field: IndexedField, start: number, end: number) => {
  const first = field.map[start]
  const last = field.map[end - 1]
  if (!first || !last) {
    return null
  }

  return {
    start: first.start,
    end: last.end,
  }
}

const mergeHighlightRanges = (ranges: HighlightRange[]) => {
  if (ranges.length <= 1) {
    return ranges
  }

  const sorted = [...ranges].sort((left, right) => left.start - right.start || left.end - right.end)
  const merged: HighlightRange[] = [{ ...sorted[0] }]

  for (const range of sorted.slice(1)) {
    const previous = merged[merged.length - 1]
    if (range.start <= previous.end) {
      previous.end = Math.max(previous.end, range.end)
      continue
    }

    merged.push({ ...range })
  }

  return merged
}

const matchSubstring = (field: IndexedField, token: string) => {
  const index = field.normalized.indexOf(token)
  if (index === -1) {
    return null
  }

  const range = buildHighlightRange(field, index, index + token.length)
  return range ? [range] : []
}

const matchSubsequence = (field: IndexedField, token: string) => {
  const positions: number[] = []
  let cursor = 0

  for (const character of token) {
    const position = field.normalized.indexOf(character, cursor)
    if (position === -1) {
      return null
    }
    positions.push(position)
    cursor = position + 1
  }

  return positions
    .map((position) => buildHighlightRange(field, position, position + 1))
    .filter((range): range is HighlightRange => Boolean(range))
}

const scoreFieldMatch = (
  field: IndexedField,
  key: keyof RepositorySearchFields,
  token: string,
): SearchFieldMatch | null => {
  const substringRanges = matchSubstring(field, token)
  if (substringRanges) {
    return {
      key,
      score: FIELD_WEIGHTS[key],
      ranges: substringRanges,
    }
  }

  const subsequenceRanges = matchSubsequence(field, token)
  if (!subsequenceRanges) {
    return null
  }

  return {
    key,
    score: FIELD_WEIGHTS[key] * 0.35,
    ranges: subsequenceRanges,
  }
}

const createEmptyHighlights = () => ({
  name: [] as HighlightRange[],
  slug: [] as HighlightRange[],
  description: [] as HighlightRange[],
  source: [] as HighlightRange[],
})

const collectHighlightRanges = (
  highlights: ReturnType<typeof createEmptyHighlights>,
  match: SearchFieldMatch,
) => {
  if (match.key === 'name') {
    highlights.name.push(...match.ranges)
  } else if (match.key === 'slug') {
    highlights.slug.push(...match.ranges)
  } else if (match.key === 'description') {
    highlights.description.push(...match.ranges)
  } else if (match.key === 'source') {
    highlights.source.push(...match.ranges)
  }
}

const buildSearchableText = (item: RepositorySearchSourceItem) =>
  normalizeSearchQuery(
    [
      item.name,
      item.slug,
      item.description ?? '',
      item.sourceLabel,
      item.statusLabel,
      ...(item.keywords ?? []),
    ].join(' '),
  )

const selectBestFieldMatch = (fields: RepositorySearchFields, token: string) => {
  const matches = (Object.keys(fields) as Array<keyof RepositorySearchFields>)
    .map((key) => scoreFieldMatch(fields[key], key, token))
    .filter((value): value is SearchFieldMatch => Boolean(value))
    .sort((left, right) => right.score - left.score)

  return matches[0] ?? null
}

const hasAnyFuzzyMatch = (fields: RepositorySearchFields, token: string) =>
  (Object.keys(fields) as Array<keyof RepositorySearchFields>).some((key) =>
    Boolean(matchSubsequence(fields[key], token)),
  )

const finalizeHighlights = (highlights: ReturnType<typeof createEmptyHighlights>) => ({
  name: mergeHighlightRanges(highlights.name),
  slug: mergeHighlightRanges(highlights.slug),
  description: mergeHighlightRanges(highlights.description),
  source: mergeHighlightRanges(highlights.source),
})

export const buildRepositorySearchIndex = <TItem extends RepositorySearchSourceItem>(
  items: TItem[],
): RepositorySearchIndexItem<TItem>[] =>
  items.map((item) => ({
    item,
    fields: {
      name: buildNormalizedMap(item.name),
      slug: buildNormalizedMap(item.slug),
      description: buildNormalizedMap(item.description ?? ''),
      source: buildNormalizedMap(item.sourceLabel),
      status: buildNormalizedMap(item.statusLabel),
    },
    searchableText: buildSearchableText(item),
  }))

export const searchRepositoryIndex = <TItem extends RepositorySearchSourceItem>(
  index: RepositorySearchIndexItem<TItem>[],
  query: string,
): RepositorySearchMatch<TItem>[] => {
  const tokens = extractTokens(query)

  if (tokens.length === 0) {
    return index.map((entry) => ({
      item: entry.item,
      score: 0,
      highlights: createEmptyHighlights(),
    }))
  }

  return index
    .flatMap((entry) => {
      const highlights = createEmptyHighlights()
      let score = 0

      for (const token of tokens) {
        if (!entry.searchableText.includes(token) && !hasAnyFuzzyMatch(entry.fields, token)) {
          return []
        }

        const bestMatch = selectBestFieldMatch(entry.fields, token)
        if (!bestMatch) {
          return []
        }

        score += bestMatch.score
        collectHighlightRanges(highlights, bestMatch)
      }

      return [{
        item: entry.item,
        score,
        highlights: finalizeHighlights(highlights),
      }]
    })
    .sort((left, right) => right.score - left.score || left.item.name.localeCompare(right.item.name))
}

export const paginateRepositorySearchResults = <TItem>(
  results: TItem[],
  page: number,
  pageSize: number,
): RepositorySearchPage<TItem> => {
  const safePageSize = Math.max(1, pageSize)
  const total = results.length
  const pageCount = Math.max(1, Math.ceil(total / safePageSize))
  const safePage = Math.min(Math.max(1, page), pageCount)
  const startIndex = total === 0 ? 0 : (safePage - 1) * safePageSize
  const endIndex = total === 0 ? 0 : Math.min(total, startIndex + safePageSize)

  return {
    items: results.slice(startIndex, endIndex),
    page: safePage,
    pageCount,
    pageSize: safePageSize,
    total,
    startIndex,
    endIndex,
  }
}

export const buildRepositoryPageNumbers = (
  currentPage: number,
  pageCount: number,
  windowSize = 5,
) => {
  if (pageCount <= 1) {
    return [1]
  }

  const safeWindowSize = Math.max(3, windowSize)
  const halfWindow = Math.floor(safeWindowSize / 2)
  const start = Math.max(1, Math.min(currentPage - halfWindow, pageCount - safeWindowSize + 1))
  const end = Math.min(pageCount, start + safeWindowSize - 1)

  return Array.from({ length: end - start + 1 }, (_, index) => start + index)
}
