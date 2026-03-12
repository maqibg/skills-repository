import test from 'node:test'
import assert from 'node:assert/strict'
import fs from 'node:fs'

const zh = JSON.parse(
  fs.readFileSync('E:/vibecoding-project/skills-repository/src/locales/zh-CN/common.json', 'utf8'),
)
const ja = JSON.parse(
  fs.readFileSync('E:/vibecoding-project/skills-repository/src/locales/ja-JP/common.json', 'utf8'),
)
const securityPage = fs.readFileSync(
  'E:/vibecoding-project/skills-repository/src/pages/SecurityPage.tsx',
  'utf8',
)

test('security locale labels do not contain placeholder question marks', () => {
  const zhSecurity = zh.security
  const jaSecurity = ja.security

  const values = [
    zhSecurity.cards.review.label,
    zhSecurity.blockingReasonsTitle,
    zhSecurity.line,
    zhSecurity.evidence,
    zhSecurity.categories.system,
    zhSecurity.categories.source,
    zhSecurity.fileKinds.cmd,
    zhSecurity.fileKinds.script,
    zhSecurity.fileKinds.archive,
    zhSecurity.fileKinds.binary,
    zhSecurity.fileKinds.unknown,
    jaSecurity.cards.review.label,
    jaSecurity.blockingReasonsTitle,
  ]

  for (const value of values) {
    assert.ok(!value.includes('???'), `unexpected placeholder text: ${value}`)
    assert.ok(!value.includes('??'), `unexpected placeholder text: ${value}`)
  }
})

test('security page does not contain garbled separator text', () => {
  assert.ok(!securityPage.includes(' 路 '), 'garbled separator found in SecurityPage')
})
