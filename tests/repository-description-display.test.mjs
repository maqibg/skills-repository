import test from 'node:test'
import assert from 'node:assert/strict'
import fs from 'node:fs'

const repositoryPageSource = fs.readFileSync(
  'E:/vibecoding-project/skills-repository/src/pages/RepositoryPage.tsx',
  'utf8',
)

test('repository list renders description beneath skill name', () => {
  assert.ok(
    repositoryPageSource.includes("resolveDescription(item.description, t)"),
    'RepositoryPage should render resolved description fallback in the list row',
  )
})

test('repository detail renders summary before markdown content', () => {
  assert.ok(
    repositoryPageSource.includes("resolveDescription(selectedDetail.description, t)"),
    'RepositoryPage should render resolved detail description fallback in detail header',
  )
  assert.ok(
    repositoryPageSource.includes("t('repository.summaryTitle')"),
    'RepositoryPage should show a summary title in detail view',
  )
})
