import test from 'node:test'
import assert from 'node:assert/strict'
import fs from 'node:fs'

const repositoryPageSource = fs.readFileSync(
  'E:/vibecoding-project/skills-repository/src/pages/RepositoryPage.tsx',
  'utf8',
)

test('repository page loads delete preview before uninstalling', () => {
  assert.ok(
    repositoryPageSource.includes('handleOpenDeletePreview'),
    'RepositoryPage should define a delete preview handler',
  )
  assert.ok(
    repositoryPageSource.includes('handleOpenDeletePreview(item.id)'),
    'RepositoryPage should load delete preview from row actions',
  )
  assert.ok(
    !repositoryPageSource.includes('onClick={() => void uninstall(item.id)}'),
    'RepositoryPage should not uninstall directly from the row button',
  )
})

test('repository page renders distributed paths in delete confirmation', () => {
  assert.ok(
    repositoryPageSource.includes("t('repository.deleteDistributedPaths')"),
    'RepositoryPage should render distributed path title in delete confirmation',
  )
  assert.ok(
    repositoryPageSource.includes('deletePreview.distributionPaths.map'),
    'RepositoryPage should list distributed paths in delete confirmation',
  )
})
