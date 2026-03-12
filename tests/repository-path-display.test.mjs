import test from 'node:test'
import assert from 'node:assert/strict'
import fs from 'node:fs'

const repositoryPageSource = fs.readFileSync(
  'E:/vibecoding-project/skills-repository/src/pages/RepositoryPage.tsx',
  'utf8',
)

test('repository detail modal should not render canonicalPath directly', () => {
  assert.ok(
    !repositoryPageSource.includes('{selectedDetail.canonicalPath}'),
    'RepositoryPage still renders selectedDetail.canonicalPath directly',
  )
})
