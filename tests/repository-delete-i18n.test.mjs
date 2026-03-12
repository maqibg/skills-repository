import test from 'node:test'
import assert from 'node:assert/strict'
import fs from 'node:fs'

const zh = JSON.parse(
  fs.readFileSync('E:/vibecoding-project/skills-repository/src/locales/zh-CN/common.json', 'utf8'),
)
const ja = JSON.parse(
  fs.readFileSync('E:/vibecoding-project/skills-repository/src/locales/ja-JP/common.json', 'utf8'),
)
const page = fs.readFileSync(
  'E:/vibecoding-project/skills-repository/src/pages/RepositoryPage.tsx',
  'utf8',
)

function assertDeleteConfirmationLocale(locale, localeName) {
  const values = [
    locale.repository.deleteConfirmTitle,
    locale.repository.deleteConfirmBody,
    locale.repository.deleteConfirmLoading,
    locale.repository.deleteConfirmWarning,
    locale.repository.deleteCanonicalPath,
    locale.repository.deleteDistributedPaths,
    locale.repository.deleteDistributedCount,
    locale.repository.deleteNoDistributions,
    locale.repository.confirmUninstall,
    locale.common.cancel,
  ]

  for (const value of values) {
    assert.equal(typeof value, 'string')
    assert.ok(!value.includes('???'), `${localeName} unexpected placeholder text: ${value}`)
    assert.ok(!value.includes('??'), `${localeName} unexpected placeholder text: ${value}`)
  }
}

test('zh-CN delete confirmation locale should not contain question-mark placeholders', () => {
  assertDeleteConfirmationLocale(zh, 'zh-CN')
})

test('ja-JP delete confirmation locale should not contain question-mark placeholders', () => {
  assertDeleteConfirmationLocale(ja, 'ja-JP')
})

test('repository page should reference a defined cancel label', () => {
  assert.ok(page.includes("t('common.cancel')"))
  assert.equal(typeof zh.common.cancel, 'string')
  assert.equal(typeof ja.common.cancel, 'string')
})
